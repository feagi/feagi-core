#[cfg(feature = "feagi-server-requests")]
use feagi_basis::feagi_macros::request_responses::{generate_from_template_request_response_structs_and_enums, template_request_category};


#[cfg(feature = "feagi-server-requests")]
template_request_category! {
    exported_macro_name: feagi_server_requests_agent,
    template: {
        category_name: "Agent",
        base_path: "agent",
        category_description: "Endpoint for handling Agent Registration and heartbeat",
        async_functions_module: agent,
        read: {
            "list": {
                title: "List",
                description: "gets a list of all registered agent IDs",
                path_parameters: [],
                request: [],
                response: [
                    "agents": String, "All Agents",
                ],
            }
        },
        create: {},
        edit: {},
        delete: {},
        patch: {}
    }
}

#[cfg(feature = "feagi-server-requests")]
feagi_server_requests_agent!(generate_from_template_request_response_structs_and_enums);