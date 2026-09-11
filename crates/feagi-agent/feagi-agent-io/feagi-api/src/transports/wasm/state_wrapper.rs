// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WASM-compatible State wrapper
//!
//! Mimics axum::extract::State for WASM builds without requiring axum dependency.

use crate::transports::http::server::ApiState;

/// WASM-compatible State wrapper
///
/// This mimics `axum::extract::State<ApiState>` for WASM builds
/// where axum is not available.
#[derive(Clone)]
pub struct WasmState(pub ApiState);

impl WasmState {
    pub fn new(state: ApiState) -> Self {
        Self(state)
    }

    pub fn into_inner(self) -> ApiState {
        self.0
    }

    pub fn inner(&self) -> &ApiState {
        &self.0
    }
}

// Implement Deref for convenience
impl std::ops::Deref for WasmState {
    type Target = ApiState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


