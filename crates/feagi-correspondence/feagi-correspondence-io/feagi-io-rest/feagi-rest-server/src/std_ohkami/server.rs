
use ohkami::prelude::*;
use ohkami::openapi;
use feagi_basis::feagi_macros::request_responses::{
    generate_from_template_ohkami_rest_server,
    template_request_category,
};

// The `System` category contract. This is the single source the generator consumes.
template_request_category! {
    exported_macro_name: feagi_server_requests_system,
    template: {
        category_name: "System",
        base_path: "system",
        category_description: "System level endpoints for the FEAGI server",
        async_functions_module: system_handlers,
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
                response: ["is_healthy": bool, "whether the FEAGI server is healthy"],
            },
        },
        create: {},
        edit: {},
        delete: {},
        patch: {}
    }
}

// Ohkami structs, async handlers, and `create_system_ohkami()`.
feagi_server_requests_system!(generate_from_template_ohkami_rest_server);

/// Hand-written async functions backing each `System` endpoint.
mod system_handlers {
    use super::*;

    pub async fn read_health_check(_request: ReadHealthCheckRequest) -> ReadHealthCheckResponse {
        // Placeholder: real health evaluation goes here.
        ReadHealthCheckResponse { status: String::from("ok") }
    }

    pub async fn read_health_check2(_request: ReadHealthCheck2Request) -> ReadHealthCheck2Response {
        ReadHealthCheck2Response { is_healthy: true }
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
