//! WebSocket Pull Server Example
//!
//! Demonstrates receiving messages from multiple WebSocket push clients.
//! Used for sensory data input in FEAGI.
//!
//! Run this with:
//! ```bash
//! cargo run --example ws_pull_server --features websocket-server
//! ```

use feagi_transports::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let _ = tracing_subscriber::fmt::try_init();
    
    println!("🦀 Starting WebSocket Pull Server on 127.0.0.1:9051");
    
    // Create pull server
    let mut pull = WsPull::with_address("127.0.0.1:9051").await?;
    pull.start_async().await?;
    
    println!("✅ Pull server started. Waiting for messages...");
    println!("📡 Clients can connect to: ws://127.0.0.1:9051");
    
    let mut count = 0u64;
    
    loop {
        match pull.pull_timeout(1000) {
            Ok(data) => {
                count += 1;
                println!("📥 [{}] Received {} bytes: {}", 
                         count,
                         data.len(),
                         String::from_utf8_lossy(&data));
            }
            Err(TransportError::Timeout) => {
                // Timeout is normal, just continue
                continue;
            }
            Err(e) => {
                eprintln!("❌ Error receiving: {}", e);
                break;
            }
        }
    }
}

