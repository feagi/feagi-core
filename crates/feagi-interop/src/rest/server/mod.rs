pub mod error;

#[cfg(feature = "std")]
pub mod server_std;

// TODO seperate swagger and openapi features
#[cfg(feature = "open_api_endpoint")]
pub mod open_api;
#[cfg(feature = "swagger_web_ui")]
pub mod swagger;
pub mod server_common;
