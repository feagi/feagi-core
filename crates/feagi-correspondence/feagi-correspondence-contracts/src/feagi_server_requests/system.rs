#[cfg(feature = "feagi-server-requests")]
use feagi_basis::feagi_macros::request_responses::{generate_from_template_request_response_structs_and_enums, template_request_category};


#[cfg(feature = "feagi-server-requests")]
template_request_category! {
    exported_macro_name: feagi_server_requests_system,
    template: {
        category_name: "System",
        base_path: "system",
        category_description: "FEAGI Server System Level Statuses",
        read: {
            "health_check": {
                title: "HealthCheck",
                description: "Checks current status of the FEAGI server",
                path_parameters: [
                    "endpoint": String
                ],
                request: [
                    "waffles": i32
                ],
                response: [
                    "is_healthy": bool, "The current health status"
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
feagi_server_requests_system!(generate_from_template_request_response_structs_and_enums);


fn test() {
    let b: ReadResponseHealthCheck = ReadResponseHealthCheck {
        is_healthy: false,
    };
    let a = SystemResponsesEnum::HealthCheck(b);

    let d = ReadRequestHealthCheck {
        waffles: 0,
        endpoint: "".to_string(),
    };
    let c = SystemRequestsEnum::HealthCheck(d);
}

