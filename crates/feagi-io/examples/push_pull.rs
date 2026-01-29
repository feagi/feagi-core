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
use std::time::Duration;

use feagi_io::implementations::websocket::{
    FEAGIWebSocketClientPusher, FEAGIWebSocketServerPuller,
};
use feagi_io::implementations::zmq::{FEAGIZMQClientPusher, FEAGIZMQServerPuller};
use feagi_io::traits_and_enums::client::FeagiClientPusher;
use feagi_io::traits_and_enums::server::FeagiServerPuller;

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
            Box::new(
                FEAGIZMQServerPuller::new(
                    ZMQ_ADDRESS.to_string(),
                    Box::new(|state| println!("[SERVER] State changed: {:?}", state)),
                )
                .expect("Failed to create ZMQ server"),
            )
        }
        Transport::WebSocket => {
            println!("=== Server Puller Example (WebSocket Transport) ===\n");
            println!("Binding to {}", WS_ADDRESS);
            Box::new(
                FEAGIWebSocketServerPuller::new(
                    WS_ADDRESS.to_string(),
                    Box::new(|state| println!("[SERVER] State changed: {:?}", state)),
                )
                .expect("Failed to create WebSocket server"),
            )
        }
    }
}

/// Creates the appropriate client (pusher) implementation based on transport type.
/// Returns a boxed trait object that can be used with transport-agnostic code.
fn create_client(transport: Transport) -> (Box<dyn FeagiClientPusher>, &'static str) {
    match transport {
        Transport::Zmq => {
            println!("=== Client Pusher Example (ZMQ Transport) ===\n");
            println!("Connecting to {}", ZMQ_ADDRESS);
            (
                Box::new(
                    FEAGIZMQClientPusher::new(
                        ZMQ_ADDRESS.to_string(),
                        Box::new(|state| println!("[CLIENT] State changed: {:?}", state)),
                    )
                    .expect("Failed to create ZMQ client"),
                ),
                ZMQ_ADDRESS,
            )
        }
        Transport::WebSocket => {
            println!("=== Client Pusher Example (WebSocket Transport) ===\n");
            println!("Connecting to {}", WS_URL);
            (
                Box::new(
                    FEAGIWebSocketClientPusher::new(
                        WS_URL.to_string(),
                        Box::new(|state| println!("[CLIENT] State changed: {:?}", state)),
                    )
                    .expect("Failed to create WebSocket client"),
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

/// Runs the server (puller) loop using any FeagiServerPuller implementation.
///
/// This function is completely transport-agnostic - it only uses the trait
/// interface and will work identically with ZMQ, WebSocket, or any future
/// transport implementation.
async fn run_server(mut server: Box<dyn FeagiServerPuller>) {
    // Start the server (binds to address)
    server.start().await.expect("Failed to start server");
    println!("Server started successfully!");
    println!("Waiting for clients to push data...\n");

    // Receive loop
    loop {
        match server.try_poll_receive().await {
            Ok(data) => {
                let message = String::from_utf8_lossy(&data);
                println!("[SERVER] Received: {}", message);
            }
            Err(e) => {
                // Some implementations return errors when no data is available
                // In production, you'd want more sophisticated error handling
                let err_str = e.to_string();
                if !err_str.contains("No clients") && !err_str.contains("No data") {
                    eprintln!("[SERVER] Error polling: {}", e);
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }
}

/// Runs the client (pusher) loop using any FeagiClientPusher implementation.
///
/// This function is completely transport-agnostic - it only uses the trait
/// interface and will work identically with ZMQ, WebSocket, or any future
/// transport implementation.
async fn run_client(mut client: Box<dyn FeagiClientPusher>, address: &str) {
    // Connect to the server
    client.connect(address).await.expect("Failed to connect");
    println!("Client connected successfully!");

    // Brief delay to ensure connection is established
    tokio::time::sleep(Duration::from_millis(200)).await;
    println!("Starting to push messages...\n");

    // Push loop
    let mut counter = 0u64;
    loop {
        let message = format!("Sensory data packet #{} from agent", counter);
        println!("[CLIENT] Pushing: {}", message);

        client
            .push_data(message.as_bytes())
            .await
            .expect("Failed to push data");

        counter += 1;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

#[tokio::main]
async fn main() {
    match parse_args() {
        Some((transport, mode)) => match mode.as_str() {
            "server" => {
                let server = create_server(transport);
                run_server(server).await;
            }
            "client" => {
                let (client, address) = create_client(transport);
                run_client(client, address).await;
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
