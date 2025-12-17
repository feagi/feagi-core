// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// Transport adapters for HTTP, ZMQ, and WASM

// http module is always available (for ApiState), but server/router code is conditional
#[cfg(feature = "http")]
pub mod http;
#[cfg(not(feature = "http"))]
pub mod http {
    // Minimal http module for ApiState when http feature is disabled
    pub mod server;
}
#[cfg(feature = "zmq")]
pub mod zmq;
#[cfg(not(feature = "http"))]
pub mod wasm;


