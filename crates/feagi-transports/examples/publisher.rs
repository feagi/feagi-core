//! Example: Publisher (PUB)
//!
//! This example demonstrates broadcasting messages to multiple subscribers.
//!
//! ```
//! cargo run --example publisher --features=zmq-server
//! ```

use feagi_transports::prelude::*;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Starting PUB server on tcp://*:5556");

    // Create publisher
    let context = Arc::new(zmq::Context::new());
    let config = ServerConfig::new("tcp://*:5556");
    let mut publisher = ZmqPub::new(context, config)?;
    publisher.start()?;

    println!("✅ Publisher ready, broadcasting messages...\n");

    // Publish messages
    let mut counter = 0;
    loop {
        // Generate message for different topics
        let topics = vec!["sensor", "motor", "brain"];

        for topic in topics {
            let message = format!("{}:{} - data at {}", topic, counter, chrono::Utc::now());
            publisher.publish(topic.as_bytes(), message.as_bytes())?;
            println!("📤 Published [{}]: {}", topic, message);
        }

        counter += 1;
        thread::sleep(Duration::from_secs(1));
    }
}

