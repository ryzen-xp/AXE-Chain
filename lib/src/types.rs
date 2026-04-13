use crate::crypto::{PublicKey, Signature};
use crate::error::CoinError;
use crate::{U256, sha256::Hash, util::MerkleRoot};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Blockchain {
    utxos: HashMap<Hash, TransactionOutput>,
    target: U256,
    blocks: Vec<Block>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub nonce: U256,
    pub transactions: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockHeader {
    pub timestamp: DateTime<Utc>,
    pub nonce: u64,
    pub prev_block_hash: Hash,
    pub merkle_root: MerkleRoot,
    pub target: U256,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionInput {
    pub prev_transaction_output_hash: Hash,
    pub signature: Signature,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionOutput {
    pub value: u64,
    pub unique_key: Uuid,
    pub pubkey: PublicKey,
}

//  Implimented the  Blockchain methods !

impl Blockchain {
    pub fn new() -> Self {
        Blockchain {
            utxos: HashMap::new(),
            target: crate::MIN_TARGET,
            blocks: vec![],
        }
    }

    pub fn utxos(&self) -> &HashMap<Hash, TransactionOutput> {
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
                    self.utxos.insert(tx.hash(), output.clone());
                }
            }
        }
    }
}

///   Impl Block Type

impl Block {
    pub fn new(header: BlockHeader, nonce: U256, transactions: Vec<Transaction>) -> Self {
        Block {
            header,
            nonce,
            transactions,
        }
    }

    pub fn verify_transactions(
        &self,
        _predicted_block_hieght: u64,
        utxo: &HashMap<Hash, TransactionOutput>,
    ) -> Result<(), CoinError> {
        let mut inputs: HashMap<Hash, TransactionOutput> = HashMap::new();

        if self.transactions.is_empty() {
            return Err(CoinError::InvalidTransaction);
        }

        for tx in &self.transactions {
            let mut input = 0;
            let mut output = 0;

            for ip in &tx.inputs {
                let prev_output = utxo.get(&ip.prev_transaction_output_hash);

                if prev_output.is_none() {
                    return Err(CoinError::InvalidTransaction);
                }

                let previous_output = prev_output.unwrap();

                //  checking  that the same output not spend  more then one .

                if inputs.contains_key(&ip.prev_transaction_output_hash) {
                    return Err(CoinError::InvalidTransactionOutput);
                }

                //  now verify that the signer is correct or not

                if ip
                    .signature
                    .verify(&ip.prev_transaction_output_hash, &previous_output.pubkey)
                {
                    return Err(CoinError::InvalidSignature);
                }

                input = input + previous_output.value;

                inputs.insert(ip.prev_transaction_output_hash, previous_output.clone());
            }

            for op in &tx.outputs {
                output = output + op.value;
            }

            if input < output {
                return Err(CoinError::InvalidTransaction);
            }
        }

        Ok(())
    }

    pub fn verify_coinbase_transactions(
        &self,
        predicted_block_height: u64,
        utxo: HashMap<Hash, TransactionOutput>,
    ) -> Result<(), CoinError> {
        let coinbase_tx = &self.transactions[0];

        if !coinbase_tx.inputs.is_empty() {
            return Err(CoinError::InvalidTransaction);
        }

        if coinbase_tx.outputs.is_empty() {
            return Err(CoinError::InvalidTransaction);
        }

        let miner_reward = self.calculate_miner_reward(utxo).unwrap();

        let block_reward = (crate::INITIAL_REWARD as u64) * 10u64.pow(8)
            / 2u64.pow((predicted_block_height / crate::HALVING_INTERVAL as u64) as u32);

        let total_coinbase_tx_outputs_value: u64 =
            coinbase_tx.outputs.iter().map(|op| op.value).sum();

        if total_coinbase_tx_outputs_value < block_reward + miner_reward {
            return Err(CoinError::InvalidTransaction);
        }
        Ok(())
    }

    pub fn calculate_miner_reward(
        &self,
        utxo: HashMap<Hash, TransactionOutput>,
    ) -> Result<u64, CoinError> {
        let mut inputs: HashMap<Hash, TransactionOutput> = HashMap::new();

        let mut outputs: HashMap<Hash, TransactionOutput> = HashMap::new();

        for tx in self.transactions.iter().skip(1) {
            for input in &tx.inputs {
                if inputs.contains_key(&input.prev_transaction_output_hash) {
                    return Err(CoinError::InvalidTransaction);
                }

                let option_prv_output = utxo.get(&input.prev_transaction_output_hash);

                if option_prv_output.is_none() {
                    return Err(CoinError::InvalidTransaction);
                }

                let prev_output = option_prv_output.unwrap();

                inputs.insert(input.prev_transaction_output_hash, prev_output.clone());
            }

            for output in &tx.outputs {
                if outputs.contains_key(&output.hash()) {
                    return Err(CoinError::InvalidTransaction);
                }

                outputs.insert(output.hash(), output.clone());
            }
        }

        let input_value: u64 = inputs.values().map(|x| x.value).sum();

        let output_value: u64 = outputs.values().map(|x| x.value).sum();

        Ok(input_value - output_value)
    }

    pub fn mine_blocks(&mut self, steps: usize) -> bool {
        if self.hash().match_target(self.header.target) {
            return true;
        }

        for _ in 0..steps {
            if let Some(nonce) = self.nonce.checked_add(U256::from(1u64)) {
                self.nonce = nonce;
            }

            if self.hash().match_target(self.header.target) {
                return true;
            }
        }

        false
    }

    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }
}

impl BlockHeader {
    pub fn new(
        timestamp: DateTime<Utc>,
        nonce: u64,
        prev_block_hash: Hash,
        merkle_root: MerkleRoot,
        target: U256,
    ) -> Self {
        BlockHeader {
            timestamp,
            nonce,
            prev_block_hash,
            merkle_root,
            target,
        }
    }

    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }
}

impl Transaction {
    pub fn new(inputs: Vec<TransactionInput>, outputs: Vec<TransactionOutput>) -> Self {
        Transaction { inputs, outputs }
    }

    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }
}

impl TransactionOutput {
    pub fn hash(&self) -> Hash {
        Hash::hash(&self)
    }
}
