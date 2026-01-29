//! Push-Pull Example
//!
//! This example demonstrates the push-pull pattern using either
//! ZMQ or WebSocket transport, selected via command-line flag.
//!
//! In this pattern:
//! - The **server** binds and waits to receive (pull) data
//! - The **client** connects and sends (pushes) data to the server
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
use feagi_io::traits_and_enums::client::{FeagiClient, FeagiClientPusher};
use feagi_io::traits_and_enums::server::{FeagiServer, FeagiServerPuller};

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

async fn run_zmq_server() {
    println!("=== ZMQ Server Puller Example ===\n");
    println!("Starting server on {}", ZMQ_ADDRESS);

    let mut server = FEAGIZMQServerPuller::new(
        ZMQ_ADDRESS.to_string(),
        Box::new(|state_change| println!("[SERVER] State changed: {:?}", state_change)),
    )
    .expect("Failed to create server");

    server.start().await.expect("Failed to start server");
    println!("Server started successfully!");
    println!("Waiting for clients to push data...\n");

    loop {
        match server.try_poll_receive().await {
            Ok(data) => {
                let message = String::from_utf8_lossy(&data);
                println!("[SERVER] Received: {}", message);
            }
            Err(e) => {
                eprintln!("[SERVER] Error polling: {}", e);
                // For ZMQ, this might just mean no data yet
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }
}

async fn run_zmq_client() {
    println!("=== ZMQ Client Pusher Example ===\n");
    println!("Connecting to {}", ZMQ_ADDRESS);

    let mut client = FEAGIZMQClientPusher::new(
        ZMQ_ADDRESS.to_string(),
        Box::new(|state_change| println!("[CLIENT] State changed: {:?}", state_change)),
    )
    .expect("Failed to create client");

    client.connect(ZMQ_ADDRESS).await.expect("Failed to connect");
    println!("Client connected successfully!");

    tokio::time::sleep(Duration::from_millis(200)).await;
    println!("Starting to push messages...\n");

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

async fn run_ws_server() {
    println!("=== WebSocket Server Puller Example ===\n");
    println!("Starting server on {}", WS_ADDRESS);

    let mut server = FEAGIWebSocketServerPuller::new(
        WS_ADDRESS.to_string(),
        Box::new(|state_change| println!("[SERVER] State changed: {:?}", state_change)),
    )
    .expect("Failed to create server");

    server.start().await.expect("Failed to start server");
    println!("Server started successfully!");
    println!("Waiting for clients to push data...\n");

    loop {
        match server.try_poll_receive().await {
            Ok(data) => {
                let message = String::from_utf8_lossy(&data);
                println!("[SERVER] Received: {}", message);
            }
            Err(e) => {
                // WebSocket returns error when no clients or no data
                if !e.to_string().contains("No clients") {
                    eprintln!("[SERVER] Error: {}", e);
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

async fn run_ws_client() {
    println!("=== WebSocket Client Pusher Example ===\n");
    println!("Connecting to {}", WS_URL);

    let mut client = FEAGIWebSocketClientPusher::new(
        WS_URL.to_string(),
        Box::new(|state_change| println!("[CLIENT] State changed: {:?}", state_change)),
    )
    .expect("Failed to create client");

    client.connect(WS_URL).await.expect("Failed to connect");
    println!("Client connected successfully!");
    println!("Starting to push messages...\n");

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
            println!("Push-Pull Example");
            println!("Tests both ZMQ and WebSocket transports\n");
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
