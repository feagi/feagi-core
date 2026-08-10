// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! The NPU capabilities the API services depend on.
//!
//! The services must not depend on a concrete engine: `feagi-api` sits below the application in
//! the dependency graph, and the engine lives above it. This trait inverts that — the application
//! (`feagi-rs`) implements it over its own NPU handle and injects it, so the services can drive
//! the running brain without `feagi-api` knowing which engine is behind it.
//!
//! Only operations the current engine genuinely supports appear here. Anything absent is reported
//! by the services as unavailable rather than being faked, so the trait doubles as an explicit
//! record of what the engine can and cannot do today.

use feagi_genomic_context::cortical_area::CorticalID;

/// A cortical area as the engine holds it.
#[derive(Debug, Clone)]
pub struct NpuCorticalArea {
    pub id: CorticalID,
    /// Voxel extents plus per-voxel neuron density: `[x, y, z, density]`.
    pub dimensions: [u64; 4],
    /// `x * y * z * density`.
    pub neuron_count: u64,
}

/// Read and control access to the running NPU.
///
/// Implementations are shared across request handlers, so every method takes `&self` and must be
/// safe to call concurrently.
pub trait NpuAccess: Send + Sync {
    /// Every cortical area currently realised in the engine.
    fn cortical_areas(&self) -> Vec<NpuCorticalArea>;

    /// One cortical area, or `None` when the engine does not hold it.
    fn cortical_area(&self, id: &CorticalID) -> Option<NpuCorticalArea>;

    /// Creates a dimensional cortical area holding `x * y * z * density` neurons.
    ///
    /// Returns a human-readable message on rejection (duplicate id, zero or unrepresentable
    /// dimensions) so the service can surface it without knowing the engine's error type.
    fn add_cortical_area(&self, id: CorticalID, x: u64, y: u64, z: u64, density: u64) -> Result<NpuCorticalArea, String>;

    /// Bursts completed since the engine started.
    fn burst_count(&self) -> u64;

    /// Configured burst frequency in hertz.
    fn burst_hz(&self) -> u64;

    /// Sets the burst frequency, returning a message when the value is rejected.
    fn set_burst_hz(&self, hz: u64) -> Result<(), String>;

    /// Whether the burst loop is currently running.
    fn is_running(&self) -> bool;

    /// Starts the burst loop. `false` when it was already running.
    fn start(&self) -> bool;

    /// Stops the burst loop. `false` when it was not running.
    fn stop(&self) -> bool;

    /// Runs exactly one burst and returns the new burst count.
    ///
    /// Used by the single-step control; callers are expected to have stopped the loop first.
    fn step_once(&self) -> u64;
}
