pub mod error;

pub mod server;

// TODO seperate swagger and openapi features
#[cfg(feature = "swagger_web_ui")]
pub mod open_api;
#[cfg(feature = "swagger_web_ui")]
pub mod swagger;
