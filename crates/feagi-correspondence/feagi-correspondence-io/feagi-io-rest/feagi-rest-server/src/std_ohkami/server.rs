use ohkami::prelude::*;
use ohkami::claw::{Json, Path, Query};
use ohkami::openapi;
use ohkami::serde::{Deserialize, Serialize};

// TODO review https://github.com/ohkami-rs/ohkami (openapi section)
// macro should generate request struct, response struct, async function for what should be done
// Note that ohkami has its own JSON wrapper that needs to be used
// we need to make use of parsing path parameters and url parameters
// macro generated async function should parse the object (return 400 error if invalid), call a named async func at a given ReqRes handler

/// Parsed values from `GET /testing/:parse_path/:parse_path_2?parse_parameter=&parse_parameter_2=`.
// OpenAPI: Schema derive registers field rustdocs as property descriptions.
#[derive(openapi::Schema)]
pub struct TestingRequest {
    /// First path segment (`:parse_path`).
    // OpenAPI: property description
    pub parse_path: String,
    /// Second path segment (`:parse_path_2`).
    // OpenAPI: property description
    pub parse_path_2: String,
    /// First integer query parameter (`parse_parameter`).
    // OpenAPI: property description
    pub parse_parameter: i32,
    /// Second integer query parameter (`parse_parameter_2`).
    // OpenAPI: property description
    pub parse_parameter_2: i32,
}

/// JSON body with both path strings reversed and both query integers doubled.
// OpenAPI: Schema + component; field rustdocs become property descriptions.
#[derive(Serialize, openapi::Schema)]
#[openapi(component)]
pub struct TestingResponse {
    /// First path segment, reversed.
    // OpenAPI: property description
    pub parse_path: String,
    /// Second path segment, reversed.
    // OpenAPI: property description
    pub parse_path_2: String,
    /// First query integer, doubled.
    // OpenAPI: property description
    pub parse_parameter: i32,
    /// Second query integer, doubled.
    // OpenAPI: property description
    pub parse_parameter_2: i32,
}

impl TestingResponse {
    /// Build a response from a parsed request: reverse both strings, double both numbers.
    pub fn new(request: TestingRequest) -> Self {
        Self {
            parse_path: request.parse_path.chars().rev().collect(),
            parse_path_2: request.parse_path_2.chars().rev().collect(),
            parse_parameter: request.parse_parameter * 2,
            parse_parameter_2: request.parse_parameter_2 * 2,
        }
    }
}

/// Query parameters for the testing route.
// OpenAPI: Schema derive turns these fields into query parameters.
#[derive(Deserialize, openapi::Schema)]
struct TestingQuery {
    /// First integer query parameter.
    // OpenAPI: query parameter description
    parse_parameter: i32,
    /// Second integer query parameter.
    // OpenAPI: query parameter description
    parse_parameter_2: i32,
}

/// OpenAPI document metadata used by [`generate_openapi_document`].
fn rest_openapi_metadata() -> openapi::OpenAPI<'static> {
    openapi::OpenAPI {
        title: "FEAGI REST Server",
        version: env!("CARGO_PKG_VERSION"),
        servers: &[],
    }
}

/// Reverse both path strings and double both query integers.
// Ohkami path params use `:name` (same role as `{parse_path}` in other frameworks).
// OpenAPI: operation attribute sets summary, parameter descriptions, and 200 text.
#[openapi::operation({
    summary: "Reverse both path strings and double both query integers",
    parse_path: "First path segment (string).",
    parse_path_2: "Second path segment (string).",
    parse_parameter: "First integer query parameter.",
    parse_parameter_2: "Second integer query parameter.",
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

/// Build the FEAGI REST Ohkami app. The caller owns serving it (this crate is a lib).
pub fn create_ohkami_server() -> Ohkami {
    Ohkami::new((
        "/testing/:parse_path/:parse_path_2".GET(handle_testing),
    ))
}

/// Write this server's OpenAPI document to `file_path` (JSON).
pub fn generate_openapi_document(file_path: impl AsRef<std::path::Path>) {
    create_ohkami_server().generate_to(file_path, rest_openapi_metadata());
}

// TODO keeping it now just for review...
#[cfg(test)]
mod tests {
    use super::*;

    /// Regenerates the crate-level OpenAPI document checked into this crate.
    #[test]
    fn generate_openapi_document_file() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "swagger_site/openapi.json");
        generate_openapi_document(path);
        assert!(std::path::Path::new(path).is_file());
    }
}
