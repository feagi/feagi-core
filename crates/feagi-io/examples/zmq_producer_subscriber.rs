//! ZMQ Publisher-Subscriber Example using FEAGI's `next` module implementations
//!
//! This example demonstrates the publish-subscribe pattern using the
//! `FEAGIZMQServerPublisher` and `FEAGIZMQClientSubscriber` from the `next` module.
//!
//! # Usage
//!
//! Terminal 1 (Publisher/Server):
//! ```sh
//! cargo run --example zmq_producer_subscriber -- publisher
//! ```
//!
//! Terminal 2 (Subscriber/Client):
//! ```sh
//! cargo run --example zmq_producer_subscriber -- subscriber
//! ```

use std::env;
use std::thread;
use std::time::Duration;

use feagi_io::next::implementations::zmq::{FEAGIZMQServerPublisher, FEAGIZMQClientSubscriber};
use feagi_io::next::traits_and_enums::server::{FeagiServer, FeagiServerPublisher};
use feagi_io::next::traits_and_enums::client::FeagiClient;

const ADDRESS: &str = "tcp://127.0.0.1:5555";

fn run_publisher() {
    println!("=== FEAGI ZMQ Publisher Example ===\n");
    println!("Starting publisher on {}", ADDRESS);

    let mut context = zmq::Context::new();

    let mut publisher = FEAGIZMQServerPublisher::new(&mut context, ADDRESS.to_string())
        .expect("Failed to create publisher");

    publisher.start().expect("Failed to start publisher");
    println!("Publisher started successfully!");

    // ZMQ PUB sockets need a brief warm-up period for subscribers to connect
    println!("Waiting for subscribers to connect...\n");
    thread::sleep(Duration::from_millis(500));

    let mut counter = 0u64;
    loop {
        let message = format!("Message #{}: Hello from FEAGI!", counter);
        println!("[PUB] Sending: {}", message);

        publisher
            .publish(message.as_bytes())
            .expect("Failed to publish message");

        counter += 1;
        thread::sleep(Duration::from_millis(500));
    }
}

/// Callback function for handling received data
fn on_data_received(data: &[u8]) {
    let message = String::from_utf8_lossy(data);
    println!("[SUB] Received: {}", message);
}

fn run_subscriber() {
    println!("=== FEAGI ZMQ Subscriber Example ===\n");
    println!("Connecting subscriber to {}", ADDRESS);

    let mut context = zmq::Context::new();

    let subscriber = FEAGIZMQClientSubscriber::new(&mut context, ADDRESS.to_string(), on_data_received)
        .expect("Failed to create subscriber");

    subscriber.connect(ADDRESS.to_string());
    println!("Subscriber connected. Waiting for messages...\n");

    // Create a raw socket for receiving since our trait-based implementation
    // uses callbacks that would require async infrastructure
    let socket = context.socket(zmq::SUB).expect("Failed to create SUB socket");
    socket.connect(ADDRESS).expect("Failed to connect");
    socket.set_subscribe(b"").expect("Failed to subscribe");

    loop {
        match socket.recv_bytes(0) {
            Ok(bytes) => {
                on_data_received(&bytes);
            }
            Err(e) => {
                eprintln!("[SUB] Error receiving: {}", e);
                break;
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("FEAGI ZMQ Publisher-Subscriber Example");
        println!("Using implementations from feagi_io::next module\n");
        println!("Usage:");
        println!("  {} publisher   - Start the publisher (sends messages)", args[0]);
        println!("  {} subscriber  - Start the subscriber (receives messages)", args[0]);
        println!();
        println!("Run the publisher first, then the subscriber in another terminal.");
        return;
    }

    match args[1].as_str() {
        "publisher" | "pub" | "p" => run_publisher(),
        "subscriber" | "sub" | "s" => run_subscriber(),
        _ => {
            eprintln!("Unknown mode: '{}'. Use 'publisher' or 'subscriber'.", args[1]);
            std::process::exit(1);
        }
    }
}
