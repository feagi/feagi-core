//! Request-Reply Example
//!
//! This example demonstrates the request-reply pattern using either
//! ZMQ or WebSocket transport, selected via command-line flag.
//!
//! In this pattern:
//! - The **client** sends a request and waits for a response
//! - The **server** receives requests, processes them, and sends responses back
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
use feagi_io::traits_and_enums::client::{FeagiClient, FeagiClientRequester};
use feagi_io::traits_and_enums::server::{FeagiServer, FeagiServerRouter};

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

async fn run_zmq_server() {
    println!("=== ZMQ Server Router Example ===\n");
    println!("Starting server on {}", ZMQ_ADDRESS);

    let mut server = FEAGIZMQServerRouter::new(
        ZMQ_ADDRESS.to_string(),
        Box::new(|state_change| println!("[SERVER] State changed: {:?}", state_change)),
    )
    .expect("Failed to create server");

    server.start().await.expect("Failed to start server");
    println!("Server started successfully!");
    println!("Waiting for client requests...\n");

    loop {
        match server.try_poll_receive().await {
            Ok((session_id, request)) => {
                let request_str = String::from_utf8_lossy(&request);
                println!(
                    "[SERVER] Received request from {:?}: {}",
                    session_id, request_str
                );

                // Process and create response
                let response_str = format!("Server processed: '{}'", request_str);
                println!(
                    "[SERVER] Sending response to {:?}: {}\n",
                    session_id, response_str
                );

                server
                    .send_response(session_id, response_str.as_bytes())
                    .await
                    .expect("Failed to send response");
            }
            Err(e) => {
                // For ZMQ, this might just mean no data yet
                if !e.to_string().contains("No data") {
                    // Just wait a bit and try again
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }
}

async fn run_zmq_client() {
    println!("=== ZMQ Client Requester Example ===\n");
    println!("Connecting to {}", ZMQ_ADDRESS);

    let mut client = FEAGIZMQClientRequester::new(
        ZMQ_ADDRESS.to_string(),
        Box::new(|state_change| println!("[CLIENT] State changed: {:?}", state_change)),
    )
    .expect("Failed to create client");

    client.connect(ZMQ_ADDRESS).await.expect("Failed to connect");
    println!("Client connected successfully!\n");

    tokio::time::sleep(Duration::from_millis(200)).await;

    let mut counter = 0u64;
    loop {
        let request = format!("Request #{}: What is the status?", counter);
        println!("[CLIENT] Sending request: {}", request);

        client
            .send_request(request.as_bytes())
            .await
            .expect("Failed to send request");

        // Wait for response
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

async fn run_ws_server() {
    println!("=== WebSocket Server Router Example ===\n");
    println!("Starting server on {}", WS_ADDRESS);

    let mut server = FEAGIWebSocketServerRouter::new(
        WS_ADDRESS.to_string(),
        Box::new(|state_change| println!("[SERVER] State changed: {:?}", state_change)),
    )
    .expect("Failed to create server");

    server.start().await.expect("Failed to start server");
    println!("Server started successfully!");
    println!("Waiting for client requests...\n");

    loop {
        match server.try_poll_receive().await {
            Ok((session_id, request)) => {
                let request_str = String::from_utf8_lossy(&request);
                println!(
                    "[SERVER] Received request from {:?}: {}",
                    session_id, request_str
                );

                // Process and create response
                let response_str = format!("Server processed: '{}'", request_str);
                println!(
                    "[SERVER] Sending response to {:?}: {}\n",
                    session_id, response_str
                );

                server
                    .send_response(session_id, response_str.as_bytes())
                    .await
                    .expect("Failed to send response");
            }
            Err(e) => {
                // WebSocket returns error when no clients
                if !e.to_string().contains("No clients") {
                    eprintln!("[SERVER] Error: {}", e);
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

async fn run_ws_client() {
    println!("=== WebSocket Client Requester Example ===\n");
    println!("Connecting to {}", WS_URL);

    let mut client = FEAGIWebSocketClientRequester::new(
        WS_URL.to_string(),
        Box::new(|state_change| println!("[CLIENT] State changed: {:?}", state_change)),
    )
    .expect("Failed to create client");

    client.connect(WS_URL).await.expect("Failed to connect");
    println!("Client connected successfully!\n");

    let mut counter = 0u64;
    loop {
        let request = format!("Request #{}: What is the status?", counter);
        println!("[CLIENT] Sending request: {}", request);

        client
            .send_request(request.as_bytes())
            .await
            .expect("Failed to send request");

        // Wait for response
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
        Some((Transport::Zmq, mode)) => match mode.as_str() {
            "server" => run_zmq_server().await,
            "client" => run_zmq_client().await,
            _ => unreachable!(),
        },
        Some((Transport::WebSocket, mode)) => match mode.as_str() {
            "server" => run_ws_server().await,
            "client" => run_ws_client().await,
            _ => unreachable!(),
        },
        None => {
            let prog = env::args().next().unwrap_or_default();
            println!("Request-Reply Example");
            println!("Tests both ZMQ and WebSocket transports\n");
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
