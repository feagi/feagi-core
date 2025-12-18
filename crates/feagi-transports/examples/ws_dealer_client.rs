// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WebSocket Dealer Client Example
//!
//! Demonstrates request-reply from client side.
//! Used for agent control plane communication.
//!
//! Run this with:
//! ```bash
//! cargo run --example ws_dealer_client --features websocket-client
//! ```

use feagi_transports::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let _ = tracing_subscriber::fmt::try_init();

    println!("🦀 Starting WebSocket Dealer Client");

    // Create dealer client
    let mut dealer = WsDealer::with_address("ws://127.0.0.1:9053").await?;
    dealer.start_async().await?;

    println!("✅ Connected to router server");
    println!("📤 Sending requests...");

    for i in 0..10 {
        let request = format!("Request #{}", i);
        println!("📤 Sending: {}", request);

        match dealer.request_async(request.as_bytes()).await {
            Ok(response) => {
                println!("📥 Response: {}", String::from_utf8_lossy(&response));
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);
                break;
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    println!("✅ Done sending requests");

    Ok(())
}
