// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WASM Transport Adapter for FEAGI API
//!
//! This adapter enables REST API calls in WASM environments by routing requests
//! to the same transport-agnostic endpoint functions used by HTTP and ZMQ adapters.
//!
//! # Architecture
//!
//! ```text
//! JavaScript/TypeScript (Browser)
//!     ↓
//! FeagiEngine.handle_rest_api_call()
//!     ↓
//! WasmApiAdapter.handle_request()
//!     ↓
//! endpoints::* (same as HTTP/ZMQ)
//!     ↓
//! Wasm*Service (extracts from RuntimeGenome)
//! ```
//!
//! # Usage
//!
//! ```rust
//! use feagi_api::transports::wasm::WasmApiAdapter;
//! use feagi_api::transports::http::server::ApiState;
//!
//! let api_state = create_api_state_from_genome(genome)?;
//! let adapter = WasmApiAdapter::new(api_state);
//!
//! let response = adapter.handle_request("GET", "/v1/system/health_check", None).await?;
//! ```

pub mod adapter;
pub mod services;
pub mod state;
#[cfg(not(feature = "http"))]
pub mod types;

pub use adapter::WasmApiAdapter;
pub use state::create_api_state_from_genome;

