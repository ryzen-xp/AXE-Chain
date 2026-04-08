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
    pub signature: Hash,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionOutput {
    pub value: u64,
    pub unique_key: Uuid,
    pub pubkey: Hash,
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

    pub fn verify_transactions(&self) {}

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
