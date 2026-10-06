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
    pub version: i32,
    pub inputs: Vec<TxIn>,
    pub outputs: Vec<TxOut>,
    pub locktime: u32,
}

impl Tx {
    pub fn new(version: i32, locktime: i32) -> Self{
        Self {
            version,
            inputs: Vec::new(),
            outputs: Vec::new(),
            locktime,
        }
    }

    pub fn tx_in(tx: TxIn) {
        self.inputs.push(tx)
    }

    pub fn tx_out(tx: TxOut) {
        self.outputs.push(tx)
    }
}
