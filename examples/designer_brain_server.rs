//! Launches the designer brain server (the genome manager, a REST server serving its and the
//! system request categories, and a zenoh websocket server announcing it) and keeps it running
//! until the process is ended with Ctrl+C.
//!
//! Run with:
//! `cargo run --example designer_brain_server`

use std::net::SocketAddr;

use feagi_basis::parallelism::feagi_worker::builder::components::worker_spawner_implementation::FeagiStdThreadSpawner;
use feagi_basis::parallelism::feagi_worker::builder::feagi_worker_builder::FeagiWorkerBuilder;
use feagi_core::runtimes::brain_server::designer_brain_server::DesignerBrainServerWorkerBuilder;
use feagi_interop::transport::zenoh::zenoh_configuration::ZenohConfigurationBuilder;

/// HTTP address Brain Visualizer connects to on launch
const REST_SERVER_ADDRESS: &str = "127.0.0.1:8000";

/// Address other nodes connect to the brain server's zenoh websocket on
const ZENOH_WS_SERVER_ADDRESS: &str = "127.0.0.1:7446";

fn main() {
    let address: SocketAddr = REST_SERVER_ADDRESS.parse().expect("REST server address is a socket address");
    let zenoh_address: SocketAddr = ZENOH_WS_SERVER_ADDRESS.parse().expect("zenoh websocket address is a socket address");

    let mut zenoh_configuration = ZenohConfigurationBuilder::new();
    zenoh_configuration.set_ws_server_bind(zenoh_address);
    let zenoh_configuration = zenoh_configuration.build().expect("a websocket server is enabled");

    let builder = DesignerBrainServerWorkerBuilder { rest_server_socket: address, zenoh_configuration };
    // Kept so the brain server keeps running. It is never stopped, Ctrl+C ends the process
    let (_brain_server, ()) = builder
        .build_worker(&FeagiStdThreadSpawner, ())
        .expect("designer brain server failed to start");

    println!("FEAGI designer brain server running");
    println!("  REST server:      http://{address}");
    println!("  OpenAPI document: http://{address}/api-docs/openapi.json");
    println!("  Swagger UI:       http://{address}/swagger-ui/feagi-server.html");
    println!("  Zenoh websocket:  ws/{zenoh_address}");
    println!("Press Ctrl+C to end the process.");

    loop {
        std::thread::park();
    }
}
