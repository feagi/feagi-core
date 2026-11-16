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

/// Synapse inserted at runtime by `memory.capture` recording: the episode link from one encoded
/// state of a memory area to the next (the area's `sequential_memory` loop). The unordered STDP
/// pass never touches it and a mapping edit spares it; see `burst-engine/src/memory_capture.rs`.
pub const SYNAPSE_EDGE_SEQUENTIAL_MEMORY: u8 = 1 << 1;

/// Bit-to-name table for decoding a packed edge-flag byte in API output (read-only; add a row
/// when a new `SYNAPSE_EDGE_*` bit is defined).
pub const SYNAPSE_EDGE_FLAG_NAMES: &[(u8, &str)] = &[
    (SYNAPSE_EDGE_ASSOCIATIVE_MEMORY, "associative_memory"),
    (SYNAPSE_EDGE_SEQUENTIAL_MEMORY, "sequential_memory"),
];
