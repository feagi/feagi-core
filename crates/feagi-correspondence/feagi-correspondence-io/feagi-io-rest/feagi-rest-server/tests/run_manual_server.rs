//! Manual test: runs the FEAGI REST server (plus a local Swagger UI) and holds it
//! until Ctrl+C.
//!
//! Skipped by default (`#[ignore]`). Not built unless `std_smol` is enabled.
//!
//! Run:
//! `cargo test -p feagi-rest-server --features std_smol --test run_manual_server -- --ignored --nocapture`
//!
//! Then open `/swagger-ui/feagi-server.html` on the bind address below.

use feagi_rest_server::create_ohkami_server;
use ohkami::prelude::*;
use ohkami::claw::content::Html;
use ohkami::openapi;
use std::sync::OnceLock;

//region Swagger assets and OpenAPI document
const SWAGGER_HTML: &str = include_str!("../swagger_site/feagi-server.html");
const SWAGGER_CSS: &[u8] = include_bytes!("../swagger_site/swagger-ui.css");
const SWAGGER_JS: &[u8] = include_bytes!("../swagger_site/swagger-ui-bundle.js");

/// OpenAPI document metadata.
fn rest_openapi_metadata() -> openapi::OpenAPI<'static> {
    openapi::OpenAPI {
        title: "FEAGI REST Server",
        version: env!("CARGO_PKG_VERSION"),
        servers: &[],
    }
}

/// OpenAPI document bytes, generated once from the API routes only (no Swagger routes).
fn openapi_json_bytes() -> &'static [u8] {
    static SPEC: OnceLock<Vec<u8>> = OnceLock::new();
    SPEC.get_or_init(|| create_ohkami_server().__openapi_document_bytes__(rest_openapi_metadata()))
}

async fn swagger_html() -> Html<&'static str> {
    Html(SWAGGER_HTML)
}

async fn swagger_css() -> Response {
    Response::OK().with_payload("text/css; charset=UTF-8", SWAGGER_CSS)
}

async fn swagger_js() -> Response {
    Response::OK().with_payload("application/javascript; charset=UTF-8", SWAGGER_JS)
}

async fn openapi_json() -> Response {
    Response::OK().with_payload("application/json", openapi_json_bytes())
}

async fn redirect_swagger() -> Response {
    Response::Found().with_headers(|h| h.location("/swagger-ui/feagi-server.html"))
}
//endregion

/// Runs the server (API + Swagger UI) until Ctrl+C. Ignored by default crate test runs.
#[test]
#[ignore = "manual: runs a live server and blocks until Ctrl+C"]
fn run_manual_server_until_ctrl_c() {
    // Address the manual server binds to. Change here to pick another host/port.
    const BIND: &str = "127.0.0.1:8080";

    // Merge the generated API routes (at `/`) with the static Swagger UI and the
    // OpenAPI document routes.
    let app = Ohkami::new((
        create_ohkami_server(),
        "/swagger-ui".GET(redirect_swagger),
        "/swagger-ui/feagi-server.html".GET(swagger_html),
        "/swagger-ui/swagger-ui.css".GET(swagger_css),
        "/swagger-ui/swagger-ui-bundle.js".GET(swagger_js),
        "/api-docs/openapi.json".GET(openapi_json),
    ));

    println!("FEAGI REST server listening on {BIND}");
    println!("Swagger UI:   http://{BIND}/swagger-ui/feagi-server.html");
    println!("Health check: http://{BIND}/system/health_check");
    println!("OpenAPI:      http://{BIND}/api-docs/openapi.json");
    println!("Stop with Ctrl+C");

    smol::block_on(async {
        app.howl(BIND).await;
    });
}
