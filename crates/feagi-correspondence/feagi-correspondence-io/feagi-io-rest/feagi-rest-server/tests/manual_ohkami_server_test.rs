//! Manual Ohkami server for exercising the example API and local Swagger UI.
//!
//! Skipped by default (`#[ignore]`). Not built unless `std_smol` is enabled.
//!
//! Run:
//! `cargo test -p feagi-rest-server --features std_smol --test manual_ohkami_server_test -- --ignored --nocapture`
//!
//! Then open `/swagger-ui/feagi-server.html` on the port below.

/// Listen port for this manual server. Change this value to pick another port.
const MANUAL_OHKAMI_PORT: u16 = 8080;

use ohkami::prelude::*;
use ohkami::claw::content::Html;
use ohkami::claw::{Json, Path, Query};
use ohkami::openapi;
use ohkami::serde::{Deserialize, Serialize};
use std::sync::OnceLock;

//region Swagger Stuff
const SWAGGER_HTML: &str = include_str!("../swagger_site/feagi-server.html");
const SWAGGER_CSS: &[u8] = include_bytes!("../swagger_site/swagger-ui.css");
const SWAGGER_JS: &[u8] = include_bytes!("../swagger_site/swagger-ui-bundle.js");

fn rest_openapi_metadata() -> openapi::OpenAPI<'static> {
    openapi::OpenAPI {
        title: "FEAGI REST Server",
        version: env!("CARGO_PKG_VERSION"),
        servers: &[],
    }
}

/// API routes only (used for OpenAPI generation).
fn api_ohkami() -> Ohkami {
    Ohkami::new(("/testing/:parse_path/:parse_path_2".GET(handle_testing),))
}

fn openapi_json_bytes() -> &'static [u8] {
    static SPEC: OnceLock<Vec<u8>> = OnceLock::new();
    SPEC.get_or_init(|| api_ohkami().__openapi_document_bytes__(rest_openapi_metadata()))
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

//region Test Structs and Handling

/// Parsed values from `GET /testing/:parse_path/:parse_path_2?parse_parameter=&parse_parameter_2=`.
// OpenAPI: Schema derive registers field rustdocs as property descriptions.
#[derive(openapi::Schema)]
struct TestingRequest {
    /// First path segment (`:parse_path`).
    // OpenAPI: property description
    parse_path: String,
    /// Second path segment (`:parse_path_2`).
    // OpenAPI: property description
    parse_path_2: String,
    /// First integer query parameter (`parse_parameter`).
    // OpenAPI: property description
    parse_parameter: i32,
    /// Second integer query parameter (`parse_parameter_2`).
    // OpenAPI: property description
    parse_parameter_2: i32,
}

/// JSON body with both path strings reversed and both query integers doubled.
// OpenAPI: Schema + component; field rustdocs become property descriptions.
#[derive(Serialize, openapi::Schema)]
#[openapi(component)]
struct TestingResponse {
    /// First path segment, reversed.
    // OpenAPI: property description
    parse_path: String,
    /// Second path segment, reversed.
    // OpenAPI: property description
    parse_path_2: String,
    /// First query integer, doubled.
    // OpenAPI: property description
    parse_parameter: i32,
    /// Second query integer, doubled.
    // OpenAPI: property description
    parse_parameter_2: i32,
}

impl TestingResponse {
    /// Build a response from a parsed request: reverse both strings, double both numbers.
    fn new(request: TestingRequest) -> Self {
        Self {
            parse_path: request.parse_path.chars().rev().collect(),
            parse_path_2: request.parse_path_2.chars().rev().collect(),
            parse_parameter: request.parse_parameter * 2,
            parse_parameter_2: request.parse_parameter_2 * 2,
        }
    }
}

/// Query parameters for the testing route.
// OpenAPI: field rustdocs become each query parameter's schema description.
#[derive(Deserialize, openapi::Schema)]
struct TestingQuery {
    /// First query integer (`parse_parameter`). Doubled in the response.
    parse_parameter: i32,
    /// Second query integer (`parse_parameter_2`). Doubled in the response.
    parse_parameter_2: i32,
}

/// Reverse both path strings and double both query integers.
// OpenAPI: handler rustdoc is the operation description.
// Query parameter comments that actually appear in the spec:
//   - rustdocs on TestingQuery fields (schema description)
//   - named keys below (parameter description)
// Path parameter keys in this attribute do not apply (Ohkami assigns path names later).
#[openapi::operation({
    summary: "Reverse both path strings and double both query integers",
    parse_path: "First path segment (string). (This doc is not shown in swagger)",
    parse_path_2: "Second path segment (string). (This doc is not shown in swagger)",
    parse_parameter: "First query integer. Required. Doubled in the JSON body.",
    parse_parameter_2: "Second query integer. Required. Doubled in the JSON body.",
    200: "JSON body with both strings reversed and both integers doubled.",
})]
async fn handle_testing(
    Path((parse_path, parse_path_2)): Path<(&str, &str)>,
    Query(query): Query<TestingQuery>,
) -> Json<TestingResponse> {
    let request = TestingRequest {
        parse_path: parse_path.to_owned(),
        parse_path_2: parse_path_2.to_owned(),
        parse_parameter: query.parse_parameter,
        parse_parameter_2: query.parse_parameter_2,
    };
    Json(TestingResponse::new(request))
}

//endregion

/// Example API plus the static Swagger UI and OpenAPI document routes.
fn manual_ohkami() -> Ohkami {
    Ohkami::new((
        "/testing/:parse_path/:parse_path_2".GET(handle_testing),
        "/".GET(redirect_swagger),
        "/swagger-ui".GET(redirect_swagger),
        "/swagger-ui/feagi-server.html".GET(swagger_html),
        "/swagger-ui/swagger-ui.css".GET(swagger_css),
        "/swagger-ui/swagger-ui-bundle.js".GET(swagger_js),
        "/api-docs/openapi.json".GET(openapi_json),
    ))
}

/// Runs the server until Ctrl+C. Ignored by default crate test runs.
#[test]
#[ignore]
fn run_manual_ohkami_server() {
    let bind = format!("127.0.0.1:{MANUAL_OHKAMI_PORT}");

    eprintln!("Manual Ohkami server listening on {bind}");
    eprintln!("Swagger UI: http://{bind}/swagger-ui/feagi-server.html");
    eprintln!("OpenAPI:    http://{bind}/api-docs/openapi.json");
    eprintln!("Stop with Ctrl+C");

    smol::block_on(async {
        manual_ohkami().howl(bind.as_str()).await;
    });
}
