use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use serde::{Deserialize, Serialize};

use super::state::{decode, encode};
use crate::transaction::{Tx, TxIn, TxOut};

pub fn decode_raw_transactions(hex: &str) -> Result<Tx, &'static str> {
    let bytes = decode(hex)?;
    let mut cursor: usize = 0;
    if cursor + 4 > bytes.len() {
        return Err("Unexpected end of data reading version");
    }
    let version = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().unwrap());
    cursor += 4;

    if cursor >= bytes.len() {
        return Err("Unexpected end of data reading input count");
    }
    let in_count = bytes[cursor] as usize;
    cursor += 1;

    let mut inputs = Vec::new();
    for _ in 0..in_count {
        if cursor + 32 > bytes.len() {
            return Err("Unexpected end of data reading tx_id");
        }
        let tx_id: [u8; 32] = bytes[cursor..cursor + 32].try_into().unwrap();
        cursor += 32;

        if cursor + 4 > bytes.len() {
            return Err("Unexpected end of data reading vout");
        }
        let vout = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().unwrap());
        cursor += 4;

        if cursor >= bytes.len() {
            return Err("Unexpected end of data reading scriptSig length");
        }
        let sig_len = bytes[cursor] as usize;
        cursor += 1;

        if cursor + sig_len > bytes.len() {
            return Err("Unexpected end of data reading scriptSig bytes");
        }
        let sig_bytes = &bytes[cursor..cursor + sig_len];
        let sig = encode(sig_bytes);
        cursor += sig_len;

        if cursor + 4 > bytes.len() {
            return Err("Unexpected end of data reading sequence");
        }
        let sequence = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().unwrap());
        cursor += 4;

        inputs.push(TxIn {
            tx_id,
            vout,
            sig,
            sequence,
        });
    }

    if cursor >= bytes.len() {
        return Err("Unexpected end of data reading output count");
    }
    let out_count = bytes[cursor] as usize;
    cursor += 1;

    let mut outputs = Vec::new();
    for _ in 0..out_count {
        if cursor + 8 > bytes.len() {
            return Err("Unexpected end of data reading output value");
        }
        let value = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap());
        cursor += 8;

        if cursor >= bytes.len() {
            return Err("Unexpected end of data reading scriptPubkey length");
        }
        let script_len = bytes[cursor] as usize;
        cursor += 1;

        if cursor + script_len > bytes.len() {
            return Err("Unexpected end of data reading scriptPubkey bytes");
        }
        let scriptPubkey = bytes[cursor..cursor + script_len].to_vec();
        cursor += script_len;

        outputs.push(TxOut {
            value,
            scriptPubkey,
        });
    }

    if cursor + 4 > bytes.len() {
        return Err("Unexpected end of data reading locktime");
    }
    let locktime = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().unwrap());
    cursor += 4;

    Ok(Tx {
        version,
        inputs,
        outputs,
        locktime,
    })
}

pub fn encode_raw_transaction(tx: &Tx) -> String {
    let mut bytes = Vec::new();

    // 1. Version (4 bytes)
    bytes.extend_from_slice(&tx.version.to_le_bytes());

    // 2. Inputs
    bytes.push(tx.inputs.len() as u8);
    for input in &tx.inputs {
        bytes.extend_from_slice(&input.tx_id);
        bytes.extend_from_slice(&input.vout.to_le_bytes());
        let sig_bytes = decode(&input.sig).unwrap_or_default();
        bytes.push(sig_bytes.len() as u8);
        bytes.extend_from_slice(&sig_bytes);
        bytes.extend_from_slice(&input.sequence.to_le_bytes());
    }

    // 3. Outputs
    bytes.push(tx.outputs.len() as u8);
    for output in &tx.outputs {
        bytes.extend_from_slice(&output.value.to_le_bytes());
        bytes.push(output.scriptPubkey.len() as u8);
        bytes.extend_from_slice(&output.scriptPubkey);
    }

    // 4. Locktime (4 bytes)
    bytes.extend_from_slice(&tx.locktime.to_le_bytes());

    encode(&bytes)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateTxInput {
    pub tx_id_hex: String,
    pub vout: u32,
    pub sequence: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateTxOutput {
    pub value: u64,
    pub script_hex: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateRawTxParams {
    pub inputs: Vec<CreateTxInput>,
    pub outputs: Vec<CreateTxOutput>,
    pub locktime: Option<u32>,
}

pub fn create_raw_transaction(params: CreateRawTxParams) -> Result<String, &'static str> {
    let mut inputs = Vec::new();
    for inp in params.inputs {
        let tx_id_bytes = decode(&inp.tx_id_hex)?;
        if tx_id_bytes.len() != 32 {
            return Err("tx_id must be exactly 32 bytes (64 hex characters)");
        }
        let mut tx_id = [0u8; 32];
        tx_id.copy_from_slice(&tx_id_bytes);

        inputs.push(TxIn {
            tx_id,
            vout: inp.vout,
            sig: String::new(),
            sequence: inp.sequence.unwrap_or(0xffffffff),
        });
    }

    let mut outputs = Vec::new();
    for out in params.outputs {
        let scriptPubkey = decode(&out.script_hex)?;
        outputs.push(TxOut {
            value: out.value,
            scriptPubkey,
        });
    }

    let tx = Tx {
        version: 1,
        inputs,
        outputs,
        locktime: params.locktime.unwrap_or(0),
    };

    Ok(encode_raw_transaction(&tx))
}

#[rpc(server)]
pub trait RawTransactionsRpc {
    #[method(name = "decoderawtransaction")]
    async fn decode_raw_transaction(&self, hex: String) -> RpcResult<Tx>;

    #[method(name = "createrawtransaction")]
    async fn create_raw_transaction(&self, params: CreateRawTxParams) -> RpcResult<String>;
}

pub struct RawTransactionsRpcServerImpl;

#[jsonrpsee::core::async_trait]
impl RawTransactionsRpcServer for RawTransactionsRpcServerImpl {
    async fn decode_raw_transaction(&self, hex: String) -> RpcResult<Tx> {
        decode_raw_transactions(&hex).map_err(|err| {
            jsonrpsee::types::ErrorObject::owned(
                -32602,
                err.to_string(),
                None::<()>,
            )
        })
    }

    async fn create_raw_transaction(&self, params: CreateRawTxParams) -> RpcResult<String> {
        create_raw_transaction(params).map_err(|err| {
            jsonrpsee::types::ErrorObject::owned(
                -32602,
                err.to_string(),
                None::<()>,
            )
        })
    }
}