//! ZMQ Push-Pull Example using FEAGI's `next` module implementations
//!
//! This example demonstrates the push-pull pattern using the
//! `FEAGIZMQClientPusher` (client) and `FEAGIZMQServerPuller` (server) from the `next` module.
//!
//! In this pattern:
//! - The **server** binds and waits to receive (pull) data
//! - The **client** connects and sends (pushes) data to the server
//!
//! This is useful for scenarios where agents send data to FEAGI (e.g., sensory data).
//!
//! # Usage
//!
//! Terminal 1 (Server/Puller - start first):
//! ```sh
//! cargo run --example zmq_push_pull -- server
//! ```
//!
//! Terminal 2 (Client/Pusher):
//! ```sh
//! cargo run --example zmq_push_pull -- client
//! ```

use std::env;
use std::thread;
use std::time::Duration;

use feagi_io::next::implementations::zmq::{FEAGIZMQServerPuller, FEAGIZMQClientPusher};
use feagi_io::next::traits_and_enums::server::{FeagiServer, FeagiServerPuller};
use feagi_io::next::traits_and_enums::client::{FeagiClient, FeagiClientPusher};

const ADDRESS: &str = "tcp://127.0.0.1:5556";

fn run_server() {
    println!("=== FEAGI ZMQ Server Puller Example ===\n");
    println!("Starting server (puller) on {}", ADDRESS);

    let mut context = zmq::Context::new();

    let mut server = FEAGIZMQServerPuller::new(&mut context, ADDRESS.to_string())
        .expect("Failed to create server puller");

    server.start().expect("Failed to start server");
    println!("Server started successfully!");
    println!("Waiting for clients to push data...\n");

    loop {
        // Non-blocking poll - check if data is available
        match server.try_poll() {
            Ok(true) => {
                // Data received!
                let data = server.get_cached_data();
                let message = String::from_utf8_lossy(data);
                println!("[SERVER] Received: {}", message);
            }
            Ok(false) => {
                // No data available, do other work or sleep briefly
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
    println!("=== FEAGI ZMQ Client Pusher Example ===\n");
    println!("Connecting client (pusher) to {}", ADDRESS);

    let mut context = zmq::Context::new();

    let client = FEAGIZMQClientPusher::new(&mut context, ADDRESS.to_string())
        .expect("Failed to create client pusher");

    client.connect(ADDRESS.to_string());
    println!("Client connected successfully!");

    // Brief delay to ensure connection is established
    thread::sleep(Duration::from_millis(200));
    println!("Starting to push messages...\n");

    let mut counter = 0u64;
    loop {
        let message = format!("Sensory data packet #{} from agent", counter);
        println!("[CLIENT] Pushing: {}", message);

        client.push_data(message.as_bytes());

        counter += 1;
        thread::sleep(Duration::from_millis(500));
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("FEAGI ZMQ Push-Pull Example");
        println!("Using implementations from feagi_io::next module\n");
        println!("Pattern: Client PUSHES data → Server PULLS/receives data\n");
        println!("Usage:");
        println!("  {} server   - Start the server (receives pushed data)", args[0]);
        println!("  {} client   - Start the client (pushes data to server)", args[0]);
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
