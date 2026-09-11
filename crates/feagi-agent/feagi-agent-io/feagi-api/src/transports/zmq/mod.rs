// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! ZMQ Transport Module
//!
//! Provides ZMQ-based control plane for the FEAGI API using feagi-io transport primitives.

pub mod adapter;

pub use adapter::ZmqApiAdapter;
