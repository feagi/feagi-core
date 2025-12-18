// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! # FEAGI Runtime Abstraction
//!
//! Cross-platform runtime traits for neural processing.
//!
//! This crate defines the **abstraction layer** that enables FEAGI to run on different platforms:
//! - Desktop/Server: `feagi-runtime-std` (Vec-based, dynamic)
//! - Embedded: `feagi-runtime-embedded` (fixed arrays, no_std)
//! - GPU: `feagi-runtime-cuda` (GPU VRAM, CUDA)
//! - WASM: `feagi-runtime-wasm` (typed arrays, browser)
//!
//! ## Architecture
//!
//! ```text
//! ┌────────────────────────────────────────────────┐
//! │ feagi-burst-engine<R: Runtime>                 │
//! │   - Generic burst processing                   │
//! │   - Works with any Runtime                     │
//! └────────────────────┬───────────────────────────┘
//!                      │ uses
//! ┌────────────────────▼───────────────────────────┐
//! │ feagi-runtime (THIS CRATE)                     │
//! │   - Runtime trait                              │
//! │   - NeuronStorage trait                        │
//! │   - SynapseStorage trait                       │
//! └────────────────────┬───────────────────────────┘
//!                      │ implemented by
//! ┌────────────────────▼───────────────────────────┐
//! │ feagi-runtime-std / embedded / cuda / wasm     │
//! │   - Platform-specific storage                  │
//! └────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! This crate is typically not used directly. Instead, you use a concrete runtime:
//!
//! ```ignore
//! use feagi_npu_runtime_std::StdRuntime;  // For desktop
//! use feagi_npu_runtime_embedded::EmbeddedRuntime;  // For ESP32
//! use feagi_runtime_cuda::CudaRuntime;  // For GPU
//! ```
//!
//! ## Implementing a New Runtime
//!
//! To add support for a new platform:
//!
//! 1. Create a new crate (e.g., `feagi-runtime-myplatform`)
//! 2. Implement `NeuronStorage` and `SynapseStorage` traits
//! 3. Implement `Runtime` trait
//! 4. Test with `feagi-burst-engine`
//!
//! See `feagi-runtime-std` for a reference implementation.

#![no_std]
#![warn(missing_docs)]

/// Crate version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "std")]
extern crate std;

pub mod error;
pub mod traits;

// Re-export key types
pub use error::{Result, RuntimeError};
pub use traits::{NeuronStorage, Runtime, SynapseStorage};

// Re-export NeuralValue from feagi-neural
pub use feagi_npu_neural::types::NeuralValue;

/// Version of the runtime trait API
///
/// Increment this when making breaking changes to the trait API.
pub const RUNTIME_TRAIT_VERSION: u32 = 1;

/// Get the runtime trait API version
pub const fn runtime_trait_version() -> u32 {
    RUNTIME_TRAIT_VERSION
}
