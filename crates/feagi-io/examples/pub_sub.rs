//! Publisher-Subscriber Example
//!
//! This example demonstrates the publish-subscribe pattern using either
//! ZMQ or WebSocket transport, selected via command-line flag.
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
use std::time::Duration;

use feagi_io::implementations::websocket::{
    FEAGIWebSocketClientSubscriber, FEAGIWebSocketServerPublisher,
};
use feagi_io::implementations::zmq::{FEAGIZMQClientSubscriber, FEAGIZMQServerPublisher};
use feagi_io::traits_and_enums::client::{FeagiClient, FeagiClientSubscriber};
use feagi_io::traits_and_enums::server::{FeagiServer, FeagiServerPublisher};

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

async fn run_zmq_publisher() {
    println!("=== ZMQ Publisher Example ===\n");
    println!("Starting publisher on {}", ZMQ_ADDRESS);

    let mut publisher = FEAGIZMQServerPublisher::new(
        ZMQ_ADDRESS.to_string(),
        Box::new(|state_change| println!("[PUB] State changed: {:?}", state_change)),
    )
    .expect("Failed to create publisher");

    publisher.start().await.expect("Failed to start publisher");
    println!("Publisher started successfully!");
    println!("Waiting for subscribers to connect...\n");

    // ZMQ PUB sockets need a brief warm-up period
    tokio::time::sleep(Duration::from_millis(500)).await;

    let mut counter = 0u64;
    loop {
        publisher.poll().await.expect("Failed to poll");

        let message = format!("Message #{}: Hello from FEAGI!", counter);
        println!("[PUB] Sending: {}", message);

        publisher
            .publish(message.as_bytes())
            .await
            .expect("Failed to publish");

        counter += 1;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

async fn run_zmq_subscriber() {
    println!("=== ZMQ Subscriber Example ===\n");
    println!("Connecting to {}", ZMQ_ADDRESS);

    let mut subscriber = FEAGIZMQClientSubscriber::new(
        ZMQ_ADDRESS.to_string(),
        Box::new(|state_change| println!("[SUB] State changed: {:?}", state_change)),
    )
    .expect("Failed to create subscriber");

    subscriber
        .connect(ZMQ_ADDRESS)
        .await
        .expect("Failed to connect");
    println!("Subscriber connected. Waiting for messages...\n");

    loop {
        match subscriber.get_subscribed_data().await {
            Ok(data) => {
                let message = String::from_utf8_lossy(&data);
                println!("[SUB] Received: {}", message);
            }
            Err(e) => {
                eprintln!("[SUB] Error receiving: {}", e);
                break;
            }
        }
    }
}

async fn run_ws_publisher() {
    println!("=== WebSocket Publisher Example ===\n");
    println!("Starting publisher on {}", WS_ADDRESS);

    let mut publisher = FEAGIWebSocketServerPublisher::new(
        WS_ADDRESS.to_string(),
        Box::new(|state_change| println!("[PUB] State changed: {:?}", state_change)),
    )
    .expect("Failed to create publisher");

    publisher.start().await.expect("Failed to start publisher");
    println!("Publisher started successfully!");
    println!("Waiting for subscribers to connect...\n");

    let mut counter = 0u64;
    loop {
        // Accept new WebSocket connections
        publisher.poll().await.expect("Failed to poll");

        if publisher.client_count() > 0 {
            let message = format!("Message #{}: Hello from FEAGI!", counter);
            println!("[PUB] Sending to {} clients: {}", publisher.client_count(), message);

            publisher
                .publish(message.as_bytes())
                .await
                .expect("Failed to publish");

            counter += 1;
        } else {
            println!("[PUB] No clients connected, waiting...");
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

async fn run_ws_subscriber() {
    println!("=== WebSocket Subscriber Example ===\n");
    println!("Connecting to {}", WS_URL);

    let mut subscriber = FEAGIWebSocketClientSubscriber::new(
        WS_URL.to_string(),
        Box::new(|state_change| println!("[SUB] State changed: {:?}", state_change)),
    )
    .expect("Failed to create subscriber");

    subscriber.connect(WS_URL).await.expect("Failed to connect");
    println!("Subscriber connected. Waiting for messages...\n");

    loop {
        match subscriber.get_subscribed_data().await {
            Ok(data) => {
                let message = String::from_utf8_lossy(&data);
                println!("[SUB] Received: {}", message);
            }
            Err(e) => {
                eprintln!("[SUB] Error receiving: {}", e);
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    match parse_args() {
        Some((Transport::Zmq, mode)) => match mode.as_str() {
            "publisher" => run_zmq_publisher().await,
            "subscriber" => run_zmq_subscriber().await,
            _ => unreachable!(),
        },
        Some((Transport::WebSocket, mode)) => match mode.as_str() {
            "publisher" => run_ws_publisher().await,
            "subscriber" => run_ws_subscriber().await,
            _ => unreachable!(),
        },
        None => {
            let prog = env::args().next().unwrap_or_default();
            println!("Publisher-Subscriber Example");
            println!("Tests both ZMQ and WebSocket transports\n");
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
