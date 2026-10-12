pub mod error;

pub mod server;

// TODO seperate swagger and openapi features
#[cfg(feature = "transport_rest_swagger")]
pub mod swagger;
#[cfg(feature = "transport_rest_swagger")]
pub mod open_api;
