// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Platform-agnostic async runtime abstraction for FEAGI
//!
//! This module provides a trait-based abstraction over different async runtimes:
//! - Tokio (desktop/server)
//! - WASM (browser/web)
//! - WASI (WebAssembly System Interface)
//!
//! Use this module to write async code that works across all platforms.

mod feagi_async_runtime;
mod feagi_runtimes;
mod main_entry_macro;
mod run_async_macro;

pub use feagi_async_runtime::{BlockOnError, FeagiAsyncRuntime, TimeoutError};
pub use feagi_runtimes::*;
