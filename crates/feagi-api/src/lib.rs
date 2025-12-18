// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// FEAGI REST API Layer
//
// This crate provides a unified, transport-agnostic API layer for FEAGI.
// It supports both HTTP (Axum) and ZMQ transports using a shared endpoint layer.

/// Crate version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod common;
pub mod endpoints;
#[cfg(feature = "http")]
pub mod middleware;
#[cfg(feature = "http")]
pub mod openapi;
pub mod security;
pub mod transports;
pub mod v1;
pub mod v2;

// Re-export commonly used types
pub use common::{ApiError, ApiRequest, ApiResponse, EmptyResponse};
pub use security::{AuthContext, Permission};
