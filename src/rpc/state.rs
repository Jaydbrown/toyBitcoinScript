use std::fmt::Write;
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use serde::{Deserialize, Serialize};

use crate::{is_true, ToyScript};

pub fn decode(s: &str) -> Result<Vec<u8>, &'static str> {
    if s.len() % 2 != 0 {
        return Err("you have an unequal number of hexadecimal string");
    }

    let mut result = Vec::new();

    for chunk in s.as_bytes().chunks_exact(2) {
        let c1 = chunk[0];
        let c2 = chunk[1];

        let val1 = hex_val(c1)?;
        let val2 = hex_val(c2)?;
        let byte = (val1 * 16) + val2;

        result.push(byte);
    }

    Ok(result)
}

pub fn encode(b: &[u8]) -> String {
    let mut s = String::with_capacity(b.len() * 2);
    for &byte in b {
        let _ = write!(s, "{:02x}", byte);
    }
    s
}

fn hex_val(byte: u8) -> Result<u8, &'static str> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err("invalid hex character"),
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct eval_script_action {
    pub script_hex: Option<String>,
    pub unlock_hex: Option<String>,
    pub lock_hex: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalScriptResult {
    pub success: bool,
    pub final_stack: Vec<String>,
    pub error: Option<String>,
}

pub fn evaluate_script(action: eval_script_action) -> Result<EvalScriptResult, &'static str> {
    let bytecode = match (&action.script_hex, &action.unlock_hex, &action.lock_hex) {
        (Some(script), None, None) => decode(script)?,
        (None, Some(unlock), Some(lock)) => {
            let mut bytes = decode(unlock)?;
            let lock_bytes = decode(lock)?;
            bytes.extend(lock_bytes);
            bytes
        }
        _ => return Err("Provide either 'script_hex' or both 'unlock_hex' and 'lock_hex'"),
    };

    let script = ToyScript::new(bytecode);

    match script.run() {
        Ok(stack) => {
            let success = match stack.top() {
                Ok(top_bytes) => is_true(top_bytes),
                Err(_) => false,
            };

            let final_stack: Vec<String> = stack.stack.iter().map(|item| encode(item)).collect();

            Ok(EvalScriptResult {
                success,
                final_stack,
                error: None,
            })
        }
        Err(err_msg) => Ok(EvalScriptResult {
            success: false,
            final_stack: Vec::new(),
            error: Some(err_msg.to_string()),
        }),
    }
}

#[rpc(server)]
pub trait StateRpc {
    #[method(name = "eval_script")]
    async fn eval_script(&self, action: eval_script_action) -> RpcResult<EvalScriptResult>;
}

pub struct StateRpcServerImpl;

#[jsonrpsee::core::async_trait]
impl StateRpcServer for StateRpcServerImpl {
    async fn eval_script(&self, action: eval_script_action) -> RpcResult<EvalScriptResult> {
        evaluate_script(action).map_err(|err| {
            jsonrpsee::types::ErrorObject::owned(
                -32602,
                err.to_string(),
                None::<()>,
            )
        })
    }
}
