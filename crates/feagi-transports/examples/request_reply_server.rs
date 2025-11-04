//! Example: Request-Reply Server (ROUTER)
//!
//! This example demonstrates how to create a simple request-reply server
//! using the ZMQ ROUTER pattern.
//!
//! Run this server first, then run the client in another terminal:
//! ```
//! cargo run --example request_reply_server --features=zmq-server
//! ```

use feagi_transports::prelude::*;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Starting ROUTER server on tcp://*:5555");

    // Create server
    let context = Arc::new(zmq::Context::new());
    let config = ServerConfig::new("tcp://*:5555");
    let mut server = ZmqRouter::new(context, config)?;
    server.start()?;

    println!("✅ Server ready, waiting for requests...\n");

    // Handle requests
    loop {
        // Receive request
        let (request, reply_handle) = server.receive()?;

        // Parse request
        let request_str = String::from_utf8_lossy(&request);
        println!("📨 Received: {}", request_str);

        // Process request
        let response = match request_str.as_ref() {
            "ping" => "pong".to_string(),
            "hello" => "world".to_string(),
            "quit" => {
                reply_handle.send(b"goodbye")?;
                println!("👋 Shutting down...");
                break;
            }
            _ => format!("echo: {}", request_str),
        };

        // Send reply
        reply_handle.send(response.as_bytes())?;
        println!("📤 Sent: {}\n", response);
    }

    server.stop()?;
    println!("✅ Server stopped");

    Ok(())
}




