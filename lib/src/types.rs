use crate::U256;
use uuid::Uuid;
use chrono::{DateTime , Utc};

pub struct Blockchain {
    pub blocks: Vec<Block>,
}

pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

pub struct BlockHeader {
    pub timestamp: DateTime<Utc>,
    pub nonce: u64,
    pub prev_block_hash: [u8; 32],
    pub merkle_root: [u8; 32],
    pub target: U256,
}

pub struct Transaction {
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
}

pub struct TransactionInput {
    pub prev_transaction_output_hash: [u8; 32],
    pub signature: [u8; 32],
}

pub struct TransactionOutput {
    pub value: u64,
    pub unique_key: Uuid,
    pub pubkey: [u8; 32],
}

impl Blockchain {
    pub fn new() -> Self {
        Blockchain { blocks: vec![] }
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }
}

impl Block {
    pub fn new(header: BlockHeader, transactions: Vec<Transaction>) -> Self {
        Block {
            header: header,
            transactions: transactions,
        }
    }

    pub fn hash(&self) -> ! {
        unimplemented!()
    }
}

impl BlockHeader {
    pub fn new(
        &self,
        timestamp: DateTime<Utc>,
        nonce: u64,
        pre_block_hash: [u8; 32],
        merkle_root: [u8; 32],
        target: U256,
    ) -> Self {
        BlockHeader {
            timestamp,
            nonce,
            prev_block_hash: pre_block_hash,
            merkle_root,
            target,
        }
    }

    pub fn hash(&self) -> ! {
        unimplemented!()
    }
}

impl Transaction {
    pub fn new(&self, input: Vec<TransactionInput>, output: Vec<TransactionOutput>) -> Self {
        Transaction {
            inputs: input,
            outputs: output,
        }
    }

    pub fn hash(&self) -> ! {
        unimplemented!()
    }
}
