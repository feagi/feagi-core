//! Example: Subscriber (SUB)
//!
//! This example demonstrates receiving broadcast messages from a publisher.
//!
//! Make sure the publisher is running first, then:
//! ```
//! cargo run --example subscriber --features=zmq-client
//! ```

use feagi_transports::prelude::*;
use std::env;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get topic filter from command line (default: all)
    let args: Vec<String> = env::args().collect();
    let topic = args.get(1).map(|s| s.as_str()).unwrap_or("");

    println!("🦀 Connecting to publisher at tcp://localhost:5556");
    println!("🔍 Subscribing to topic: '{}'", topic);

    // Create subscriber
    let context = Arc::new(zmq::Context::new());
    let config = ClientConfig::new("tcp://localhost:5556");
    let mut subscriber = ZmqSub::new(context, config)?;
    subscriber.start()?;

    // Subscribe to topic (empty = all messages)
    subscriber.subscribe(topic.as_bytes())?;

    println!("✅ Connected! Listening for messages...\n");

    // Receive messages
    loop {
        let (topic_bytes, data) = subscriber.receive()?;
        let topic_str = String::from_utf8_lossy(&topic_bytes);
        let message = String::from_utf8_lossy(&data);
        println!("📨 [{}]: {}", topic_str, message);
    }
}


