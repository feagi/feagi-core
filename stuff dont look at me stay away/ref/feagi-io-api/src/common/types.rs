// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Transport-agnostic type aliases for endpoints
//!
//! These types allow endpoints to work with both HTTP (axum) and WASM transports
//! without modification.

#[cfg(feature = "http")]
pub use crate::transports::http::server::ApiState;
#[cfg(feature = "http")]
pub use axum::extract::{Path, Query, State};
#[cfg(feature = "http")]
pub use axum::response::Json;

#[cfg(not(feature = "http"))]
pub use crate::transports::http::server::ApiState;
#[cfg(not(feature = "http"))]
pub use crate::transports::wasm::types::{Json, Path, Query, State};
