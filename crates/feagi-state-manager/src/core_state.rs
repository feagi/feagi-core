// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Memory-mapped core state with lock-free atomic operations
//!
//! This module provides a 64-byte cache-line aligned atomic state structure
//! for ultra-fast cross-thread/cross-process state access (5-20ns reads).

use atomic_polyfill::{AtomicU32, AtomicU64, AtomicU8, Ordering};

/// Burst engine state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BurstEngineState {
    Unavailable = 0,
    Initializing = 1,
    Ready = 2,
    Running = 3,
    Paused = 4,
    Error = 5,
    LightSleep = 6,
    DeepSleep = 7,
}

impl From<u8> for BurstEngineState {
    fn from(value: u8) -> Self {
        match value {
            1 => BurstEngineState::Initializing,
            2 => BurstEngineState::Ready,
            3 => BurstEngineState::Running,
            4 => BurstEngineState::Paused,
            5 => BurstEngineState::Error,
            6 => BurstEngineState::LightSleep,
            7 => BurstEngineState::DeepSleep,
            _ => BurstEngineState::Unavailable,
        }
    }
}

/// Genome state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GenomeState {
    Missing = 0,
    Loading = 1,
    Loaded = 2,
    Saving = 3,
    Error = 4,
}

impl From<u8> for GenomeState {
    fn from(value: u8) -> Self {
        match value {
            1 => GenomeState::Loading,
            2 => GenomeState::Loaded,
            3 => GenomeState::Saving,
            4 => GenomeState::Error,
            _ => GenomeState::Missing,
        }
    }
}

/// Connectome state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ConnectomeState {
    Missing = 0,
    Initializing = 1,
    Updating = 2,
    Ready = 3,
    Snapshotting = 4,
    Error = 5,
}

impl From<u8> for ConnectomeState {
    fn from(value: u8) -> Self {
        match value {
            1 => ConnectomeState::Initializing,
            2 => ConnectomeState::Updating,
            3 => ConnectomeState::Ready,
            4 => ConnectomeState::Snapshotting,
            5 => ConnectomeState::Error,
            _ => ConnectomeState::Missing,
        }
    }
}

/// Service state (API, ZMQ, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ServiceState {
    Unavailable = 0,
    Initializing = 1,
    Ready = 2,
    Degraded = 3,
    Error = 4,
}

impl From<u8> for ServiceState {
    fn from(value: u8) -> Self {
        match value {
            1 => ServiceState::Initializing,
            2 => ServiceState::Ready,
            3 => ServiceState::Degraded,
            4 => ServiceState::Error,
            _ => ServiceState::Unavailable,
        }
    }
}

/// Memory-mapped core state (128 bytes, cache-line aligned)
#[repr(C, align(64))]
pub struct MemoryMappedState {
    // Service states (8 bytes)
    pub genome_state: AtomicU8,
    pub connectome_state: AtomicU8,
    pub burst_engine_state: AtomicU8,
    pub fq_sampler_state: AtomicU8,
    pub api_state: AtomicU8,
    pub zmq_state: AtomicU8,
    pub brain_readiness: AtomicU8,
    pub _reserved1: AtomicU8,

    // Counters (8 bytes)
    pub agent_count: AtomicU32,
    pub burst_frequency: AtomicU32, // f32 as u32 bits

    // Statistics (24 bytes)
    pub neuron_count: AtomicU32,
    pub synapse_count: AtomicU32,
    pub cortical_area_count: AtomicU32,
    pub memory_usage: AtomicU32,
    pub regular_neuron_count: AtomicU32,
    pub memory_neuron_count: AtomicU32,
    
    // Capacity (static values set at initialization, never change)
    pub neuron_capacity: AtomicU32,
    pub synapse_capacity: AtomicU32,

    // Versioning & timestamps (16 bytes)
    pub version: AtomicU64,
    pub last_modified: AtomicU64,

    // Genome tracking (8 bytes)
    pub genome_timestamp: AtomicU64,

    // Fatigue state (6 bytes)
    pub fatigue_index: AtomicU8,        // 0-100
    pub fatigue_active: AtomicU8,        // 0=false, 1=true
    pub regular_neuron_util: AtomicU8,  // 0-100
    pub memory_neuron_util: AtomicU8,    // 0-100
    pub synapse_util: AtomicU8,         // 0-100
    pub _reserved_fatigue: AtomicU8,     // Reserved for future use

    // Padding to 128 bytes (66 bytes) - reduced by 16 bytes for capacity and neuron count fields
    pub _padding: [u8; 66],
}

impl MemoryMappedState {
    /// Create a new state with default values
    pub fn new() -> Self {
        Self {
            genome_state: AtomicU8::new(GenomeState::Missing as u8),
            connectome_state: AtomicU8::new(ConnectomeState::Missing as u8),
            burst_engine_state: AtomicU8::new(BurstEngineState::Unavailable as u8),
            fq_sampler_state: AtomicU8::new(ServiceState::Unavailable as u8),
            api_state: AtomicU8::new(ServiceState::Unavailable as u8),
            zmq_state: AtomicU8::new(ServiceState::Unavailable as u8),
            brain_readiness: AtomicU8::new(0),
            _reserved1: AtomicU8::new(0),

            agent_count: AtomicU32::new(0),
            burst_frequency: AtomicU32::new(0),

            neuron_count: AtomicU32::new(0),
            synapse_count: AtomicU32::new(0),
            cortical_area_count: AtomicU32::new(0),
            memory_usage: AtomicU32::new(0),
            regular_neuron_count: AtomicU32::new(0),
            memory_neuron_count: AtomicU32::new(0),
            neuron_capacity: AtomicU32::new(0),
            synapse_capacity: AtomicU32::new(0),

            version: AtomicU64::new(0),
            last_modified: AtomicU64::new(0),
            genome_timestamp: AtomicU64::new(0),

            fatigue_index: AtomicU8::new(0),
            fatigue_active: AtomicU8::new(0),
            regular_neuron_util: AtomicU8::new(0),
            memory_neuron_util: AtomicU8::new(0),
            synapse_util: AtomicU8::new(0),
            _reserved_fatigue: AtomicU8::new(0),

            _padding: [0; 66],
        }
    }

    // ===== Lock-Free State Accessors (5-20ns reads) =====

    /// Get burst engine state (atomic read)
    pub fn get_burst_engine_state(&self) -> BurstEngineState {
        BurstEngineState::from(self.burst_engine_state.load(Ordering::Acquire))
    }

    /// Set burst engine state (atomic write)
    pub fn set_burst_engine_state(&self, state: BurstEngineState) {
        self.burst_engine_state
            .store(state as u8, Ordering::Release);
        self.increment_version();
    }

    /// Get genome state (atomic read)
    pub fn get_genome_state(&self) -> GenomeState {
        GenomeState::from(self.genome_state.load(Ordering::Acquire))
    }

    /// Set genome state (atomic write)
    pub fn set_genome_state(&self, state: GenomeState) {
        self.genome_state.store(state as u8, Ordering::Release);
        self.increment_version();
    }

    /// Get connectome state (atomic read)
    pub fn get_connectome_state(&self) -> ConnectomeState {
        ConnectomeState::from(self.connectome_state.load(Ordering::Acquire))
    }

    /// Set connectome state (atomic write)
    pub fn set_connectome_state(&self, state: ConnectomeState) {
        self.connectome_state.store(state as u8, Ordering::Release);
        self.increment_version();
    }

    /// Get API state (atomic read)
    pub fn get_api_state(&self) -> ServiceState {
        ServiceState::from(self.api_state.load(Ordering::Acquire))
    }

    /// Set API state (atomic write)
    pub fn set_api_state(&self, state: ServiceState) {
        self.api_state.store(state as u8, Ordering::Release);
        self.increment_version();
    }

    /// Get ZMQ state (atomic read)
    pub fn get_zmq_state(&self) -> ServiceState {
        ServiceState::from(self.zmq_state.load(Ordering::Acquire))
    }

    /// Set ZMQ state (atomic write)
    pub fn set_zmq_state(&self, state: ServiceState) {
        self.zmq_state.store(state as u8, Ordering::Release);
        self.increment_version();
    }

    /// Check if brain is ready (atomic read)
    pub fn is_brain_ready(&self) -> bool {
        self.brain_readiness.load(Ordering::Acquire) != 0
    }

    /// Set brain readiness (atomic write)
    pub fn set_brain_ready(&self, ready: bool) {
        self.brain_readiness
            .store(if ready { 1 } else { 0 }, Ordering::Release);
        self.increment_version();
    }

    // ===== Statistics Accessors =====

    /// Get agent count (atomic read)
    pub fn get_agent_count(&self) -> u32 {
        self.agent_count.load(Ordering::Acquire)
    }

    /// Set agent count (atomic write)
    pub fn set_agent_count(&self, count: u32) {
        self.agent_count.store(count, Ordering::Release);
        self.increment_version();
    }

    /// Increment agent count (atomic)
    pub fn increment_agent_count(&self) -> u32 {
        let new_count = self.agent_count.fetch_add(1, Ordering::AcqRel) + 1;
        self.increment_version();
        new_count
    }

    /// Decrement agent count (atomic)
    pub fn decrement_agent_count(&self) -> u32 {
        let new_count = self
            .agent_count
            .fetch_sub(1, Ordering::AcqRel)
            .saturating_sub(1);
        self.increment_version();
        new_count
    }

    /// Get burst frequency (atomic read)
    pub fn get_burst_frequency(&self) -> f32 {
        f32::from_bits(self.burst_frequency.load(Ordering::Acquire))
    }

    /// Set burst frequency (atomic write)
    pub fn set_burst_frequency(&self, freq: f32) {
        self.burst_frequency
            .store(freq.to_bits(), Ordering::Release);
        self.increment_version();
    }

    /// Get neuron count (atomic read)
    pub fn get_neuron_count(&self) -> u32 {
        self.neuron_count.load(Ordering::Acquire)
    }

    /// Set neuron count (atomic write)
    pub fn set_neuron_count(&self, count: u32) {
        self.neuron_count.store(count, Ordering::Release);
        self.increment_version();
    }

    /// Add to neuron count (atomic increment)
    pub fn add_neuron_count(&self, delta: u32) -> u32 {
        let new_count = self.neuron_count.fetch_add(delta, Ordering::AcqRel) + delta;
        self.increment_version();
        new_count
    }

    /// Subtract from neuron count (atomic decrement)
    pub fn subtract_neuron_count(&self, delta: u32) -> u32 {
        let new_count = self
            .neuron_count
            .fetch_sub(delta, Ordering::AcqRel)
            .saturating_sub(delta);
        self.increment_version();
        new_count
    }

    /// Get synapse count (atomic read)
    pub fn get_synapse_count(&self) -> u32 {
        self.synapse_count.load(Ordering::Acquire)
    }

    /// Set synapse count (atomic write)
    pub fn set_synapse_count(&self, count: u32) {
        self.synapse_count.store(count, Ordering::Release);
        self.increment_version();
    }

    /// Add to synapse count (atomic increment)
    pub fn add_synapse_count(&self, delta: u32) -> u32 {
        let new_count = self.synapse_count.fetch_add(delta, Ordering::AcqRel) + delta;
        self.increment_version();
        new_count
    }

    /// Subtract from synapse count (atomic decrement)
    pub fn subtract_synapse_count(&self, delta: u32) -> u32 {
        let new_count = self
            .synapse_count
            .fetch_sub(delta, Ordering::AcqRel)
            .saturating_sub(delta);
        self.increment_version();
        new_count
    }

    /// Get cortical area count (atomic read)
    pub fn get_cortical_area_count(&self) -> u32 {
        self.cortical_area_count.load(Ordering::Acquire)
    }

    /// Set cortical area count (atomic write)
    pub fn set_cortical_area_count(&self, count: u32) {
        self.cortical_area_count.store(count, Ordering::Release);
        self.increment_version();
    }

    /// Get neuron capacity (atomic read)
    /// Capacity is set at NPU initialization and never changes
    pub fn get_neuron_capacity(&self) -> u32 {
        self.neuron_capacity.load(Ordering::Acquire)
    }

    /// Set neuron capacity (atomic write)
    /// Should be called once when NPU is initialized
    pub fn set_neuron_capacity(&self, capacity: u32) {
        self.neuron_capacity.store(capacity, Ordering::Release);
        self.increment_version();
    }

    /// Get synapse capacity (atomic read)
    /// Capacity is set at NPU initialization and never changes
    pub fn get_synapse_capacity(&self) -> u32 {
        self.synapse_capacity.load(Ordering::Acquire)
    }

    /// Set synapse capacity (atomic write)
    /// Should be called once when NPU is initialized
    pub fn set_synapse_capacity(&self, capacity: u32) {
        self.synapse_capacity.store(capacity, Ordering::Release);
        self.increment_version();
    }

    /// Get regular neuron count (atomic read)
    pub fn get_regular_neuron_count(&self) -> u32 {
        self.regular_neuron_count.load(Ordering::Acquire)
    }

    /// Set regular neuron count (atomic write)
    pub fn set_regular_neuron_count(&self, count: u32) {
        self.regular_neuron_count.store(count, Ordering::Release);
        self.increment_version();
    }

    /// Get memory neuron count (atomic read)
    pub fn get_memory_neuron_count(&self) -> u32 {
        self.memory_neuron_count.load(Ordering::Acquire)
    }

    /// Set memory neuron count (atomic write)
    pub fn set_memory_neuron_count(&self, count: u32) {
        self.memory_neuron_count.store(count, Ordering::Release);
        self.increment_version();
    }

    // ===== Fatigue State Accessors =====

    /// Get fatigue index (atomic read)
    /// Returns value 0-100 representing maximum utilization across all fatigue criteria
    pub fn get_fatigue_index(&self) -> u8 {
        self.fatigue_index.load(Ordering::Acquire)
    }

    /// Set fatigue index (atomic write)
    /// Value should be 0-100
    pub fn set_fatigue_index(&self, index: u8) {
        self.fatigue_index.store(index.min(100), Ordering::Release);
        self.increment_version();
    }

    /// Check if fatigue is active (atomic read)
    pub fn is_fatigue_active(&self) -> bool {
        self.fatigue_active.load(Ordering::Acquire) != 0
    }

    /// Set fatigue active state (atomic write)
    pub fn set_fatigue_active(&self, active: bool) {
        self.fatigue_active
            .store(if active { 1 } else { 0 }, Ordering::Release);
        self.increment_version();
    }

    /// Get regular neuron utilization percentage (atomic read)
    /// Returns value 0-100
    pub fn get_regular_neuron_util(&self) -> u8 {
        self.regular_neuron_util.load(Ordering::Acquire)
    }

    /// Set regular neuron utilization percentage (atomic write)
    /// Value should be 0-100
    pub fn set_regular_neuron_util(&self, util: u8) {
        self.regular_neuron_util.store(util.min(100), Ordering::Release);
        self.increment_version();
    }

    /// Get memory neuron utilization percentage (atomic read)
    /// Returns value 0-100
    pub fn get_memory_neuron_util(&self) -> u8 {
        self.memory_neuron_util.load(Ordering::Acquire)
    }

    /// Set memory neuron utilization percentage (atomic write)
    /// Value should be 0-100
    pub fn set_memory_neuron_util(&self, util: u8) {
        self.memory_neuron_util.store(util.min(100), Ordering::Release);
        self.increment_version();
    }

    /// Get synapse utilization percentage (atomic read)
    /// Returns value 0-100
    pub fn get_synapse_util(&self) -> u8 {
        self.synapse_util.load(Ordering::Acquire)
    }

    /// Set synapse utilization percentage (atomic write)
    /// Value should be 0-100
    pub fn set_synapse_util(&self, util: u8) {
        self.synapse_util.store(util.min(100), Ordering::Release);
        self.increment_version();
    }

    // ===== Versioning =====

    /// Get state version (atomic read)
    pub fn get_version(&self) -> u64 {
        self.version.load(Ordering::Acquire)
    }

    /// Increment version (internal)
    fn increment_version(&self) {
        self.version.fetch_add(1, Ordering::AcqRel);
        self.update_timestamp();
    }

    /// Update last modified timestamp (internal)
    fn update_timestamp(&self) {
        #[cfg(feature = "std")]
        {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            self.last_modified.store(now, Ordering::Release);
        }

        #[cfg(not(feature = "std"))]
        {
            // For no_std, just increment a counter
            self.last_modified.fetch_add(1, Ordering::Release);
        }
    }

    /// Get last modified timestamp
    pub fn get_last_modified(&self) -> u64 {
        self.last_modified.load(Ordering::Acquire)
    }
}

impl Default for MemoryMappedState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_size() {
        assert_eq!(std::mem::size_of::<MemoryMappedState>(), 128);
    }

    #[test]
    fn test_state_alignment() {
        assert_eq!(std::mem::align_of::<MemoryMappedState>(), 64);
    }

    #[test]
    fn test_burst_engine_state() {
        let state = MemoryMappedState::new();
        assert_eq!(
            state.get_burst_engine_state(),
            BurstEngineState::Unavailable
        );

        state.set_burst_engine_state(BurstEngineState::Running);
        assert_eq!(state.get_burst_engine_state(), BurstEngineState::Running);
    }

    #[test]
    fn test_genome_state() {
        let state = MemoryMappedState::new();
        assert_eq!(state.get_genome_state(), GenomeState::Missing);

        state.set_genome_state(GenomeState::Loaded);
        assert_eq!(state.get_genome_state(), GenomeState::Loaded);
    }

    #[test]
    fn test_agent_count() {
        let state = MemoryMappedState::new();
        assert_eq!(state.get_agent_count(), 0);

        state.increment_agent_count();
        assert_eq!(state.get_agent_count(), 1);

        state.increment_agent_count();
        assert_eq!(state.get_agent_count(), 2);

        state.decrement_agent_count();
        assert_eq!(state.get_agent_count(), 1);
    }

    #[test]
    fn test_burst_frequency() {
        let state = MemoryMappedState::new();
        assert_eq!(state.get_burst_frequency(), 0.0);

        state.set_burst_frequency(30.5);
        assert_eq!(state.get_burst_frequency(), 30.5);
    }

    #[test]
    fn test_brain_readiness() {
        let state = MemoryMappedState::new();
        assert!(!state.is_brain_ready());

        state.set_brain_ready(true);
        assert!(state.is_brain_ready());

        state.set_brain_ready(false);
        assert!(!state.is_brain_ready());
    }

    #[test]
    fn test_version_increment() {
        let state = MemoryMappedState::new();
        let v1 = state.get_version();

        state.set_burst_engine_state(BurstEngineState::Running);
        let v2 = state.get_version();
        assert!(v2 > v1);

        state.set_neuron_count(1000);
        let v3 = state.get_version();
        assert!(v3 > v2);
    }

    #[test]
    fn test_statistics() {
        let state = MemoryMappedState::new();

        state.set_neuron_count(1_000_000);
        state.set_synapse_count(50_000_000);
        state.set_cortical_area_count(100);

        assert_eq!(state.get_neuron_count(), 1_000_000);
        assert_eq!(state.get_synapse_count(), 50_000_000);
        assert_eq!(state.get_cortical_area_count(), 100);
    }

    #[test]
    fn test_fatigue_state() {
        let state = MemoryMappedState::new();
        assert_eq!(state.get_fatigue_index(), 0);
        assert!(!state.is_fatigue_active());

        state.set_fatigue_index(85);
        assert_eq!(state.get_fatigue_index(), 85);

        state.set_fatigue_active(true);
        assert!(state.is_fatigue_active());

        state.set_fatigue_active(false);
        assert!(!state.is_fatigue_active());
    }

    #[test]
    fn test_fatigue_utilization() {
        let state = MemoryMappedState::new();

        state.set_regular_neuron_util(75);
        state.set_memory_neuron_util(80);
        state.set_synapse_util(90);

        assert_eq!(state.get_regular_neuron_util(), 75);
        assert_eq!(state.get_memory_neuron_util(), 80);
        assert_eq!(state.get_synapse_util(), 90);
    }

    #[test]
    fn test_fatigue_index_clamping() {
        let state = MemoryMappedState::new();

        // Test clamping to 100
        state.set_fatigue_index(150);
        assert_eq!(state.get_fatigue_index(), 100);

        state.set_regular_neuron_util(200);
        assert_eq!(state.get_regular_neuron_util(), 100);
    }
}
