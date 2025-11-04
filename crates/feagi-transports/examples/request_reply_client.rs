//! Example: Request-Reply Client (DEALER)
//!
//! This example demonstrates how to create a simple request-reply client
//! using the ZMQ DEALER pattern.
//!
//! Make sure the server is running first, then:
//! ```
//! cargo run --example request_reply_client --features=zmq-client
//! ```

use feagi_transports::prelude::*;
use std::io::{self, Write};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Connecting to server at tcp://localhost:5555");

    // Create client
    let context = Arc::new(zmq::Context::new());
    let config = ClientConfig::new("tcp://localhost:5555");
    let mut client = ZmqDealer::new(context, config)?;
    client.start()?;

    println!("✅ Connected! Type messages to send (or 'quit' to exit):\n");

    // Interactive loop
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        // Read user input
        print!("> ");
        io::stdout().flush()?;
        input.clear();
        stdin.read_line(&mut input)?;
        let message = input.trim();

        if message.is_empty() {
            continue;
        }

        // Send request
        println!("📤 Sending: {}", message);
        let response = client.request_timeout(message.as_bytes(), 5000)?;

        // Print response
        let response_str = String::from_utf8_lossy(&response);
        println!("📨 Received: {}\n", response_str);

        if message == "quit" {
            break;
        }
    }

    client.stop()?;
    println!("✅ Client disconnected");

    Ok(())
}




