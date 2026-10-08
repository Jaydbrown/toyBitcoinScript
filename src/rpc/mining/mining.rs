use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use serde::{Deserialize, Serialize};

use super::merkle::{compute_merkle_root, double_sha256};
use crate::rpc::state::{decode, encode};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub version: u32,
    pub prev_block_hash: [u8; 32],
    pub merkle_root: [u8; 32],
    pub timestamp: u32,
    pub bits: u32,
    pub nonce: u32,
}

impl BlockHeader {
    pub fn to_bytes(&self) -> [u8; 80] {
        let mut bytes = [0u8; 80];
        bytes[0..4].copy_from_slice(&self.version.to_le_bytes());
        bytes[4..36].copy_from_slice(&self.prev_block_hash);
        bytes[36..68].copy_from_slice(&self.merkle_root);
        bytes[68..72].copy_from_slice(&self.timestamp.to_le_bytes());
        bytes[72..76].copy_from_slice(&self.bits.to_le_bytes());
        bytes[76..80].copy_from_slice(&self.nonce.to_le_bytes());
        bytes
    }

    pub fn hash(&self) -> [u8; 32] {
        double_sha256(&self.to_bytes())
    }

    pub fn meets_difficulty(&self, leading_zero_bytes: usize) -> bool {
        let hash = self.hash();
        for &byte in &hash[..leading_zero_bytes] {
            if byte != 0 {
                return false;
            }
        }
        true
    }

    /// Checks if the block hash hex starts with `leading_zero_hex_chars` zeroes
    pub fn meets_difficulty_hex(&self, leading_zero_hex_chars: usize) -> bool {
        let hash_hex = encode(&self.hash());
        hash_hex.chars().take(leading_zero_hex_chars).all(|c| c == '0')
    }

    /// Increments `nonce` in a loop until the header's hash meets the byte difficulty target
    pub fn mine(&mut self, leading_zero_bytes: usize) -> (u32, [u8; 32]) {
        while !self.meets_difficulty(leading_zero_bytes) {
            self.nonce = self.nonce.wrapping_add(1);
        }
        (self.nonce, self.hash())
    }

    /// Increments `nonce` in a loop until the header's hash meets the hex characters target
    pub fn mine_hex(&mut self, leading_zero_hex_chars: usize) -> (u32, [u8; 32]) {
        while !self.meets_difficulty_hex(leading_zero_hex_chars) {
            self.nonce = self.nonce.wrapping_add(1);
        }
        (self.nonce, self.hash())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTemplate {
    pub prev_block_hash: String,
    pub height: u64,
    pub target_bits: u32,
    pub transactions: Vec<String>,
    pub cur_time: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitBlockResult {
    pub accepted: bool,
    pub block_hash: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitBlockParams {
    pub header_hex: String,
    pub tx_hexes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateBlockResult {
    pub nonce: u32,
    pub block_hash: String,
    pub header_hex: String,
    pub height: u64,
}

/// Automatically searches for a valid nonce and returns the mined block
pub fn generate_block(leading_zero_bytes: usize) -> GenerateBlockResult {
    let mut header = BlockHeader {
        version: 1,
        prev_block_hash: [0u8; 32],
        merkle_root: [0u8; 32],
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32,
        bits: 0x1d00ffff,
        nonce: 0,
    };

    let (nonce, hash) = header.mine(leading_zero_bytes);

    GenerateBlockResult {
        nonce,
        block_hash: encode(&hash),
        header_hex: encode(&header.to_bytes()),
        height: 1,
    }
}

pub fn verify_and_submit_block(params: SubmitBlockParams) -> Result<SubmitBlockResult, &'static str> {
    let header_bytes = decode(&params.header_hex)?;
    if header_bytes.len() != 80 {
        return Err("Block header must be exactly 80 bytes (160 hex characters)");
    }

    let mut prev_block_hash = [0u8; 32];
    prev_block_hash.copy_from_slice(&header_bytes[4..36]);

    let mut merkle_root = [0u8; 32];
    merkle_root.copy_from_slice(&header_bytes[36..68]);

    let header = BlockHeader {
        version: u32::from_le_bytes(header_bytes[0..4].try_into().unwrap()),
        prev_block_hash,
        merkle_root,
        timestamp: u32::from_le_bytes(header_bytes[68..72].try_into().unwrap()),
        bits: u32::from_le_bytes(header_bytes[72..76].try_into().unwrap()),
        nonce: u32::from_le_bytes(header_bytes[76..80].try_into().unwrap()),
    };

    // Calculate tx hashes to verify merkle root
    let mut tx_hashes = Vec::new();
    for tx_hex in &params.tx_hexes {
        let raw_tx = decode(tx_hex)?;
        tx_hashes.push(double_sha256(&raw_tx));
    }

    if !tx_hashes.is_empty() {
        let computed_root = compute_merkle_root(tx_hashes)?;
        if computed_root != header.merkle_root {
            return Ok(SubmitBlockResult {
                accepted: false,
                block_hash: encode(&header.hash()),
                error: Some("Merkle root mismatch".to_string()),
            });
        }
    }

    let block_hash = encode(&header.hash());
    Ok(SubmitBlockResult {
        accepted: true,
        block_hash,
        error: None,
    })
}

#[rpc(server)]
pub trait MiningRpc {
    #[method(name = "getblocktemplate")]
    async fn get_block_template(&self) -> RpcResult<BlockTemplate>;

    #[method(name = "submitblock")]
    async fn submit_block(&self, params: SubmitBlockParams) -> RpcResult<SubmitBlockResult>;

    #[method(name = "generate")]
    async fn generate(&self, difficulty_zero_bytes: Option<usize>) -> RpcResult<GenerateBlockResult>;
}

pub struct MiningRpcServerImpl {
    pub chain: Option<crate::blockchain::SharedBlockchain>,
}

impl MiningRpcServerImpl {
    pub fn new(chain: crate::blockchain::SharedBlockchain) -> Self {
        Self { chain: Some(chain) }
    }
}

impl Default for MiningRpcServerImpl {
    fn default() -> Self {
        Self { chain: None }
    }
}

#[jsonrpsee::core::async_trait]
impl MiningRpcServer for MiningRpcServerImpl {
    async fn get_block_template(&self) -> RpcResult<BlockTemplate> {
        let (prev_block_hash, height) = if let Some(chain) = &self.chain {
            let c = chain.read().await;
            (c.best_block_hash_hex(), c.height() + 1)
        } else {
            ("0000000000000000000000000000000000000000000000000000000000000000".to_string(), 1)
        };

        let cur_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        Ok(BlockTemplate {
            prev_block_hash,
            height,
            target_bits: 0x1d00ffff,
            transactions: Vec::new(),
            cur_time,
        })
    }

    async fn submit_block(&self, params: SubmitBlockParams) -> RpcResult<SubmitBlockResult> {
        verify_and_submit_block(params).map_err(|err| {
            jsonrpsee::types::ErrorObject::owned(
                -32602,
                err.to_string(),
                None::<()>,
            )
        })
    }

    async fn generate(&self, difficulty_zero_bytes: Option<usize>) -> RpcResult<GenerateBlockResult> {
        let diff = difficulty_zero_bytes.unwrap_or(1);

        let (prev_hash, height) = if let Some(chain) = &self.chain {
            let c = chain.read().await;
            (c.best_block_hash(), c.height() + 1)
        } else {
            ([0u8; 32], 1)
        };

        let mut header = BlockHeader {
            version: 1,
            prev_block_hash: prev_hash,
            merkle_root: [0u8; 32],
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as u32,
            bits: 0x1d00ffff,
            nonce: 0,
        };

        let (nonce, hash) = header.mine(diff);

        if let Some(chain) = &self.chain {
            let mut c = chain.write().await;
            let block = crate::blockchain::Block {
                header: header.clone(),
                transactions: Vec::new(),
            };
            let _ = c.add_block(block);
        }

        Ok(GenerateBlockResult {
            nonce,
            block_hash: encode(&hash),
            header_hex: encode(&header.to_bytes()),
            height,
        })
    }
}
