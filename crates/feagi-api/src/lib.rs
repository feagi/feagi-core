// FEAGI REST API Layer
//
// This crate provides a unified, transport-agnostic API layer for FEAGI.
// It supports both HTTP (Axum) and ZMQ transports using a shared endpoint layer.

pub mod common;
pub mod endpoints;
pub mod middleware;
pub mod security;
pub mod transports;
pub mod v1;
pub mod v2;

// Re-export commonly used types
pub use common::{ApiError, ApiRequest, ApiResponse, EmptyResponse};
pub use security::{AuthContext, Permission};

