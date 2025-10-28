//! Memory-mapped core state with lock-free atomic operations
//!
//! This module provides a 64-byte cache-line aligned atomic state structure
//! for ultra-fast cross-thread/cross-process state access (5-20ns reads).

use atomic_polyfill::{AtomicU8, AtomicU32, AtomicU64, Ordering};

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
}

impl From<u8> for BurstEngineState {
    fn from(value: u8) -> Self {
        match value {
            1 => BurstEngineState::Initializing,
            2 => BurstEngineState::Ready,
            3 => BurstEngineState::Running,
            4 => BurstEngineState::Paused,
            5 => BurstEngineState::Error,
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

/// Memory-mapped core state (64 bytes, cache-line aligned)
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
    pub burst_frequency: AtomicU32,  // f32 as u32 bits
    
    // Statistics (16 bytes)
    pub neuron_count: AtomicU32,
    pub synapse_count: AtomicU32,
    pub cortical_area_count: AtomicU32,
    pub memory_usage: AtomicU32,
    
    // Versioning & timestamps (16 bytes)
    pub version: AtomicU64,
    pub last_modified: AtomicU64,
    
    // Genome tracking (8 bytes)
    pub genome_timestamp: AtomicU64,
    
    // Padding to 64 bytes (8 bytes)
    pub _padding: [u8; 8],
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
            
            version: AtomicU64::new(0),
            last_modified: AtomicU64::new(0),
            genome_timestamp: AtomicU64::new(0),
            
            _padding: [0; 8],
        }
    }
    
    // ===== Lock-Free State Accessors (5-20ns reads) =====
    
    /// Get burst engine state (atomic read)
    pub fn get_burst_engine_state(&self) -> BurstEngineState {
        BurstEngineState::from(self.burst_engine_state.load(Ordering::Acquire))
    }
    
    /// Set burst engine state (atomic write)
    pub fn set_burst_engine_state(&self, state: BurstEngineState) {
        self.burst_engine_state.store(state as u8, Ordering::Release);
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
        self.brain_readiness.store(if ready { 1 } else { 0 }, Ordering::Release);
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
        let new_count = self.agent_count.fetch_sub(1, Ordering::AcqRel).saturating_sub(1);
        self.increment_version();
        new_count
    }
    
    /// Get burst frequency (atomic read)
    pub fn get_burst_frequency(&self) -> f32 {
        f32::from_bits(self.burst_frequency.load(Ordering::Acquire))
    }
    
    /// Set burst frequency (atomic write)
    pub fn set_burst_frequency(&self, freq: f32) {
        self.burst_frequency.store(freq.to_bits(), Ordering::Release);
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
    
    /// Get synapse count (atomic read)
    pub fn get_synapse_count(&self) -> u32 {
        self.synapse_count.load(Ordering::Acquire)
    }
    
    /// Set synapse count (atomic write)
    pub fn set_synapse_count(&self, count: u32) {
        self.synapse_count.store(count, Ordering::Release);
        self.increment_version();
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
        assert_eq!(std::mem::size_of::<MemoryMappedState>(), 64);
    }
    
    #[test]
    fn test_state_alignment() {
        assert_eq!(std::mem::align_of::<MemoryMappedState>(), 64);
    }
    
    #[test]
    fn test_burst_engine_state() {
        let state = MemoryMappedState::new();
        assert_eq!(state.get_burst_engine_state(), BurstEngineState::Unavailable);
        
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
}

