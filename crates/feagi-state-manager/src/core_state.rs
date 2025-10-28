//! Memory-mapped core state with lock-free atomic operations

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
}

