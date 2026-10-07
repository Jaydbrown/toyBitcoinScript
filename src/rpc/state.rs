use jsonrpsee::proc_macros::rpc;
use jsonrpsee::core::RpcResult;
use jsonrpsee::server::Server;
use std::fmt::Write;

pub fn decode(s: &str) -> Result<Vec<u8>,  &'static str>{
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

pub fn encode(b: &[u8]) -> String{
    if b.is_empty() {
        return
    }

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

pub struct eval_script_action{
    pub script_hex: Option<String>,
    pub unlock_hex: Option<String>,
    pub lock_hex: Option<String>,
}
