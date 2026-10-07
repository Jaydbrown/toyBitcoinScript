use jsonrpsee::proc_macros::rpc;
use jsonrpsee::core::RpcResult;
use jsonrpsee::server::Server;

pub struct eval_script_action{
    pub script_hex: Option<String>,
    pub unlock_hex: Option<String>,
    pub lock_hex: Option<String>,
}

pub fn decode(&self, string: &str) -> Result<Vec<u8>,  &'static str>{
    if let some(hex_str) != 0 {
        return Err("you have an unequal number of hexadecimal string");
    }

    
}