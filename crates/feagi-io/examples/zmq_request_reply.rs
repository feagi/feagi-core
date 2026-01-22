//! ZMQ Request-Reply Example using FEAGI's `next` module implementations
//!
//! This example demonstrates the request-reply pattern using the
//! `FEAGIZMQServerRouter` (server) and `FEAGIZMQClientRequester` (client) from the `next` module.
//!
//! In this pattern:
//! - The **client** sends a request and polls for a response
//! - The **server** polls for requests, processes them, and sends responses back
//!
//! This is useful for RPC-style communication (e.g., API calls, configuration queries).
//!
//! # Usage
//!
//! Terminal 1 (Server/Router - start first):
//! ```sh
//! cargo run --example zmq_request_reply -- server
//! ```
//!
//! Terminal 2 (Client/Requester):
//! ```sh
//! cargo run --example zmq_request_reply -- client
//! ```

use std::env;
use std::thread;
use std::time::Duration;

use feagi_io::next::implementations::zmq::{FEAGIZMQServerRouter, FEAGIZMQClientRequester};
use feagi_io::next::traits_and_enums::server::{FeagiServer, FeagiServerRouter};
use feagi_io::next::traits_and_enums::client::{FeagiClient, FeagiClientRequester};

const ADDRESS: &str = "tcp://127.0.0.1:5557";

fn run_server() {
    println!("=== FEAGI ZMQ Server Router Example ===\n");
    println!("Starting server (router) on {}", ADDRESS);

    let mut context = zmq::Context::new();

    let mut server = FEAGIZMQServerRouter::new(
        &mut context,
        ADDRESS.to_string(),
        |state_change| println!("[SERVER] State changed: {:?}", state_change)
    ).expect("Failed to create server router");

    server.start().expect("Failed to start server");
    println!("Server started successfully!");
    println!("Waiting for client requests...\n");

    loop {
        // Non-blocking poll for incoming requests - returns Option<(ClientId, &[u8])>
        match server.try_poll_receive() {
            Ok(Some((client_id, request))) => {
                // Request received! client_id identifies which client sent it
                let request_str = String::from_utf8_lossy(request);
                println!("[SERVER] Received request from {:?}: {}", client_id, request_str);

                // Process and create response
                let response_str = format!("Server processed: '{}'", request_str);
                println!("[SERVER] Sending response to {:?}: {}\n", client_id, response_str);

                // Send response back to the specific client using their ClientId
                server.send_response(client_id, response_str.as_bytes())
                    .expect("Failed to send response");
            }
            Ok(None) => {
                // No request available, do other work or sleep briefly
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                eprintln!("[SERVER] Error polling: {}", e);
                break;
            }
        }
    }
}

fn run_client() {
    println!("=== FEAGI ZMQ Client Requester Example ===\n");
    println!("Connecting client (dealer) to {}", ADDRESS);

    let mut context = zmq::Context::new();

    let mut client = FEAGIZMQClientRequester::new(
        &mut context,
        ADDRESS.to_string(),
        |state_change| println!("[CLIENT] State changed: {:?}", state_change)
    ).expect("Failed to create client requester");

    client.connect(ADDRESS).expect("Failed to connect");
    println!("Client connected successfully!\n");

    // Brief delay to ensure connection is established
    thread::sleep(Duration::from_millis(200));

    let mut counter = 0u64;
    loop {
        let request = format!("Request #{}: What is the status?", counter);
        println!("[CLIENT] Sending request: {}", request);

        // Send the request
        client.send_request(request.as_bytes())
            .expect("Failed to send request");

        // Poll for response - returns Option<&[u8]>
        loop {
            match client.try_poll_receive() {
                Ok(Some(response)) => {
                    let response_str = String::from_utf8_lossy(response);
                    println!("[CLIENT] Received response: {}\n", response_str);
                    break;
                }
                Ok(None) => {
                    // No response yet, keep polling
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    eprintln!("[CLIENT] Error polling response: {}", e);
                    break;
                }
            }
        }

        counter += 1;
        thread::sleep(Duration::from_secs(2));
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("FEAGI ZMQ Request-Reply Example");
        println!("Using implementations from feagi_io::next module\n");
        println!("Pattern: Client sends REQUEST → Server processes → Server sends REPLY\n");
        println!("Usage:");
        println!("  {} server   - Start the server (handles requests)", args[0]);
        println!("  {} client   - Start the client (sends requests)", args[0]);
        println!();
        println!("Run the server first, then the client in another terminal.");
        return;
    }

    match args[1].as_str() {
        "server" | "srv" | "s" => run_server(),
        "client" | "cli" | "c" => run_client(),
        _ => {
            eprintln!("Unknown mode: '{}'. Use 'server' or 'client'.", args[1]);
            std::process::exit(1);
        }
    }
}
