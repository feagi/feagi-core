//! FEAGI REST server (Ohkami) for the `System` request/response category.
//!
//! The request/response structs, the shared request/response enums, the Ohkami-facing
//! structs, their `From` conversions, one async handler per endpoint, and the routing
//! builder are all generated from a single template below:
//! - `generate_from_template_request_response_structs_and_enums!` emits the shared
//!   `System{Requests,Responses}Enum` and the `Read*` structs.
//! - `generate_from_template_ohkami_rest_server!` emits the Ohkami structs, handlers,
//!   and `create_system_ohkami()`.
//!
//! Each generated handler decodes the request, wraps it into `SystemRequestsEnum`, calls
//! the hand-written dispatch function [`handle_system`], and maps the returned
//! `SystemResponsesEnum` back into an Ohkami response.

use ohkami::prelude::*;
use ohkami::openapi;
use feagi_basis::feagi_macros::request_responses::{
    generate_from_template_ohkami_rest_server,
    generate_from_template_request_response_structs_and_enums,
    template_request_category,
};

// The `System` category contract. This is the single source the generators consume.
template_request_category! {
    exported_macro_name: feagi_server_requests_system,
    template: {
        category_name: "System",
        base_path: "system",
        category_description: "System level endpoints for the FEAGI server",
        read: {
            "health_check": {
                title: "HealthCheck",
                description: "Checks current status of the FEAGI server",
                path_parameters: [],
                request: [],
                response: ["status": String, "the current health status"],
            },
            "health_check2": {
                title: "HealthCheck2",
                description: "Checks current status of thdfgdfgdfge FEAGI server 2",
                path_parameters: [],
                request: [],
                response: ["status": String, "the current health status 2"],
            },
        },
        create: {},
        edit: {},
        delete: {},
        patch: {}
    }
}

// Shared request/response enums + `Read*` structs.
feagi_server_requests_system!(generate_from_template_request_response_structs_and_enums);

// Ohkami structs, `From` conversions, async handlers, and `create_system_ohkami()`.
feagi_server_requests_system!(generate_from_template_ohkami_rest_server);

/// Hand-written dispatch for the `System` category.
///
/// The generated handlers await this with the decoded request enum and expect the matching
/// response enum variant back. This is currently a placeholder returning canned data.
async fn handle_system(request: SystemRequestsEnum) -> SystemResponsesEnum {
    match request {
        SystemRequestsEnum::HealthCheck() => {
            // Placeholder: real health evaluation goes here.
            SystemResponsesEnum::HealthCheck(ReadResponseHealthCheck { status: String::from("ok") })
        }
        SystemRequestsEnum::HealthCheck2() => {
            SystemResponsesEnum::HealthCheck2(ReadResponseHealthCheck2 { status: String::from("ok") })
        }
    }
}

/// OpenAPI document metadata used by [`generate_openapi_document`].
fn rest_openapi_metadata() -> openapi::OpenAPI<'static> {
    openapi::OpenAPI {
        title: "FEAGI REST Server",
        version: env!("CARGO_PKG_VERSION"),
        servers: &[],
    }
}

/// Build the FEAGI REST Ohkami app. The caller owns serving it (this crate is a lib).
pub fn create_ohkami_server() -> Ohkami {
    create_system_ohkami()
}

/// Write this server's OpenAPI document to `file_path` (JSON).
pub fn generate_openapi_document(file_path: impl AsRef<std::path::Path>) {
    create_ohkami_server().generate_to(file_path, rest_openapi_metadata());
}
