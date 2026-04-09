use crate::crypto::{PublicKey, Signature};
use crate::error::CoinError;
use crate::{U256, sha256::Hash, util::MerkleRoot};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Blockchain {
    pub utxos: HashMap<Hash, TransactionOutput>,
    pub blocks: Vec<Block>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    pub header: BlockHeader,
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
            blocks: vec![],
        }
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

        self.blocks.push(block);

        Ok(())
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
    pub fn new(header: BlockHeader, transactions: Vec<Transaction>) -> Self {
        Block {
            header,
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
        _utxo: HashMap<Hash, TransactionOutput>,
    ) -> Result<u64, CoinError> {
        Ok(1)
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
