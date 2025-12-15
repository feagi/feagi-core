// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Transport-agnostic type aliases for endpoints
//!
//! These types allow endpoints to work with both HTTP (axum) and WASM transports
//! without modification.

#[cfg(feature = "http")]
pub use axum::extract::{State, Query};
#[cfg(feature = "http")]
pub use axum::response::Json;

#[cfg(not(feature = "http"))]
pub use crate::transports::wasm::types::{State, Json};

// Query extractor for WASM (simple wrapper around HashMap)
#[cfg(not(feature = "http"))]
pub struct Query<T>(pub T);

#[cfg(not(feature = "http"))]
impl<T> std::ops::Deref for Query<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(not(feature = "http"))]
impl<T> Query<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
    
    pub fn into_inner(self) -> T {
        self.0
    }
}
