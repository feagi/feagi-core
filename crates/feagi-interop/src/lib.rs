#[cfg(feature = "alloc")]
extern crate alloc;

pub mod embodiment;

pub mod error;

pub mod interop_config;
pub mod interop_peer;
pub mod interop_request_requester;
pub mod interop_request_responder;
pub mod transport;
pub mod reference_endpoints;
pub mod node;
