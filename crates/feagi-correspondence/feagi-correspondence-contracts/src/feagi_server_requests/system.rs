#[cfg(feature = "feagi-server-requests")]
use feagi_basis::feagi_macros::request_responses::{generate_from_template_request_response_structs_and_enums, template_request_category};


#[cfg(feature = "feagi-server-requests")]
template_request_category! {
    exported_macro_name: feagi_server_requests_system,
    template: {
        category_name: "System",
        base_path: "system",
        read: {
            "health_check": {
                title: "HealthCheck",
                description: "Checks current status of the FEAGI server",
                path_parameters: [],
                request: [],
                response: ["is_healthy": bool, "the current health status"],
            }
        },
        create: {},
        edit: {},
        delete: {},
        patch: {}
    }
}

#[cfg(feature = "feagi-server-requests")]
feagi_server_requests_system!(generate_from_template_request_response_structs_and_enums);




