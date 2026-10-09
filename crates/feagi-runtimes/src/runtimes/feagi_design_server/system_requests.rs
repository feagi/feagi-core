use feagi_basis::feagi_basis_proc_common::{define_template_categorized_requests, from_template_make_request_message_structs};

define_template_categorized_requests! {
    exported_macro_name: template_design_server_system_requests,
    template: {
        category_name: "System",
        base_path: "system",
        category_description: "FEAGI Server System Level Statuses",
        read: {
            "SystemHealth": {
                path: "health",
                path_item_descriptions: {},
                description: "Get System Health Status",
                response: {
                    feagi_running: bool, "Is FEAGI running?",
                }
            }
        }
    }
}

template_design_server_system_requests!(from_template_make_request_message_structs);




