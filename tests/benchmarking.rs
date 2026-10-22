// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Benchmarking integration tests.
//!
//! Each module under `benchmarking/` holds one or more tests that measure
//! performance or timing characteristics (e.g. burst engine jitter).

#[path = "benchmarking/burst_engine_jitter.rs"]
mod burst_engine_jitter;
