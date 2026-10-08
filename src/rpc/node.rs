use std::sync::Arc;
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use serde::{Deserialize, Serialize};

use crate::p2p::Swarm;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainInfo {
    pub chain: String,
    pub blocks: u64,
    pub best_block_hash: String,
    pub difficulty: u32,
    pub verification_progress: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub version: String,
    pub subversion: String,
    pub protocol_version: u32,
    pub connections: u32,
    pub network_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolInfo {
    pub size: usize,
    pub bytes: usize,
    pub usage: usize,
}

#[rpc(server)]
pub trait NodeRpc {
    #[method(name = "getblockchaininfo")]
    async fn get_blockchain_info(&self) -> RpcResult<BlockchainInfo>;

    #[method(name = "getnetworkinfo")]
    async fn get_network_info(&self) -> RpcResult<NetworkInfo>;

    #[method(name = "getmempoolinfo")]
    async fn get_mempool_info(&self) -> RpcResult<MempoolInfo>;
}

pub struct NodeRpcServerImpl {
    pub swarm: Option<Arc<Swarm>>,
    pub chain: Option<crate::blockchain::SharedBlockchain>,
}

impl NodeRpcServerImpl {
    pub fn new(swarm: Arc<Swarm>, chain: crate::blockchain::SharedBlockchain) -> Self {
        Self {
            swarm: Some(swarm),
            chain: Some(chain),
        }
    }
}

impl Default for NodeRpcServerImpl {
    fn default() -> Self {
        Self {
            swarm: None,
            chain: None,
        }
    }
}

#[jsonrpsee::core::async_trait]
impl NodeRpcServer for NodeRpcServerImpl {
    async fn get_blockchain_info(&self) -> RpcResult<BlockchainInfo> {
        let (blocks, best_block_hash) = if let Some(chain) = &self.chain {
            let c = chain.read().await;
            (c.height(), c.best_block_hash_hex())
        } else {
            (0, "0000000000000000000000000000000000000000000000000000000000000000".to_string())
        };

        Ok(BlockchainInfo {
            chain: "toyBitcoinScript".to_string(),
            blocks,
            best_block_hash,
            difficulty: 1,
            verification_progress: 1.0,
        })
    }

    async fn get_network_info(&self) -> RpcResult<NetworkInfo> {
        let connections = if let Some(swarm) = &self.swarm {
            swarm.peer_count().await as u32
        } else {
            0
        };

        Ok(NetworkInfo {
            version: "0.1.0".to_string(),
            subversion: "/Jay's Bitcoin Node:0.1.0/".to_string(),
            protocol_version: 70015,
            connections,
            network_active: true,
        })
    }

    async fn get_mempool_info(&self) -> RpcResult<MempoolInfo> {
        let (size, bytes, usage) = if let Some(swarm) = &self.swarm {
            swarm.mempool_info().await
        } else {
            (0, 0, 0)
        };

        Ok(MempoolInfo {
            size,
            bytes,
            usage,
        })
    }
}
