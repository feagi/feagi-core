// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Transport-agnostic type aliases for endpoints
//!
//! These types allow endpoints to work with both HTTP (axum) and WASM transports
//! without modification.

#[cfg(feature = "http")]
pub use axum::extract::{State, Query, Path};
#[cfg(feature = "http")]
pub use axum::response::Json;
#[cfg(feature = "http")]
pub use crate::transports::http::server::ApiState;

#[cfg(not(feature = "http"))]
pub use crate::transports::wasm::types::{State, Json, Query, Path};
// ApiState is always available (defined in http/server.rs but doesn't require http feature)
pub use crate::transports::http::server::ApiState;
