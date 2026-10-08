use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc;

use super::message::P2pMessage;

/// Continuously reads newline-delimited JSON messages from the incoming TCP half
pub async fn read_peer_loop(
    reader: OwnedReadHalf,
    incoming_tx: mpsc::Sender<P2pMessage>,
) -> Result<(), &'static str> {
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        match buf_reader.read_line(&mut line).await {
            Ok(0) => {
                // Connection closed by peer
                return Ok(());
            }
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if let Ok(msg) = serde_json::from_str::<P2pMessage>(trimmed) {
                    if incoming_tx.send(msg).await.is_err() {
                        // Swarm receiver dropped
                        return Ok(());
                    }
                }
            }
            Err(_) => return Err("Error reading from peer socket"),
        }
    }
}

/// Continuously writes outgoing P2pMessages from an mpsc channel to the TCP half
pub async fn write_peer_loop(
    mut writer: OwnedWriteHalf,
    mut outgoing_rx: mpsc::Receiver<P2pMessage>,
) -> Result<(), &'static str> {
    while let Some(msg) = outgoing_rx.recv().await {
        if let Ok(json) = serde_json::to_string(&msg) {
            let data = format!("{}\n", json);
            if writer.write_all(data.as_bytes()).await.is_err() {
                return Err("Failed to write to peer socket");
            }
            let _ = writer.flush().await;
        }
    }
    Ok(())
}

