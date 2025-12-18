// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WebSocket Router Server Example
//!
//! Demonstrates request-reply pattern with routing.
//! Used for per-agent control channels in FEAGI.
//!
//! Run this with:
//! ```bash
//! cargo run --example ws_router_server --features websocket-server
//! ```

use feagi_transports::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let _ = tracing_subscriber::fmt::try_init();

    println!("🦀 Starting WebSocket Router Server on 127.0.0.1:9053");

    // Create router
    let mut router = WsRouter::with_address("127.0.0.1:9053").await?;
    router.start_async().await?;

    println!("✅ Router server started. Handling requests...");
    println!("📡 Clients can connect to: ws://127.0.0.1:9053");

    let mut count = 0u64;

    loop {
        match router.receive_timeout(1000) {
            Ok((request, reply_handle)) => {
                count += 1;
                let request_str = String::from_utf8_lossy(&request);
                println!("📥 [{}] Request: {}", count, request_str);

                // Process request and send reply
                let response = format!("OK: Processed '{}'", request_str);
                reply_handle.send(response.as_bytes())?;

                println!("📤 [{}] Reply sent", count);
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
