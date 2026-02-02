//! Publisher-Subscriber Example
//!
//! This example demonstrates the publish-subscribe pattern using trait objects,
//! allowing the same business logic to work with any transport implementation
//! (ZMQ or WebSocket), selected via command-line flag.
//!
//! The key demonstration here is that `run_publisher` and `run_subscriber` are
//! **transport-agnostic** - they only know about the trait interface, not the
//! concrete implementation. The transport is selected at runtime via the flag.
//!
//! # Usage
//!
//! Terminal 1 (Publisher/Server):
//! ```sh
//! cargo run --example pub_sub --features "zmq-transport ws-transport" -- --transport zmq publisher
//! cargo run --example pub_sub --features "zmq-transport ws-transport" -- --transport ws publisher
//! ```
//!
//! Terminal 2 (Subscriber/Client):
//! ```sh
//! cargo run --example pub_sub --features "zmq-transport ws-transport" -- --transport zmq subscriber
//! cargo run --example pub_sub --features "zmq-transport ws-transport" -- --transport ws subscriber
//! ```

use std::env;
use std::time::Duration;

use feagi_io::implementations::websocket::{
    FEAGIWebSocketClientSubscriber, FEAGIWebSocketServerPublisher,
};
use feagi_io::implementations::zmq::{FEAGIZMQClientSubscriber, FeagiZmqServerPublisher};
use feagi_io::traits_and_enums::client::FeagiClientSubscriber;
use feagi_io::traits_and_enums::server::FeagiServerPublisher;

const ZMQ_ADDRESS: &str = "tcp://127.0.0.1:5555";
const WS_ADDRESS: &str = "127.0.0.1:8080";
const WS_URL: &str = "ws://127.0.0.1:8080";

#[derive(Debug, Clone, Copy, PartialEq)]
enum Transport {
    Zmq,
    WebSocket,
}

fn parse_args() -> Option<(Transport, String)> {
    let args: Vec<String> = env::args().collect();

    let mut transport = None;
    let mut mode = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--transport" | "-t" => {
                if i + 1 < args.len() {
                    transport = match args[i + 1].to_lowercase().as_str() {
                        "zmq" => Some(Transport::Zmq),
                        "ws" | "websocket" => Some(Transport::WebSocket),
                        _ => None,
                    };
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "publisher" | "pub" | "p" => {
                mode = Some("publisher".to_string());
                i += 1;
            }
            "subscriber" | "sub" | "s" => {
                mode = Some("subscriber".to_string());
                i += 1;
            }
            _ => i += 1,
        }
    }

    match (transport, mode) {
        (Some(t), Some(m)) => Some((t, m)),
        _ => None,
    }
}

/// Creates the appropriate publisher implementation based on transport type.
/// Returns a boxed trait object that can be used with transport-agnostic code.
fn create_publisher(transport: Transport) -> Box<dyn FeagiServerPublisher> {
    match transport {
        Transport::Zmq => {
            println!("=== Publisher Example (ZMQ Transport) ===\n");
            println!("Binding to {}", ZMQ_ADDRESS);
            Box::new(
                FeagiZmqServerPublisher::new(
                    ZMQ_ADDRESS.to_string(),
                    Box::new(|state| println!("[PUB] State changed: {:?}", state)),
                )
                .expect("Failed to create ZMQ publisher"),
            )
        }
        Transport::WebSocket => {
            println!("=== Publisher Example (WebSocket Transport) ===\n");
            println!("Binding to {}", WS_ADDRESS);
            Box::new(
                FEAGIWebSocketServerPublisher::new(
                    WS_ADDRESS.to_string(),
                    Box::new(|state| println!("[PUB] State changed: {:?}", state)),
                )
                .expect("Failed to create WebSocket publisher"),
            )
        }
    }
}

/// Creates the appropriate subscriber implementation based on transport type.
/// Returns a boxed trait object that can be used with transport-agnostic code.
fn create_subscriber(transport: Transport) -> (Box<dyn FeagiClientSubscriber>, &'static str) {
    match transport {
        Transport::Zmq => {
            println!("=== Subscriber Example (ZMQ Transport) ===\n");
            println!("Connecting to {}", ZMQ_ADDRESS);
            (
                Box::new(
                    FEAGIZMQClientSubscriber::new(
                        ZMQ_ADDRESS.to_string(),
                        Box::new(|state| println!("[SUB] State changed: {:?}", state)),
                    )
                    .expect("Failed to create ZMQ subscriber"),
                ),
                ZMQ_ADDRESS,
            )
        }
        Transport::WebSocket => {
            println!("=== Subscriber Example (WebSocket Transport) ===\n");
            println!("Connecting to {}", WS_URL);
            (
                Box::new(
                    FEAGIWebSocketClientSubscriber::new(
                        WS_URL.to_string(),
                        Box::new(|state| println!("[SUB] State changed: {:?}", state)),
                    )
                    .expect("Failed to create WebSocket subscriber"),
                ),
                WS_URL,
            )
        }
    }
}

// ============================================================================
// TRANSPORT-AGNOSTIC BUSINESS LOGIC
// ============================================================================
// The functions below work with ANY implementation of the traits.
// They don't know or care whether they're using ZMQ, WebSocket, or any future
// transport - they only interact through the trait interface.
// ============================================================================

/// Runs the publisher loop using any FeagiServerPublisher implementation.
///
/// This function is completely transport-agnostic - it only uses the trait
/// interface and will work identically with ZMQ, WebSocket, or any future
/// transport implementation.
async fn run_publisher(mut publisher: Box<dyn FeagiServerPublisher>) {
    // Start the server (binds to address)
    publisher.start().await.expect("Failed to start publisher");
    println!("Publisher started successfully!");
    println!("Waiting for subscribers to connect...\n");

    // Brief warm-up period for connection establishment
    tokio::time::sleep(Duration::from_millis(500)).await;

    let mut counter = 0u64;
    loop {
        // Poll for new connections / maintenance tasks
        publisher.poll().await.expect("Failed to poll");

        // Create and publish message
        let message = format!("Message #{}: Hello from FEAGI!", counter);
        println!("[PUB] Publishing: {}", message);

        publisher
            .publish(message.as_bytes())
            .await
            .expect("Failed to publish");

        counter += 1;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

/// Runs the subscriber loop using any FeagiClientSubscriber implementation.
///
/// This function is completely transport-agnostic - it only uses the trait
/// interface and will work identically with ZMQ, WebSocket, or any future
/// transport implementation.
async fn run_subscriber(mut subscriber: Box<dyn FeagiClientSubscriber>, address: &str) {
    // Connect to the server
    subscriber
        .connect(address)
        .await
        .expect("Failed to connect");
    println!("Subscriber connected. Waiting for messages...\n");

    // Receive loop
    loop {
        match subscriber.get_subscribed_data().await {
            Ok(data) => {
                let message = String::from_utf8_lossy(&data);
                println!("[SUB] Received: {}", message);
            }
            Err(e) => {
                eprintln!("[SUB] Error receiving: {}", e);
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    match parse_args() {
        Some((transport, mode)) => match mode.as_str() {
            "publisher" => {
                let publisher = create_publisher(transport);
                run_publisher(publisher).await;
            }
            "subscriber" => {
                let (subscriber, address) = create_subscriber(transport);
                run_subscriber(subscriber, address).await;
            }
            _ => unreachable!(),
        },
        None => {
            let prog = env::args().next().unwrap_or_default();
            println!("Publisher-Subscriber Example");
            println!("Demonstrates transport-agnostic code using trait objects\n");
            println!("Usage:");
            println!("  {} --transport <zmq|ws> <publisher|subscriber>\n", prog);
            println!("Examples:");
            println!("  {} --transport zmq publisher", prog);
            println!("  {} --transport zmq subscriber", prog);
            println!("  {} --transport ws publisher", prog);
            println!("  {} --transport ws subscriber", prog);
            println!();
            println!("Run the publisher first, then the subscriber in another terminal.");
        }
    }
}
