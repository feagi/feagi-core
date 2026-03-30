// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Per-synapse edge flags (packed in `u8`, bit semantics).
//!
//! Stored in parallel with synapse SoA rows. Used to distinguish **associative**
//! memory-as-source STDP edges from episodic (pattern/hash) pathways — see
//! `plasticity/docs/memory-episodic-associative-design.md`.

/// Synapse created or maintained for **associative** memory plasticity (memory→downstream
/// STDP-eligible edge, real synapse on the mapping).
pub const SYNAPSE_EDGE_ASSOCIATIVE_MEMORY: u8 = 1 << 0;
