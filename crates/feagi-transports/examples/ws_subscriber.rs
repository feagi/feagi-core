// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WebSocket Subscriber Example
//!
//! Demonstrates receiving messages from a WebSocket publisher.
//! Used for receiving motor commands and visualization data.
//!
//! Run this with:
//! ```bash
//! cargo run --example ws_subscriber --features websocket-client
//! ```

use feagi_transports::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let _ = tracing_subscriber::fmt::try_init();

    println!("🦀 Starting WebSocket Subscriber");

    // Create subscriber
    let mut subscriber = WsSub::with_address("ws://127.0.0.1:9050").await?;
    subscriber.start_async().await?;

    // Subscribe to topics
    subscriber.subscribe(b"visualization")?;
    println!("✅ Subscribed to: visualization");
    println!("📡 Waiting for messages...");

    loop {
        match subscriber.receive_timeout(1000) {
            Ok((topic, data)) => {
                println!(
                    "📥 Received: {} -> {}",
                    String::from_utf8_lossy(&topic),
                    String::from_utf8_lossy(&data)
                );
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

    Ok(())
}
