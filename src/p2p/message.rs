use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum P2pMessage {
    Ping(u64),
    Pong(u64),
    NewTransaction { tx_hex: String },
    NewBlock { block_hex: String },
    GetBlocks { from_height: u64 },
}

