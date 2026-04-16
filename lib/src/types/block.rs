use super::{Transaction, TransactionOutput};
use crate::error::CoinError;
use crate::{U256, sha256::Hash, util::MerkleRoot};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
