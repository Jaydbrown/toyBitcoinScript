use std::sync::Arc;
use jsonrpsee::server::Server;
use toyBitcoinScript::p2p::Swarm;
use toyBitcoinScript::rpc::{
    MiningRpcServer, MiningRpcServerImpl,
    NodeRpcServer, NodeRpcServerImpl,
    RawTransactionsRpcServer, RawTransactionsRpcServerImpl,
    StateRpcServer, StateRpcServerImpl,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(r"
       ==============          ====          ==        ==
             ==               ==  ==          ==      == 
             ==              ========          ==    ==  
             ==             ==      ==          ======   
       ==    ==            ==        ==           ==     
        ======            ==          ==          ==     

     ====  ==  ======  ====   ====   ==  ==   ==
     == == ==    ==   ==     ==  ==  ==  ===  ==
     ====  ==    ==   ==     ==  ==  ==  == = ==
     == == ==    ==   ==     ==  ==  ==  ==  ===
     ====  ==    ==    ====   ====   ==  ==   ==

          ==   ==   ====   ====    =====
          ===  ==  ==  ==  == ==   ==   
          == = ==  ==  ==  ==  ==  ==== 
          ==  ===  ==  ==  == ==   ==   
          ==   ==   ====   ====    =====

             Welcome to Jay's Bitcoin Node
        A light representation of a Bitcoin node
");

    // Read optional CLI arguments: cargo run -- [rpc_port] [p2p_port] [optional_peer_addr]
    let args: Vec<String> = std::env::args().collect();
    let rpc_port = args.get(1).map(|s| s.as_str()).unwrap_or("9944");
    let p2p_port = args.get(2).map(|s| s.as_str()).unwrap_or("8001");
    let connect_peer = args.get(3).map(|s| s.as_str());

    let rpc_addr = format!("127.0.0.1:{}", rpc_port);
    let p2p_addr = format!("127.0.0.1:{}", p2p_port);

    // 1. Initialize P2P Swarm
    let swarm = Arc::new(Swarm::new());
    swarm.start_listener(&p2p_addr).await?;
    println!("[P2P] Listening for swarm peers on {}", p2p_addr);
    if let Some(peer) = connect_peer {
        println!("[P2P] Connecting to peer: {} ...", peer);
        if let Err(e) = swarm.connect_to_peer(peer).await {
            eprintln!("[P2P] Warning: Could not connect to initial peer {}: {}", peer, e);
        } else {
            println!("[P2P] Successfully connected to peer {}", peer);
        }
    }

    let server = Server::builder().build(&rpc_addr).await?;

    let mut rpc_module = StateRpcServerImpl.into_rpc();
    rpc_module.merge(RawTransactionsRpcServerImpl.into_rpc())?;
    let node_rpc = NodeRpcServerImpl::new(swarm.clone());
    rpc_module.merge(node_rpc.into_rpc())?;
    rpc_module.merge(MiningRpcServerImpl.into_rpc())?;
    let handle = server.start(rpc_module);
    println!("[RPC] JSON-RPC Server listening on http://{}", rpc_addr);
    println!("\nAvailable RPC Methods:");
    println!("  - eval_script");
    println!("  - decoderawtransaction");
    println!("  - createrawtransaction");
    println!("  - getblockchaininfo");
    println!("  - getnetworkinfo");
    println!("  - getmempoolinfo");
    println!("  - getblocktemplate");
    println!("  - submitblock");
    println!("  - generate\n");
    println!("Node is running! Press Ctrl+C to stop.\n");

    // Keep server running
    handle.stopped().await;
    Ok(())
}
