use std::sync::Arc;
use tokio::sync::RwLock;

use crate::rpc::mining::mining::BlockHeader;
use crate::rpc::state::encode;
use crate::transaction::Tx;

#[derive(Debug, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Tx>,
}

impl Block {
    pub fn genesis() -> Self {
        let header = BlockHeader {
            version: 1,
            prev_block_hash: [0u8; 32],
            merkle_root: [0u8; 32],
            timestamp: 1231006505,
            bits: 0x1d00ffff,
            nonce: 2083236893,
        };
        Self {
            header,
            transactions: Vec::new(),
        }
    }

    pub fn hash(&self) -> [u8; 32] {
        self.header.hash()
    }

    pub fn hash_hex(&self) -> String {
        encode(&self.hash())
    }
}

#[derive(Debug, Clone)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        Self {
            blocks: vec![Block::genesis()],
        }
    }

    pub fn height(&self) -> u64 {
        self.blocks.len() as u64 - 1
    }

    pub fn best_block_hash(&self) -> [u8; 32] {
        self.blocks.last().unwrap().hash()
    }

    pub fn best_block_hash_hex(&self) -> String {
        encode(&self.best_block_hash())
    }

    pub fn add_block(&mut self, block: Block) -> Result<u64, &'static str> {
        // Rule: Previous block hash must match current tip
        if block.header.prev_block_hash != self.best_block_hash() {
            return Err("Block rejected: prev_block_hash does not match chain tip");
        }

        // Rule: Header must satisfy proof of work
        if !block.header.meets_difficulty(1) {
            return Err("Block rejected: invalid proof of work");
        }

        self.blocks.push(block);
        Ok(self.height())
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedBlockchain = Arc<RwLock<Blockchain>>;
