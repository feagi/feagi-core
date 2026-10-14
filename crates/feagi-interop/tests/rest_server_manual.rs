//! Manual test: runs the std REST server as a worker until stopped by hand.
//!
//! Run with:
//! `cargo test -p feagi-interop --features std,swagger_web_ui --test rest_server_manual -- --ignored --nocapture`
//!
//! Press Enter to stop the server cleanly through `stop_rest_server`, or Ctrl+C to end the
//! process. The server binds `127.0.0.1:8000`, the HTTP port Brain Visualizer uses by default.

#![cfg(all(feature = "std", feature = "transport_rest"))]

use std::io::BufRead;
use std::net::{SocketAddr, TcpStream};

use feagi_basis::parallelism::block_on_thread;
use feagi_interop::rest::server::server_common::FeagiRestServerWorker;
use feagi_interop::rest::server::server_std::{RestServer, RestServerStartParameters};

/// HTTP address Brain Visualizer connects to on launch.
const LISTEN_ADDRESS: &str = "127.0.0.1:8000";

/// Returns the fixed listen address used by this manual server.
fn listen_address() -> SocketAddr {
    LISTEN_ADDRESS.parse().expect("listen address is a socket address")
}

/// Prints the address and the routes enabled by the current features.
fn print_routes(address: SocketAddr) {
    println!("FEAGI REST server listening on http://{address}");
    #[cfg(feature = "open_api_endpoint")]
    println!("  OpenAPI document: http://{address}/api-docs/openapi.json");
    #[cfg(feature = "swagger_web_ui")]
    println!("  Swagger UI:       http://{address}/swagger-ui/feagi-server.html");
    println!("Press Enter to stop the server, or Ctrl+C to end the process.");
}

/// Blocks until a line is entered. If stdin is closed, waits until the process is ended instead.
fn wait_for_enter() {
    let mut line = String::new();
    let bytes_read = std::io::stdin().lock().read_line(&mut line).unwrap_or(0);
    if bytes_read == 0 {
        println!("stdin is closed, waiting for Ctrl+C.");
        loop {
            std::thread::park();
        }
    }
}

#[test]
#[ignore = "manual: serves until stopped by hand"]
fn rest_server_runs_until_stopped_manually() {
    let address = listen_address();
    let server = RestServer::start_rest_server(RestServerStartParameters { socket_address: address }).expect("REST server failed to start");
    print_routes(address);

    wait_for_enter();

    println!("Stopping REST server...");
    let stopper = std::thread::spawn(move || block_on_thread(server.stop_rest_server()));
    // The worker only sees the stop after its blocking accept returns, so wake it with one connection
    let _ = TcpStream::connect(address);
    stopper.join().expect("stop thread panicked").expect("REST server failed to stop");
    println!("REST server stopped.");
}
