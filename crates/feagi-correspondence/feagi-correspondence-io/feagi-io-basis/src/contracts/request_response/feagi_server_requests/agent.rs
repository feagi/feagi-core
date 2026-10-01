
use feagi_basis::feagi_macros::request_responses::{generate_from_template_request_response_structs_and_enums, template_request_category};
use feagi_basis::prelude::{generate_feagi_error, FeagiFail, FeagiErrorTrait, FeagiFailTrait, FeagiError};
use feagi_basis::thread_messaging::multi_request_channel::alloc_requester_processor::{create_requester_and_responder, RequestResponder, PooledOneshotRequester};


template_request_category! {
    exported_macro_name: feagi_server_requests_agent,
    template: {
        category_name: "Agent",
        base_path: "agent",
        category_description: "Endpoint for handling Agent Registration and heartbeat",
        feagi_error_type: FeagiRequestReceiveAgentError,
        read: {
            "list": {
                title: "List",
                description: "gets a list of all registered agent IDs",
                path_parameters: [],
                request: [],
                response: [
                    "agents": String, "List of all agents connected",
                ],
            }
        },
        create: {},
        edit: {},
        delete: {},
        patch: {}
    }
}


feagi_server_requests_agent!(generate_from_template_request_response_structs_and_enums);


generate_feagi_error! {
    /// The root error for the Basis crate
    FeagiRequestReceiveAgentError,
    keys: {
        Etc: FeagiFailEtc,
    },
    sub_errors: {

    },
}

/// A Genric error type. Anything using this should be updated with more specific errors
#[derive(FeagiFail)]
pub struct FeagiFailEtc {
    context: &'static str,
}

impl Default for FeagiRequestReceiveAgentError {
    fn default() -> Self {
        Self::Etc(FeagiFailEtc::new("Default"))
    }
}
