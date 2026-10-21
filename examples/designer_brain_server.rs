//! Launches the designer brain server (the genome manager, and a REST server serving its request
//! categories) and keeps it running until the process is ended with Ctrl+C.
//!
//! Run with:
//! `cargo run --example designer_brain_server`

use std::net::SocketAddr;

use feagi_basis::parallelism::feagi_worker::feagi_worker_builder::FeagiWorkerBuilder;
use feagi_basis::parallelism::feagi_worker::spawner_implementations::std_thread::FeagiStdThreadSpawner;
use feagi_core::runtimes::brain_server::designer_brain_server::DesignerBrainServerWorkerBuilder;

/// HTTP address Brain Visualizer connects to on launch
const REST_SERVER_ADDRESS: &str = "127.0.0.1:8000";

fn main() {
    let address: SocketAddr = REST_SERVER_ADDRESS.parse().expect("REST server address is a socket address");
    let builder = DesignerBrainServerWorkerBuilder { rest_server_socket: address };
    // Kept so the brain server keeps running. It is never stopped, Ctrl+C ends the process
    let (_brain_server, ()) = builder
        .build_worker(&FeagiStdThreadSpawner, ())
        .expect("designer brain server failed to start");

    println!("FEAGI designer brain server running");
    println!("  REST server:      http://{address}");
    println!("  OpenAPI document: http://{address}/api-docs/openapi.json");
    println!("  Swagger UI:       http://{address}/swagger-ui/feagi-server.html");
    println!("Press Ctrl+C to end the process.");

    loop {
        std::thread::park();
    }
}
