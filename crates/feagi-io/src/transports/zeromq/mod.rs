// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Async-native ZeroMQ transport spike (pure Rust).
//!
//! This module is intentionally minimal and isolated behind the `zeromq-transport`
//! feature flag. It provides a compilation boundary for the `zeromq` crate while
//! we validate parity with the existing `zmq` (libzmq) transport.
//!
//! Do not wire this into production paths until parity verification is complete.

/// Placeholder marker for the zeromq transport spike.
///
/// This is intentionally minimal to avoid behavior changes before
/// a full parity review is approved.
#[derive(Debug, Default, Clone, Copy)]
pub struct ZeromqTransportSpike;
