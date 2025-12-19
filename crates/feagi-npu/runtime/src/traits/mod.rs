// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Runtime abstraction traits for cross-platform neural processing
//!
//! This module defines the core traits that enable FEAGI to run on different platforms:
//! - Desktop/Server (Vec-based, dynamic allocation)
//! - Embedded (fixed arrays, no_std)
//! - GPU (CUDA VRAM, GPU memory)
//! - WASM (WebAssembly.Memory, typed arrays)
//!
//! ## Design Philosophy
//!
//! - **Storage Abstraction**: Separate "what" from "how" (types vs storage)
//! - **Zero-Cost**: Traits compile to direct calls (no runtime overhead)
//! - **Platform-Agnostic**: Same burst engine code works everywhere
//! - **Type-Safe**: Compile-time guarantees for platform compatibility

pub mod runtime;
pub mod error;

// Re-export key types
pub use runtime::{NeuronStorage, Runtime, SynapseStorage};
pub use error::{Result, RuntimeError};

// Re-export NeuralValue from feagi-neural
pub use feagi_npu_neural::types::NeuralValue;

