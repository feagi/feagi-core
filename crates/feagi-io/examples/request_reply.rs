//! Request-Reply Example
//!
//! This example demonstrates the request-reply pattern using trait objects,
//! allowing the same business logic to work with any transport implementation
//! (ZMQ or WebSocket), selected via command-line flag.
//!
//! In this pattern:
//! - The **client** sends a request and waits for a response
//! - The **server** receives requests, processes them, and sends responses back
//!
//! The key demonstration here is that `run_server` and `run_client` are
//! **transport-agnostic** - they only know about the trait interface.
//!
//! # Usage
//!
//! Terminal 1 (Server/Router - start first):
//! ```sh
//! cargo run --example request_reply --features "zmq-transport ws-transport" -- --transport zmq server
//! cargo run --example request_reply --features "zmq-transport ws-transport" -- --transport ws server
//! ```
//!
//! Terminal 2 (Client/Requester):
//! ```sh
//! cargo run --example request_reply --features "zmq-transport ws-transport" -- --transport zmq client
//! cargo run --example request_reply --features "zmq-transport ws-transport" -- --transport ws client
//! ```

use std::env;
use std::time::Duration;

use feagi_io::implementations::websocket::{
    FEAGIWebSocketClientRequester, FEAGIWebSocketServerRouter,
};
use feagi_io::implementations::zmq::{FEAGIZMQClientRequester, FEAGIZMQServerRouter};
use feagi_io::traits_and_enums::client::FeagiClientRequester;
use feagi_io::traits_and_enums::server::FeagiServerRouter;

const ZMQ_ADDRESS: &str = "tcp://127.0.0.1:5557";
const WS_ADDRESS: &str = "127.0.0.1:8082";
const WS_URL: &str = "ws://127.0.0.1:8082";

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

/// Creates the appropriate server (router) implementation based on transport type.
/// Returns a boxed trait object that can be used with transport-agnostic code.
fn create_server(transport: Transport) -> Box<dyn FeagiServerRouter> {
    match transport {
        Transport::Zmq => {
            println!("=== Server Router Example (ZMQ Transport) ===\n");
            println!("Binding to {}", ZMQ_ADDRESS);
            Box::new(
                FEAGIZMQServerRouter::new(
                    ZMQ_ADDRESS.to_string(),
                    Box::new(|state| println!("[SERVER] State changed: {:?}", state)),
                )
                .expect("Failed to create ZMQ server"),
            )
        }
        Transport::WebSocket => {
            println!("=== Server Router Example (WebSocket Transport) ===\n");
            println!("Binding to {}", WS_ADDRESS);
            Box::new(
                FEAGIWebSocketServerRouter::new(
                    WS_ADDRESS.to_string(),
                    Box::new(|state| println!("[SERVER] State changed: {:?}", state)),
                )
                .expect("Failed to create WebSocket server"),
            )
        }
    }
}

/// Creates the appropriate client (requester) implementation based on transport type.
/// Returns a boxed trait object that can be used with transport-agnostic code.
fn create_client(transport: Transport) -> (Box<dyn FeagiClientRequester>, &'static str) {
    match transport {
        Transport::Zmq => {
            println!("=== Client Requester Example (ZMQ Transport) ===\n");
            println!("Connecting to {}", ZMQ_ADDRESS);
            (
                Box::new(
                    FEAGIZMQClientRequester::new(
                        ZMQ_ADDRESS.to_string(),
                        Box::new(|state| println!("[CLIENT] State changed: {:?}", state)),
                    )
                    .expect("Failed to create ZMQ client"),
                ),
                ZMQ_ADDRESS,
            )
        }
        Transport::WebSocket => {
            println!("=== Client Requester Example (WebSocket Transport) ===\n");
            println!("Connecting to {}", WS_URL);
            (
                Box::new(
                    FEAGIWebSocketClientRequester::new(
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

/// Runs the server (router) loop using any FeagiServerRouter implementation.
///
/// This function is completely transport-agnostic - it only uses the trait
/// interface and will work identically with ZMQ, WebSocket, or any future
/// transport implementation.
async fn run_server(mut server: Box<dyn FeagiServerRouter>) {
    // Start the server (binds to address)
    server.start().await.expect("Failed to start server");
    println!("Server started successfully!");
    println!("Waiting for client requests...\n");

    // Request handling loop
    loop {
        match server.try_poll_receive().await {
            Ok((session_id, request)) => {
                let request_str = String::from_utf8_lossy(&request);
                println!(
                    "[SERVER] Received request from {:?}: {}",
                    session_id, request_str
                );

                // Process the request and create a response
                let response_str = format!("Server processed: '{}'", request_str);
                println!(
                    "[SERVER] Sending response to {:?}: {}\n",
                    session_id, response_str
                );

                // Send the response back to the specific client
                server
                    .send_response(session_id, response_str.as_bytes())
                    .await
                    .expect("Failed to send response");
            }
            Err(e) => {
                // Some implementations return errors when no data is available
                let err_str = e.to_string();
                if !err_str.contains("No clients") && !err_str.contains("No data") {
                    eprintln!("[SERVER] Error polling: {}", e);
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }
}

/// Runs the client (requester) loop using any FeagiClientRequester implementation.
///
/// This function is completely transport-agnostic - it only uses the trait
/// interface and will work identically with ZMQ, WebSocket, or any future
/// transport implementation.
async fn run_client(mut client: Box<dyn FeagiClientRequester>, address: &str) {
    // Connect to the server
    client.connect(address).await.expect("Failed to connect");
    println!("Client connected successfully!\n");

    // Brief delay to ensure connection is established
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Request-response loop
    let mut counter = 0u64;
    loop {
        let request = format!("Request #{}: What is the status?", counter);
        println!("[CLIENT] Sending request: {}", request);

        // Send the request
        client
            .send_request(request.as_bytes())
            .await
            .expect("Failed to send request");

        // Wait for and handle the response
        match client.get_response().await {
            Ok(response) => {
                let response_str = String::from_utf8_lossy(&response);
                println!("[CLIENT] Received response: {}\n", response_str);
            }
            Err(e) => {
                eprintln!("[CLIENT] Error getting response: {}", e);
            }
        }

        counter += 1;
        tokio::time::sleep(Duration::from_secs(2)).await;
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
            println!("Request-Reply Example");
            println!("Demonstrates transport-agnostic code using trait objects\n");
            println!("Pattern: Client sends REQUEST -> Server processes -> Server sends REPLY\n");
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
