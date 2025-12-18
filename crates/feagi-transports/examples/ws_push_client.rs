// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WebSocket Push Client Example
//!
//! Demonstrates sending messages to a WebSocket pull server.
//! Used for sending sensory data to FEAGI.
//!
//! Run this with:
//! ```bash
//! cargo run --example ws_push_client --features websocket-client
//! ```

use feagi_transports::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let _ = tracing_subscriber::fmt::try_init();

    println!("🦀 Starting WebSocket Push Client");

    // Create push client
    let mut push = WsPush::with_address("ws://127.0.0.1:9051").await?;
    push.start_async().await?;

    println!("✅ Connected to pull server");
    println!("📤 Sending sensory data...");

    let mut counter = 0u64;

    loop {
        let data = format!("Sensory data frame {}", counter);

        push.push_async(data.as_bytes()).await?;

        println!("📤 Sent: {}", data);

        counter += 1;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
