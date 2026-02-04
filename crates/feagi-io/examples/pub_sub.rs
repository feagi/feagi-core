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

use feagi_config::{load_config, FeagiConfig};
use feagi_io::core::protocol_implementations::websocket::{
    FeagiWebSocketClientSubscriberProperties, FeagiWebSocketServerPublisherProperties,
};
use feagi_io::core::protocol_implementations::zmq::{
    FeagiZmqClientSubscriberProperties, FeagiZmqServerPublisherProperties,
};
use feagi_io::core::traits_and_enums::client::{FeagiClientSubscriber, FeagiClientSubscriberProperties};
use feagi_io::core::traits_and_enums::server::{FeagiServerPublisher, FeagiServerPublisherProperties};
use feagi_io::core::traits_and_enums::FeagiEndpointState;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Transport {
    Zmq,
    WebSocket,
}

#[derive(Debug, Clone)]
struct ExampleEndpoints {
    zmq_address: String,
    ws_address: String,
    ws_url: String,
}

/// Load the FEAGI configuration using the standard loader.
fn load_feagi_config() -> FeagiConfig {
    load_config(None, None).expect("Failed to load FEAGI configuration")
}

fn format_tcp_endpoint(host: &str, port: u16) -> String {
    if host.contains(':') {
        format!("tcp://[{host}]:{port}")
    } else {
        format!("tcp://{host}:{port}")
    }
}

fn format_ws_address(host: &str, port: u16) -> String {
    if host.contains(':') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    }
}

fn format_ws_url(host: &str, port: u16) -> String {
    if host.contains(':') {
        format!("ws://[{host}]:{port}")
    } else {
        format!("ws://{host}:{port}")
    }
}

fn build_endpoints() -> ExampleEndpoints {
    let config = load_feagi_config();
    ExampleEndpoints {
        zmq_address: format_tcp_endpoint(&config.zmq.host, config.ports.zmq_pub_sub_port),
        ws_address: format_ws_address(&config.websocket.host, config.websocket.visualization_port),
        ws_url: format_ws_url(&config.websocket.host, config.websocket.visualization_port),
    }
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
fn create_publisher(transport: Transport, endpoints: &ExampleEndpoints) -> Box<dyn FeagiServerPublisher> {
    match transport {
        Transport::Zmq => {
            println!("=== Publisher Example (ZMQ Transport) ===\n");
            println!("Binding to {}", endpoints.zmq_address);
            let props = FeagiZmqServerPublisherProperties::new(&endpoints.zmq_address)
                .expect("Failed to create ZMQ publisher properties");
            props.as_boxed_server_publisher()
        }
        Transport::WebSocket => {
            println!("=== Publisher Example (WebSocket Transport) ===\n");
            println!("Binding to {}", endpoints.ws_address);
            let props = FeagiWebSocketServerPublisherProperties::new(&endpoints.ws_address)
                .expect("Failed to create WebSocket publisher properties");
            props.as_boxed_server_publisher()
        }
    }
}

/// Creates the appropriate subscriber implementation based on transport type.
/// Returns a boxed trait object that can be used with transport-agnostic code.
fn create_subscriber(transport: Transport, endpoints: &ExampleEndpoints) -> Box<dyn FeagiClientSubscriber> {
    match transport {
        Transport::Zmq => {
            println!("=== Subscriber Example (ZMQ Transport) ===\n");
            println!("Connecting to {}", endpoints.zmq_address);
            let props = FeagiZmqClientSubscriberProperties::new(&endpoints.zmq_address)
                .expect("Failed to create ZMQ subscriber properties");
            props.as_boxed_client_subscriber()
        }
        Transport::WebSocket => {
            println!("=== Subscriber Example (WebSocket Transport) ===\n");
            println!("Connecting to {}", endpoints.ws_url);
            let props = FeagiWebSocketClientSubscriberProperties::new(&endpoints.ws_url)
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
    publisher.request_start().expect("Failed to start publisher");
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
            FeagiEndpointState::ActiveHasData => {
                match subscriber.consume_retrieved_data() {
                    Ok(data) => {
                        let message = String::from_utf8_lossy(data);
                        println!("[SUB] Received: {}", message);
                    }
                    Err(e) => {
                        eprintln!("[SUB] Error consuming data: {}", e);
                    }
                }
            }
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
    let endpoints = build_endpoints();
    match parse_args() {
        Some((transport, mode)) => match mode.as_str() {
            "publisher" => {
                let publisher = create_publisher(transport, &endpoints);
                run_publisher(publisher);
            }
            "subscriber" => {
                let subscriber = create_subscriber(transport, &endpoints);
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
