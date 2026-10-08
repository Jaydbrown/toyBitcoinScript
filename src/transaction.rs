use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxIn {
    pub tx_id: [u8; 32],
    pub vout: u32,
    pub sig: String,
    pub sequence: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxOut {
    pub value: u64,
    pub scriptPubkey: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tx {
    pub version: u32,
    pub inputs: Vec<TxIn>,
    pub outputs: Vec<TxOut>,
    pub locktime: u32,
}

impl Tx {
    pub fn new(version: u32, locktime: u32) -> Self {
        Self {
            version,
            inputs: Vec::new(),
            outputs: Vec::new(),
            locktime,
        }
    }

    pub fn tx_in(&mut self, tx: TxIn) {
        self.inputs.push(tx)
    }

    pub fn tx_out(&mut self, tx: TxOut) {
        self.outputs.push(tx)
    }
}
