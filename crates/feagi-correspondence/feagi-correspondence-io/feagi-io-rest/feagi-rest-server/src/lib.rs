#[cfg(feature = "ohkami")]
mod std_ohkami;

#[cfg(feature = "ohkami")]
pub use std_ohkami::server::{
    create_ohkami_server, generate_openapi_document, TestingRequest, TestingResponse,
};
