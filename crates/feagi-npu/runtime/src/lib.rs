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
//! Cross-platform runtime traits and implementations for neural processing.
//!
//! This crate provides:
//! - **Traits** (always available): `Runtime`, `NeuronStorage`, `SynapseStorage`
//! - **Std Implementation** (behind `std` feature): `StdRuntime` for desktop/server
//! - **Embedded Implementation** (behind `embedded` feature): `EmbeddedRuntime` for no_std
//!
//! ## Features
//!
//! - `default` = `[]` (traits only, no_std compatible)
//! - `std` = Standard library runtime implementation (Vec-based, parallel)
//! - `embedded` = Embedded runtime implementation (fixed arrays, no_std)
//! - `alloc` = Heap allocation without std (for some trait methods)
//!
//! ## Usage
//!
//! ### Desktop/Server
//! ```toml
//! [dependencies]
//! feagi-npu-runtime = { version = "2.0", features = ["std"] }
//! ```
//!
//! ```rust
//! use feagi_npu_runtime::StdRuntime;
//! let runtime = StdRuntime::new();
//! ```
//!
//! ### Embedded
//! ```toml
//! [dependencies]
//! feagi-npu-runtime = { version = "2.0", features = ["embedded"], default-features = false }
//! ```
//!
//! ```rust
//! use feagi_npu_runtime::embedded::EmbeddedRuntime;
//! let runtime = EmbeddedRuntime::new();
//! ```
//!
//! ### Traits Only (for custom implementations)
//! ```toml
//! [dependencies]
//! feagi-npu-runtime = { version = "2.0", default-features = false }
//! ```
//!
//! ```rust
//! use feagi_npu_runtime::{Runtime, NeuronStorage, SynapseStorage};
//! // Implement your own runtime
//! ```

#![no_std]
#![warn(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

/// Crate version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Traits module (always available)
pub mod traits;

// Re-export traits for convenience
pub use traits::{NeuralValue, NeuronStorage, Result, Runtime, RuntimeError, SynapseStorage};

// Standard library implementation (behind "std" feature)
#[cfg(feature = "std")]
pub mod std_impl;

// Re-export std module contents for convenience (backward compatibility)
#[cfg(feature = "std")]
pub use std_impl::{NeuronArray as StdNeuronArray, StdRuntime, SynapseArray as StdSynapseArray};

// Embedded implementation (behind "embedded" feature)
#[cfg(feature = "embedded")]
pub mod embedded_impl;

// Re-export embedded module contents for convenience
#[cfg(feature = "embedded")]
pub use embedded_impl::{
    EmbeddedRuntime, NeuronArray as EmbeddedNeuronArray, SynapseArray as EmbeddedSynapseArray,
};

// Convenience module for embedded (re-exports from embedded_impl)
/// Embedded runtime implementations for no_std environments
#[cfg(feature = "embedded")]
pub mod embedded {
    pub use super::embedded_impl::{EmbeddedRuntime, NeuronArray, SynapseArray};
}

/// Version of the runtime trait API
///
/// Increment this when making breaking changes to the trait API.
pub const RUNTIME_TRAIT_VERSION: u32 = 1;

/// Get the runtime trait API version
pub const fn runtime_trait_version() -> u32 {
    RUNTIME_TRAIT_VERSION
}
