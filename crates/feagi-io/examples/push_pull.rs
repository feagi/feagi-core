//! Push-Pull Example
//!
//! This example demonstrates the push-pull pattern using trait objects,
//! allowing the same business logic to work with any transport implementation
//! (ZMQ or WebSocket), selected via command-line flag.
//!
//! In this pattern:
//! - The **server** binds and waits to receive (pull) data
//! - The **client** connects and sends (pushes) data to the server
//!
//! The key demonstration here is that `run_server` and `run_client` are
//! **transport-agnostic** - they only know about the trait interface.
//!
//! # Usage
//!
//! Terminal 1 (Server/Puller - start first):
//! ```sh
//! cargo run --example push_pull --features "zmq-transport ws-transport" -- --transport zmq server
//! cargo run --example push_pull --features "zmq-transport ws-transport" -- --transport ws server
//! ```
//!
//! Terminal 2 (Client/Pusher):
//! ```sh
//! cargo run --example push_pull --features "zmq-transport ws-transport" -- --transport zmq client
//! cargo run --example push_pull --features "zmq-transport ws-transport" -- --transport ws client
//! ```

use std::env;
use std::thread;
use std::time::Duration;

use feagi_io::core::protocol_implementations::websocket::{
    FeagiWebSocketClientPusherProperties, FeagiWebSocketServerPullerProperties,
};
use feagi_io::core::protocol_implementations::zmq::{
    FeagiZmqClientPusherProperties, FeagiZmqServerPullerProperties,
};
use feagi_io::core::traits_and_enums::client::{FeagiClientPusher, FeagiClientPusherProperties};
use feagi_io::core::traits_and_enums::server::{FeagiServerPuller, FeagiServerPullerProperties};
use feagi_io::core::traits_and_enums::FeagiEndpointState;

const ZMQ_ADDRESS: &str = "tcp://127.0.0.1:5556";
const WS_ADDRESS: &str = "127.0.0.1:8081";
const WS_URL: &str = "ws://127.0.0.1:8081";

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
            "server" | "srv" | "s" => {
                mode = Some("server".to_string());
                i += 1;
            }
            "client" | "cli" | "c" => {
                mode = Some("client".to_string());
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

/// Creates the appropriate server (puller) implementation based on transport type.
/// Returns a boxed trait object that can be used with transport-agnostic code.
fn create_server(transport: Transport) -> Box<dyn FeagiServerPuller> {
    match transport {
        Transport::Zmq => {
            println!("=== Server Puller Example (ZMQ Transport) ===\n");
            println!("Binding to {}", ZMQ_ADDRESS);
            let props = FeagiZmqServerPullerProperties::new(ZMQ_ADDRESS)
                .expect("Failed to create ZMQ server properties");
            props.as_boxed_server_puller()
        }
        Transport::WebSocket => {
            println!("=== Server Puller Example (WebSocket Transport) ===\n");
            println!("Binding to {}", WS_ADDRESS);
            let props = FeagiWebSocketServerPullerProperties::new(WS_ADDRESS)
                .expect("Failed to create WebSocket server properties");
            props.as_boxed_server_puller()
        }
    }
}

/// Creates the appropriate client (pusher) implementation based on transport type.
/// Returns a boxed trait object that can be used with transport-agnostic code.
fn create_client(transport: Transport) -> Box<dyn FeagiClientPusher> {
    match transport {
        Transport::Zmq => {
            println!("=== Client Pusher Example (ZMQ Transport) ===\n");
            println!("Connecting to {}", ZMQ_ADDRESS);
            let props = FeagiZmqClientPusherProperties::new(ZMQ_ADDRESS)
                .expect("Failed to create ZMQ client properties");
            props.as_boxed_client_pusher()
        }
        Transport::WebSocket => {
            println!("=== Client Pusher Example (WebSocket Transport) ===\n");
            println!("Connecting to {}", WS_URL);
            let props = FeagiWebSocketClientPusherProperties::new(WS_URL)
                .expect("Failed to create WebSocket client properties");
            props.as_boxed_client_pusher()
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

/// Runs the server (puller) loop using any FeagiServerPuller implementation.
///
/// This function is completely transport-agnostic - it only uses the trait
/// interface and will work identically with ZMQ, WebSocket, or any future
/// transport implementation.
fn run_server(mut server: Box<dyn FeagiServerPuller>) {
    // Start the server (binds to address)
    server.request_start().expect("Failed to start server");
    println!("Server start requested...");

    // Wait for the server to become active
    loop {
        match server.poll() {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                println!("Server started successfully!");
                break;
            }
            FeagiEndpointState::Pending => {
                thread::sleep(Duration::from_millis(10));
            }
            FeagiEndpointState::Errored(e) => {
                panic!("Server failed to start: {:?}", e);
            }
            FeagiEndpointState::Inactive => {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }

    println!("Waiting for clients to push data...\n");

    // Receive loop
    loop {
        match server.poll() {
            FeagiEndpointState::ActiveHasData => {
                match server.consume_retrieved_data() {
                    Ok(data) => {
                        let message = String::from_utf8_lossy(data);
                        println!("[SERVER] Received: {}", message);
                    }
                    Err(e) => {
                        eprintln!("[SERVER] Error consuming data: {}", e);
                    }
                }
            }
            FeagiEndpointState::ActiveWaiting => {
                // No data yet, keep polling
                thread::sleep(Duration::from_millis(10));
            }
            FeagiEndpointState::Errored(e) => {
                eprintln!("[SERVER] Error: {:?}", e);
                break;
            }
            _ => {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

/// Runs the client (pusher) loop using any FeagiClientPusher implementation.
///
/// This function is completely transport-agnostic - it only uses the trait
/// interface and will work identically with ZMQ, WebSocket, or any future
/// transport implementation.
fn run_client(mut client: Box<dyn FeagiClientPusher>) {
    // Connect to the server
    client.request_connect().expect("Failed to request connection");
    println!("Client connection requested...");

    // Wait for the connection to become active
    loop {
        match client.poll() {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                println!("Client connected successfully!");
                break;
            }
            FeagiEndpointState::Pending => {
                thread::sleep(Duration::from_millis(10));
            }
            FeagiEndpointState::Errored(e) => {
                panic!("Client failed to connect: {:?}", e);
            }
            FeagiEndpointState::Inactive => {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }

    // Brief delay to ensure connection is established
    thread::sleep(Duration::from_millis(200));
    println!("Starting to push messages...\n");

    // Push loop
    let mut counter = 0u64;
    loop {
        let message = format!("Sensory data packet #{} from agent", counter);
        println!("[CLIENT] Pushing: {}", message);

        client
            .publish_data(message.as_bytes())
            .expect("Failed to push data");

        counter += 1;
        thread::sleep(Duration::from_millis(500));
    }
}

fn main() {
    match parse_args() {
        Some((transport, mode)) => match mode.as_str() {
            "server" => {
                let server = create_server(transport);
                run_server(server);
            }
            "client" => {
                let client = create_client(transport);
                run_client(client);
            }
            _ => unreachable!(),
        },
        None => {
            let prog = env::args().next().unwrap_or_default();
            println!("Push-Pull Example");
            println!("Demonstrates transport-agnostic code using trait objects\n");
            println!("Pattern: Client PUSHES data -> Server PULLS/receives data\n");
            println!("Usage:");
            println!("  {} --transport <zmq|ws> <server|client>\n", prog);
            println!("Examples:");
            println!("  {} --transport zmq server", prog);
            println!("  {} --transport zmq client", prog);
            println!("  {} --transport ws server", prog);
            println!("  {} --transport ws client", prog);
            println!();
            println!("Run the server first, then the client in another terminal.");
        }
    }
}
