pub struct TxIn{
    pub tx_id: [u8; 32],
    pub vout: u32,
    pub sig: String,
    pub sequence:  u32,
}

pub struct TxOut {
    pub value: u64,
    scriptPubkey: Vec<u8>,
}

pub struct Tx {
    pub version: i64,
    pub inputs: Vec<TxIn>,
    pub outputs: Vec<TxOut>,
    pub locktime: u32,
}

