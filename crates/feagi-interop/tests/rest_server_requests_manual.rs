//! Manual test: runs the std REST server with the coffee shop category, along with the OpenAPI
//! document and the Swagger UI, until the process is ended with Ctrl+C.
//!
//! Run with:
//! `cargo test -p feagi-interop --features std,swagger_web_ui --test rest_server_requests_manual -- --ignored --nocapture`

#![cfg(all(feature = "std", feature = "swagger_web_ui"))]

mod common;

use std::net::SocketAddr;

use common::{start_coffee_shop, Channels, ShopGroup};
use feagi_basis::parallelism::feagi_worker::feagi_worker_builder::FeagiWorkerBuilder;
use feagi_basis::parallelism::feagi_worker::spawner_implementations::std_thread::FeagiStdThreadSpawner;
use feagi_interop::transport::rest::server::server_basis::RestServerWorkerBuilder;
use feagi_interop::transport::rest::server::server_std::StdRestServerBuilder;

const LISTEN_ADDRESS: &str = "127.0.0.1:8000";

#[test]
#[ignore = "manual: serves until the process is ended with Ctrl+C"]
fn rest_server_serves_coffee_shop_until_ended() {
    let (group, configurables, _responder_thread) = start_coffee_shop("downtown");
    let address: SocketAddr = LISTEN_ADDRESS.parse().expect("listen address is a socket address");
    // Kept so the server keeps running, it is never stopped
    let mut builder = StdRestServerBuilder::<ShopGroup>::new(address);
    builder.open_api_json = Some(feagi_interop::feagi_open_api_json!(ShopGroup, Channels));
    let (_server, ()) = builder
        .build_worker(&FeagiStdThreadSpawner, (group, configurables))
        .expect("REST server failed to start");

    println!("FEAGI REST server listening on http://{address}");
    println!("  OpenAPI document: http://{address}/api-docs/openapi.json");
    println!("  Swagger UI:       http://{address}/swagger-ui/feagi-server.html");
    println!("Coffee shop endpoints (order 7 is the only order):");
    println!("  GET    http://{address}/shops/downtown/orders/7?include_price=true");
    println!("  GET    http://{address}/shops/downtown/status");
    println!("  POST   http://{address}/shops/downtown/orders     body {{\"drink\":1,\"size\":3}}");
    println!("  DELETE http://{address}/shops/downtown/orders/7");
    println!("Press Ctrl+C to end the process.");

    loop {
        std::thread::park();
    }
}
