// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Hash state storage for change detection.
//!
//! Stores event-driven data hashes so health_check can read them without recomputation.

use core::sync::atomic::{AtomicU64, Ordering};

/// Atomic storage for FEAGI data hashes (event-driven updates).
#[derive(Debug, Default)]
pub struct HashState {
    brain_regions_hash: AtomicU64,
    cortical_areas_hash: AtomicU64,
    brain_geometry_hash: AtomicU64,
    morphologies_hash: AtomicU64,
    cortical_mappings_hash: AtomicU64,
    agent_data_hash: AtomicU64,
}

impl HashState {
    /// Create a new hash state with zeroed hashes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get brain regions hash.
    pub fn get_brain_regions_hash(&self) -> u64 {
        self.brain_regions_hash.load(Ordering::Acquire)
    }

    /// Set brain regions hash.
    pub fn set_brain_regions_hash(&self, value: u64) {
        self.brain_regions_hash.store(value, Ordering::Release);
    }

    /// Get cortical areas hash.
    pub fn get_cortical_areas_hash(&self) -> u64 {
        self.cortical_areas_hash.load(Ordering::Acquire)
    }

    /// Set cortical areas hash.
    pub fn set_cortical_areas_hash(&self, value: u64) {
        self.cortical_areas_hash.store(value, Ordering::Release);
    }

    /// Get brain geometry hash.
    pub fn get_brain_geometry_hash(&self) -> u64 {
        self.brain_geometry_hash.load(Ordering::Acquire)
    }

    /// Set brain geometry hash.
    pub fn set_brain_geometry_hash(&self, value: u64) {
        self.brain_geometry_hash.store(value, Ordering::Release);
    }

    /// Get morphologies hash.
    pub fn get_morphologies_hash(&self) -> u64 {
        self.morphologies_hash.load(Ordering::Acquire)
    }

    /// Set morphologies hash.
    pub fn set_morphologies_hash(&self, value: u64) {
        self.morphologies_hash.store(value, Ordering::Release);
    }

    /// Get cortical mappings hash.
    pub fn get_cortical_mappings_hash(&self) -> u64 {
        self.cortical_mappings_hash.load(Ordering::Acquire)
    }

    /// Set cortical mappings hash.
    pub fn set_cortical_mappings_hash(&self, value: u64) {
        self.cortical_mappings_hash.store(value, Ordering::Release);
    }

    /// Get agent data hash.
    pub fn get_agent_data_hash(&self) -> u64 {
        self.agent_data_hash.load(Ordering::Acquire)
    }

    /// Set agent data hash.
    pub fn set_agent_data_hash(&self, value: u64) {
        self.agent_data_hash.store(value, Ordering::Release);
    }
}
