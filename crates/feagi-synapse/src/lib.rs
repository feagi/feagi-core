/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! # FEAGI Synaptic Computation (Platform-Agnostic)
//!
//! Pure synaptic algorithms that work on any platform:
//! - Desktop (std)
//! - ESP32 (no_std)
//! - HPC clusters (std + MPI)
//! - GPU (WGPU/CUDA)
//!
//! ## Design Principles
//! - **No allocations**: All functions work on borrowed slices
//! - **No I/O**: Pure computation only
//! - **No platform dependencies**: Works with `no_std`
//! - **SIMD-friendly**: Vectorizable operations
//!
//! ## Target Platforms
//! - ✅ Desktop (Linux, macOS, Windows)
//! - ✅ Embedded (ESP32, ARM Cortex-M)
//! - ✅ RTOS (FreeRTOS, Zephyr)
//! - ✅ WASM (browser, Node.js)

#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod contribution;
pub mod weight;

pub use contribution::*;
pub use weight::*;


