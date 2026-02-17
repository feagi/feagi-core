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
use std::thread;
use std::time::Duration;

use feagi_io::protocol_implementations::websocket::websocket_std::{
    FeagiWebSocketClientSubscriberProperties, FeagiWebSocketServerPublisherProperties,
};
use feagi_io::protocol_implementations::zmq::{
    FeagiZmqClientSubscriberProperties, FeagiZmqServerPublisherProperties,
};
use feagi_io::traits_and_enums::client::{FeagiClientSubscriber, FeagiClientSubscriberProperties};
use feagi_io::traits_and_enums::server::{FeagiServerPublisher, FeagiServerPublisherProperties};
use feagi_io::traits_and_enums::shared::FeagiEndpointState;

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
            let props = FeagiZmqServerPublisherProperties::new(ZMQ_ADDRESS, ZMQ_ADDRESS)
                .expect("Failed to create ZMQ publisher properties");
            props.as_boxed_server_publisher()
        }
        Transport::WebSocket => {
            println!("=== Publisher Example (WebSocket Transport) ===\n");
            println!("Binding to {}", WS_ADDRESS);
            let props = FeagiWebSocketServerPublisherProperties::new(WS_ADDRESS, WS_URL)
                .expect("Failed to create WebSocket publisher properties");
            props.as_boxed_server_publisher()
        }
    }
}

/// Creates the appropriate subscriber implementation based on transport type.
/// Returns a boxed trait object that can be used with transport-agnostic code.
fn create_subscriber(transport: Transport) -> Box<dyn FeagiClientSubscriber> {
    match transport {
        Transport::Zmq => {
            println!("=== Subscriber Example (ZMQ Transport) ===\n");
            println!("Connecting to {}", ZMQ_ADDRESS);
            let props = FeagiZmqClientSubscriberProperties::new(ZMQ_ADDRESS)
                .expect("Failed to create ZMQ subscriber properties");
            props.as_boxed_client_subscriber()
        }
        Transport::WebSocket => {
            println!("=== Subscriber Example (WebSocket Transport) ===\n");
            println!("Connecting to {}", WS_URL);
            let props = FeagiWebSocketClientSubscriberProperties::new(WS_URL)
                .expect("Failed to create WebSocket subscriber properties");
            props.as_boxed_client_subscriber()
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
fn run_publisher(mut publisher: Box<dyn FeagiServerPublisher>) {
    // Start the server (binds to address)
    publisher
        .request_start()
        .expect("Failed to start publisher");
    println!("Publisher start requested...");

    // Wait for the server to become active
    loop {
        match publisher.poll() {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                println!("Publisher started successfully!");
                break;
            }
            FeagiEndpointState::Pending => {
                thread::sleep(Duration::from_millis(10));
            }
            FeagiEndpointState::Errored(e) => {
                panic!("Publisher failed to start: {:?}", e);
            }
            FeagiEndpointState::Inactive => {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }

    println!("Waiting for subscribers to connect...\n");
    thread::sleep(Duration::from_millis(500));

    let mut counter = 0u64;
    loop {
        // Poll for maintenance tasks / new connections
        let _ = publisher.poll();

        // Create and publish message
        let message = format!("Message #{}: Hello from FEAGI!", counter);
        println!("[PUB] Publishing: {}", message);

        publisher
            .publish_data(message.as_bytes())
            .expect("Failed to publish");

        counter += 1;
        thread::sleep(Duration::from_millis(500));
    }
}

/// Runs the subscriber loop using any FeagiClientSubscriber implementation.
///
/// This function is completely transport-agnostic - it only uses the trait
/// interface and will work identically with ZMQ, WebSocket, or any future
/// transport implementation.
fn run_subscriber(mut subscriber: Box<dyn FeagiClientSubscriber>) {
    // Connect to the server
    subscriber
        .request_connect()
        .expect("Failed to request connection");
    println!("Subscriber connection requested...");

    // Wait for the connection to become active
    loop {
        match subscriber.poll() {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                println!("Subscriber connected successfully!");
                break;
            }
            FeagiEndpointState::Pending => {
                thread::sleep(Duration::from_millis(10));
            }
            FeagiEndpointState::Errored(e) => {
                panic!("Subscriber failed to connect: {:?}", e);
            }
            FeagiEndpointState::Inactive => {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }

    println!("Waiting for messages...\n");

    // Receive loop
    loop {
        match subscriber.poll() {
            FeagiEndpointState::ActiveHasData => match subscriber.consume_retrieved_data() {
                Ok(data) => {
                    let message = String::from_utf8_lossy(data);
                    println!("[SUB] Received: {}", message);
                }
                Err(e) => {
                    eprintln!("[SUB] Error consuming data: {}", e);
                }
            },
            FeagiEndpointState::ActiveWaiting => {
                // No data yet, keep polling
                thread::sleep(Duration::from_millis(10));
            }
            FeagiEndpointState::Errored(e) => {
                eprintln!("[SUB] Error: {:?}", e);
                break;
            }
            _ => {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

fn main() {
    match parse_args() {
        Some((transport, mode)) => match mode.as_str() {
            "publisher" => {
                let publisher = create_publisher(transport);
                run_publisher(publisher);
            }
            "subscriber" => {
                let subscriber = create_subscriber(transport);
                run_subscriber(subscriber);
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
