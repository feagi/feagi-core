//! Runs the std REST server with a category of messages, answered by a responder on another
//! thread, and makes HTTP requests to it.
//!
//! Run with:
//! `cargo test -p feagi-interop --features std,transport_rest --test rest_server_requests`

#![cfg(all(feature = "std", feature = "transport_rest"))]

mod common;

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use common::{start_coffee_shop, ShopGroup};
use feagi_basis::parallelism::block_on_thread;
use feagi_basis::parallelism::feagi_worker::builder::components::worker_spawner_implementation::FeagiStdThreadSpawner;
use feagi_basis::parallelism::feagi_worker::builder::feagi_worker_builder::FeagiWorkerBuilder;
use feagi_interop::transport::rest::server::server_basis::RestServerWorkerBuilder;
use feagi_interop::transport::rest::server::server_std::StdRestServerBuilder;

/// Fixed so the test can connect to it. Unlikely to be in use
const LISTEN_ADDRESS: &str = "127.0.0.1:18731";

/// Sends one HTTP request and returns the status code and body of the response
fn http(method: &str, path: &str, body: Option<&str>) -> (u16, String) {
    let mut stream = TcpStream::connect(LISTEN_ADDRESS).expect("server accepts connections");
    let body = body.unwrap_or("");
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .unwrap();

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    let status = response.split(' ').nth(1).and_then(|code| code.parse().ok()).expect("response has a status line");
    let body = response.split_once("\r\n\r\n").map(|(_, body)| body.trim().to_string()).unwrap_or_default();
    (status, body)
}

/// The served OpenAPI document lists every message, with configurables as path parameters
#[cfg(feature = "open_api_endpoint")]
fn check_open_api_document() {
    let (status, body) = http("GET", "/api-docs/openapi.json", None);
    assert_eq!(status, 200);
    let document: serde_json::Value = serde_json::from_str(&body).expect("OpenAPI document is json");

    assert_eq!(document["tags"][0]["name"], "CoffeeShop");
    let paths = document["paths"].as_object().unwrap();
    let mut path_keys: Vec<&str> = paths.keys().map(String::as_str).collect();
    path_keys.sort();
    assert_eq!(path_keys, ["/shops/{shop_name}/orders", "/shops/{shop_name}/orders/{order_id}", "/shops/{shop_name}/status"]);

    // Read and Delete share the order's path
    let order = &paths["/shops/{shop_name}/orders/{order_id}"];
    assert_eq!(order["delete"]["description"], "Cancel an order");
    let read_parameters = order["get"]["parameters"].as_array().unwrap();
    // A configurable is a string path parameter
    assert_eq!(read_parameters[0]["name"], "shop_name");
    assert_eq!(read_parameters[0]["in"], "path");
    assert_eq!(read_parameters[0]["schema"]["type"], "string");
    assert_eq!(read_parameters[1]["name"], "order_id");
    assert_eq!(read_parameters[1]["in"], "path");
    assert_eq!(read_parameters[1]["schema"]["type"], "integer");
    // A Read payload is the URL query
    assert_eq!(read_parameters[2]["name"], "include_price");
    assert_eq!(read_parameters[2]["in"], "query");
    assert_eq!(read_parameters[2]["required"], false);
    assert_eq!(order["get"]["responses"]["200"]["content"]["application/json"]["schema"]["properties"]["price"]["type"], "integer");

    // Other payloads are the json body
    let create_body = &paths["/shops/{shop_name}/orders"]["post"]["requestBody"]["content"]["application/json"]["schema"];
    assert_eq!(create_body["required"], serde_json::json!(["drink", "size"]));
}

#[test]
fn rest_server_sends_requests_through_category_channels() {
    let (group, configurables, responder_thread) = start_coffee_shop("downtown");
    let address: SocketAddr = LISTEN_ADDRESS.parse().unwrap();
    #[allow(unused_mut)]
    let mut builder = StdRestServerBuilder::<ShopGroup>::new(address);
    #[cfg(feature = "open_api_endpoint")]
    {
        builder.open_api_json = Some(feagi_interop::feagi_open_api_json!(ShopGroup, common::Channels));
    }
    let (mut server, ()) = builder
        .build_worker(&FeagiStdThreadSpawner, (group, configurables))
        .expect("REST server starts");

    // Read messages take their payload from the query
    assert_eq!(http("GET", "/shops/downtown/orders/7?include_price=true", None), (200, r#"{"drink":2,"price":450}"#.into()));
    assert_eq!(http("GET", "/shops/downtown/orders/7", None), (200, r#"{"drink":2,"price":null}"#.into()));
    assert_eq!(http("GET", "/shops/downtown/status", None), (200, r#"{"open":true}"#.into()));

    // Other messages take theirs from a json body
    assert_eq!(http("POST", "/shops/downtown/orders", Some(r#"{"drink":1,"size":3}"#)), (200, r#"{"order_id":44}"#.into()));

    // Messages of different methods share a path
    assert_eq!(http("DELETE", "/shops/downtown/orders/7", None), (200, "null".into()));

    // A message's own error
    assert_eq!(http("GET", "/shops/downtown/orders/8", None), (400, r#""OrderNotFound""#.into()));

    // The path exists, but not with this method, including methods no message uses
    assert_eq!(http("PUT", "/shops/downtown/orders/7", None).0, 405);
    assert_eq!(http("OPTIONS", "/shops/downtown/orders/7", None).0, 405);

    // Only the category's base path is not a message
    assert_eq!(http("GET", "/shops/downtown", None).0, 404);

    // Paths that are not a message, through a different configurable or a parameter that does not parse
    assert_eq!(http("GET", "/shops/uptown/status", None).0, 404);
    assert_eq!(http("GET", "/shops/downtown/orders/seven", None).0, 404);

    // Payloads that do not parse
    assert_eq!(http("POST", "/shops/downtown/orders", Some(r#"{"drink":"tea"}"#)).0, 400);
    assert_eq!(http("POST", "/shops/downtown/orders", None).0, 400);

    #[cfg(feature = "open_api_endpoint")]
    check_open_api_document();

    block_on_thread(server.stop()).expect("REST server acknowledges stopping");
    server.join().expect("REST server worker did not panic");
    // The server held the last requesters, so the responder stops once it is gone
    responder_thread.join().unwrap();
}
