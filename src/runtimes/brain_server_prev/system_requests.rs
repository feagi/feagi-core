use feagi_basis::feagi_basis_proc_common::{
    define_template_categorized_requests, from_template_make_request_message_categorized_enum, from_template_make_request_message_structs,
};
use feagi_basis::feagi_message::feagi_message::FeagiMessage;
use feagi_basis::feagi_message::feagi_message_path::{FeagiMessagePath, MessagePathElement};
use feagi_basis::feagi_message::request_response::feagi_request_message::{FeagiMessageRequestMethod, FeagiRequestMessage};
use feagi_basis::feagi_message::request_response::feagi_request_message_path::FeagiRequestMessagePath;
use feagi_basis::feagi_message::request_response::feagi_request_message_specifications::FeagiMessageResponse;
use serde::{Deserialize, Serialize};

define_template_categorized_requests! {
    exported_macro_name: template_runtime_system_requests,
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

template_runtime_system_requests!(from_template_make_request_message_structs);

template_runtime_system_requests!(from_template_make_request_message_categorized_enum);
