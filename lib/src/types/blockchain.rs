impl Blockchain {
    pub fn new() -> Self {
        Blockchain {
            utxos: HashMap::new(),
            target: crate::MIN_TARGET,
            blocks: vec![],
            mempool: vec![],
        }
    }

    pub fn utxos(&self) -> &HashMap<Hash, (bool, TransactionOutput)> {
        &self.utxos
    }

    pub fn target(&self) -> U256 {
        self.target
    }

    pub fn blocks(&self) -> impl Iterator<Item = &Block> {
        self.blocks.iter()
    }

    pub fn block_hieght(&self) -> u64 {
        self.blocks.len() as u64
    }

    pub fn mempool(&self) -> &[(DateTime<Utc>, Transaction)] {
        &self.mempool
    }

    pub fn add_block(&mut self, block: Block) -> Result<(), CoinError> {
        if self.blocks.is_empty() {
            //  Checking that the previous block  is zero or not if zero it's mean it's first block of chain !
            if block.header.prev_block_hash != Hash::zero() {
                println!("zero hash!!");

                return Err(CoinError::InvalidBlock);
            }
        } else {
            let last_block = self.blocks.last().unwrap();

            if block.header.prev_block_hash != last_block.hash() {
                println!("Prev Block Hash is Wrong!!");

                return Err(CoinError::InvalidBlock);
            }

            if !block.header.hash().match_target(block.header.target) {
                println!("Does not match target");
                return Err(CoinError::InvalidBlock);
            }

            let merkle_root = MerkleRoot::calculate(&block.transactions);

            if merkle_root != block.header.merkle_root {
                println!("Mismatch Merkle root !!");

                return Err(CoinError::InvalidBlock);
            }

            // Checking  timestamp of the blocks , new block timestamp should be greaater then the  prev_block timestamp !!

            if block.header.timestamp <= last_block.header.timestamp {
                println!("Timestamp mismatch !!");
                return Err(CoinError::InvalidBlock);
            }

            //  verifying all tx in blocks
        }

        let _block_transactions: HashSet<_> = block.transactions.iter().map(|x| x.hash()).collect();

        // self.mempool
        //     .retain(|(_, tx)| !block_transactions.contains(&tx.hash()));
        self.blocks.push(block);

        //  This used to adjust the defficulty
        self.try_adjust_target();

        Ok(())
    }

    pub fn try_adjust_target(&mut self) {
        if self.blocks.is_empty() {
            return;
        }
        if self.blocks.len() % crate::DIFFICULTY_UPDATE_INTERVAL as usize != 0 {
            return;
        }
        // measure the time it took to mine the last

        // crate::DIFFICULTY_UPDATE_INTERVAL blocks
        // with chrono
        let start_time = self.blocks
            [self.blocks.len() - crate::DIFFICULTY_UPDATE_INTERVAL as usize]
            .header
            .timestamp;
        let end_time = self.blocks.last().unwrap().header.timestamp;
        let time_diff = end_time - start_time;
        // convert time_diff to seconds
        let time_diff_seconds = time_diff.num_seconds();
        // calculate the ideal number of seconds
        let target_seconds = crate::IDEAL_BLOCK_TIME * crate::DIFFICULTY_UPDATE_INTERVAL;
        // multiply the current target by actual time divided by

        let new_target = BigDecimal::parse_bytes(self.target.to_string().as_bytes(), 10)
            .expect("Failed to convert ")
            * (BigDecimal::from(time_diff_seconds) / BigDecimal::from(target_seconds));

        let new_target_str = new_target
            .to_string()
            .split(".")
            .next()
            .expect("Failed to convert")
            .to_owned();

        let new_target_u256 =
            U256::from_str_radix(&new_target_str, 10).expect("Failed to  convert into U256");
        // clamp new_target to be within the range of
        // 4 * self.target and self.target / 4
        let new_target = if new_target_u256 < self.target / 4 {
            self.target / 4
        } else if new_target_u256 > self.target * 4 {
            self.target * 4
        } else {
            new_target_u256
        };
        // if the new target is more than the minimum target,
        // set it to the minimum target
        self.target = new_target.min(crate::MIN_TARGET);
    }

    pub fn rebuild_utxos(&mut self) {
        for block in &self.blocks {
            for tx in &block.transactions {
                for input in &tx.inputs {
                    self.utxos.remove(&input.prev_transaction_output_hash);
                }

                for output in &tx.outputs {
                    self.utxos.insert(tx.hash(), (false, output.clone()));
                }
            }
        }
    }

    pub fn add_tx_mempool(&mut self, tx: Transaction) -> Result<(), CoinError> {
        let mut known_inputs: HashSet<Hash> = HashSet::new();

        for input in &tx.inputs {
            if !self.utxos.contains_key(&input.prev_transaction_output_hash) {
                return Err(CoinError::InvalidTransactionInput);
            }

            if known_inputs.contains(&input.prev_transaction_output_hash) {
                return Err(CoinError::InvalidTransaction);
            }

            known_inputs.insert(input.prev_transaction_output_hash);
        }

        for input in &tx.inputs {
            if let Some((true, _)) = self.utxos.get(&input.prev_transaction_output_hash) {
                let refrencing_transaction =
                    self.mempool.iter().enumerate().find(|(_, (_, tx))| {
                        tx.outputs
                            .iter()
                            .any(|output| output.hash() == input.prev_transaction_output_hash)
                    });

                if let Some((idx, refrencing_transaction)) = refrencing_transaction {
                    for input in &refrencing_transaction.1.inputs {
                        self.utxos
                            .entry(input.prev_transaction_output_hash)
                            .and_modify(|(x, _)| {
                                *x = false;
                            });
                    }

                    self.mempool.remove(idx);
                }
            }
        }

        self.mempool.push((Utc::now(), tx));

        self.mempool.sort_by_key(|tx| {
            let all_inputs =
                tx.1.inputs
                    .iter()
                    .map(|x| {
                        self.utxos
                            .get(&x.prev_transaction_output_hash)
                            .expect("failed to get output")
                            .1
                            .value
                    })
                    .sum::<u64>();

            let all_output = tx.1.outputs.iter().map(|x| x.value).sum::<u64>();

            let miner_fees = all_inputs - all_output;

            miner_fees
        });

        Ok(())
    }

    pub fn clean_mempool(&mut self) -> Result<(), CoinError> {
        let now = Utc::now();

        let mut tx_unmarked: Vec<Hash> = vec![];

        self.mempool.retain(|(date, tx)| {
            let time_diff = now - *date;

            if time_diff > chrono::Duration::seconds(MAX_MEMPOOL_TRANSACTION_AGE as i64) {
                tx_unmarked.extend(tx.inputs.iter().map(|x| x.prev_transaction_output_hash));

                false
            } else {
                true
            }
        });

        for hash in tx_unmarked {
            self.utxos.entry(hash).and_modify(|(x, _)| {
                *x = false;
            });
        }

        Ok(())
    }
}
