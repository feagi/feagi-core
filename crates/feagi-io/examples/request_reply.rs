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
use std::thread;
use std::time::Duration;

use feagi_config::{load_config, FeagiConfig};
use feagi_io::core::protocol_implementations::websocket::{
    FeagiWebSocketClientRequesterProperties, FeagiWebSocketServerRouterProperties,
};
use feagi_io::core::protocol_implementations::zmq::{
    FeagiZmqClientRequesterProperties, FeagiZmqServerRouterProperties,
};
use feagi_io::core::traits_and_enums::client::{FeagiClientRequester, FeagiClientRequesterProperties};
use feagi_io::core::traits_and_enums::server::{FeagiServerRouter, FeagiServerRouterProperties};
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
        zmq_address: format_tcp_endpoint(&config.zmq.host, config.ports.zmq_req_rep_port),
        ws_address: format_ws_address(&config.websocket.host, config.websocket.rest_api_port),
        ws_url: format_ws_url(&config.websocket.host, config.websocket.rest_api_port),
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
fn create_server(transport: Transport, endpoints: &ExampleEndpoints) -> Box<dyn FeagiServerRouter> {
    match transport {
        Transport::Zmq => {
            println!("=== Server Router Example (ZMQ Transport) ===\n");
            println!("Binding to {}", endpoints.zmq_address);
            let props = FeagiZmqServerRouterProperties::new(&endpoints.zmq_address)
                .expect("Failed to create ZMQ server properties");
            props.as_boxed_server_router()
        }
        Transport::WebSocket => {
            println!("=== Server Router Example (WebSocket Transport) ===\n");
            println!("Binding to {}", endpoints.ws_address);
            let props = FeagiWebSocketServerRouterProperties::new(&endpoints.ws_address)
                .expect("Failed to create WebSocket server properties");
            props.as_boxed_server_router()
        }
    }
}

/// Creates the appropriate client (requester) implementation based on transport type.
/// Returns a boxed trait object that can be used with transport-agnostic code.
fn create_client(transport: Transport, endpoints: &ExampleEndpoints) -> Box<dyn FeagiClientRequester> {
    match transport {
        Transport::Zmq => {
            println!("=== Client Requester Example (ZMQ Transport) ===\n");
            println!("Connecting to {}", endpoints.zmq_address);
            let props = FeagiZmqClientRequesterProperties::new(&endpoints.zmq_address)
                .expect("Failed to create ZMQ client properties");
            props.as_boxed_client_requester()
        }
        Transport::WebSocket => {
            println!("=== Client Requester Example (WebSocket Transport) ===\n");
            println!("Connecting to {}", endpoints.ws_url);
            let props = FeagiWebSocketClientRequesterProperties::new(&endpoints.ws_url)
                .expect("Failed to create WebSocket client properties");
            props.as_boxed_client_requester()
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
fn run_server(mut server: Box<dyn FeagiServerRouter>) {
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

    println!("Waiting for client requests...\n");

    // Request handling loop
    loop {
        match server.poll() {
            FeagiEndpointState::ActiveHasData => {
                match server.consume_retrieved_request() {
                    Ok((session_id, request)) => {
                        let request_str = String::from_utf8_lossy(request);
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
                            .publish_response(session_id, response_str.as_bytes())
                            .expect("Failed to send response");
                    }
                    Err(e) => {
                        eprintln!("[SERVER] Error consuming request: {}", e);
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

/// Runs the client (requester) loop using any FeagiClientRequester implementation.
///
/// This function is completely transport-agnostic - it only uses the trait
/// interface and will work identically with ZMQ, WebSocket, or any future
/// transport implementation.
fn run_client(mut client: Box<dyn FeagiClientRequester>) {
    // Connect to the server
    client.request_connect().expect("Failed to request connection");
    println!("Client connection requested...");

    // Wait for the connection to become active
    loop {
        match client.poll() {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                println!("Client connected successfully!\n");
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

    // Request-response loop
    let mut counter = 0u64;
    loop {
        let request = format!("Request #{}: What is the status?", counter);
        println!("[CLIENT] Sending request: {}", request);

        // Send the request
        client
            .publish_request(request.as_bytes())
            .expect("Failed to send request");

        // Wait for and handle the response
        loop {
            match client.poll() {
                FeagiEndpointState::ActiveHasData => {
                    match client.consume_retrieved_response() {
                        Ok(response) => {
                            let response_str = String::from_utf8_lossy(response);
                            println!("[CLIENT] Received response: {}\n", response_str);
                        }
                        Err(e) => {
                            eprintln!("[CLIENT] Error getting response: {}", e);
                        }
                    }
                    break;
                }
                FeagiEndpointState::ActiveWaiting => {
                    thread::sleep(Duration::from_millis(10));
                }
                FeagiEndpointState::Errored(e) => {
                    eprintln!("[CLIENT] Error: {:?}", e);
                    return;
                }
                _ => {
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }

        counter += 1;
        thread::sleep(Duration::from_secs(2));
    }
}

fn main() {
    let endpoints = build_endpoints();
    match parse_args() {
        Some((transport, mode)) => match mode.as_str() {
            "server" => {
                let server = create_server(transport, &endpoints);
                run_server(server);
            }
            "client" => {
                let client = create_client(transport, &endpoints);
                run_client(client);
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
