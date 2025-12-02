// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WebSocket Publisher Example
//!
//! Demonstrates broadcasting messages to WebSocket subscribers.
//! Used for visualization and motor command streams.
//!
//! Run this with:
//! ```bash
//! cargo run --example ws_publisher --features websocket-server
//! ```

use feagi_transports::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let _ = tracing_subscriber::fmt::try_init();
    
    println!("🦀 Starting WebSocket Publisher on 127.0.0.1:9050");
    
    // Create publisher
    let mut publisher = WsPub::with_address("127.0.0.1:9050").await?;
    publisher.start_async().await?;
    
    println!("✅ Publisher started. Broadcasting messages...");
    println!("📡 Connect subscribers to: ws://127.0.0.1:9050");
    
    let mut counter = 0u64;
    
    loop {
        // Publish with topic
        let topic = b"visualization";
        let data = format!("Frame {}", counter);
        publisher.publish(topic, data.as_bytes())?;
        
        println!("📤 Published: {} -> {}", 
                 String::from_utf8_lossy(topic), data);
        
        counter += 1;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

