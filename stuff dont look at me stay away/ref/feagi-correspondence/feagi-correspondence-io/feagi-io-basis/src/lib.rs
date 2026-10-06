/// Common traits for all endpoints, and for launching as a FEAGI Endpoint
pub mod feagi_server_endpoint_and_config;

/// `create_request_response_sender_receiver_sets` expands to `paste` in the caller.
#[doc(hidden)]
pub use paste;

pub mod contracts;




