// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! # FEAGI State Manager
//!
//! Runtime state management for FEAGI - cross-platform, RTOS-compatible, and WASM-ready.
//!
//! ## Platform Support
//!
//! - **std** (default): Linux, macOS, Windows, Docker
//! - **no_std**: RTOS, embedded systems (FreeRTOS, Zephyr, bare-metal)
//! - **wasm**: WebAssembly (single-threaded)
//! - **wasm-threaded**: WebAssembly with Web Workers
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │   Memory-Mapped Core State          │  ← Lock-free atomic operations (5-20ns reads)
//! │   (64-byte cache-line aligned)      │
//! └─────────────────────────────────────┘
//!           ↓
//! ┌─────────────────────────────────────┐
//! │   Agent Registry                    │  ← Arc<RwLock> (read-optimized)
//! │   Cortical Lock Manager             │  ← Wait-free algorithm
//! │   FCL Window Size Cache             │  ← Rarely accessed
//! └─────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! // TODO: Update once full API is implemented
//! use feagi_state_manager::StateManager;
//!
//! // Create or attach to shared state
//! let state = StateManager::new()?;
//!
//! // Lock-free read (<20ns)
//! let burst_state = state.get_burst_engine_state();
//!
//! // Lock-free write (<30ns)
//! state.set_burst_engine_state(BurstEngineState::Running);
//!
//! // Agent operations (read-lock, rare writes)
//! state.register_agent(agent_info)?;
//! let agents = state.get_agents();
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use once_cell::sync::Lazy;
#[cfg(feature = "std")]
use parking_lot::RwLock;
#[cfg(feature = "std")]
use std::sync::Arc;

/// Crate version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Platform-specific imports
#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "no_std")]
extern crate alloc;

// Module structure
pub mod agent_registry; // Agent management
pub mod core_state; // Memory-mapped atomic state
pub mod cortical_locks; // Cortical locking
pub mod events; // Event streaming
pub mod fcl_cache; // FCL window size cache
pub mod persistence; // State save/load

// Re-exports
pub use core_state::{
    BurstEngineState, ConnectomeState, GenomeState, MemoryMappedState, ServiceState,
};

#[cfg(feature = "std")]
pub use agent_registry::{AgentInfo, AgentRegistry, AgentType};

#[cfg(feature = "std")]
pub use cortical_locks::CorticalLockManager;

#[cfg(feature = "std")]
pub use fcl_cache::FCLWindowCache;

#[cfg(feature = "std")]
pub use persistence::StateSnapshot;

/// State manager error types
#[derive(Debug)]
pub enum StateError {
    /// I/O error (file operations)
    Io(std::io::Error),

    /// Invalid state transition
    InvalidTransition(String),

    /// Agent not found
    AgentNotFound(String),

    /// Agent already registered
    AgentAlreadyRegistered(String),

    /// Memory mapping failed
    MemoryMapError(String),

    /// Serialization error
    SerializationError(String),

    /// Persistence error (save/load)
    PersistenceError(String),
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateError::Io(e) => write!(f, "I/O error: {}", e),
            StateError::InvalidTransition(msg) => write!(f, "Invalid transition: {}", msg),
            StateError::AgentNotFound(id) => write!(f, "Agent not found: {}", id),
            StateError::AgentAlreadyRegistered(id) => write!(f, "Agent already registered: {}", id),
            StateError::MemoryMapError(msg) => write!(f, "Memory map error: {}", msg),
            StateError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            StateError::PersistenceError(msg) => write!(f, "Persistence error: {}", msg),
        }
    }
}

impl std::error::Error for StateError {}

impl From<std::io::Error> for StateError {
    fn from(e: std::io::Error) -> Self {
        StateError::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, StateError>;

/// Main state manager - coordinates all state subsystems
#[cfg(feature = "std")]
pub struct StateManager {
    /// Core atomic state (lock-free, 64-byte cache-aligned)
    core_state: std::sync::Arc<MemoryMappedState>,

    /// Agent registry (read-optimized locking)
    agent_registry: std::sync::Arc<AgentRegistry>,

    /// Cortical area locks (for neurogenesis/plasticity)
    cortical_locks: std::sync::Arc<CorticalLockManager>,

    /// FCL window size cache
    fcl_cache: std::sync::Arc<FCLWindowCache>,
}

#[cfg(feature = "std")]
impl StateManager {
    /// Create a new state manager with default configuration
    pub fn new() -> Result<Self> {
        Self::with_default_fcl_window(20)
    }

    /// Create a new state manager with custom FCL window size
    pub fn with_default_fcl_window(fcl_window: usize) -> Result<Self> {
        Ok(Self {
            core_state: std::sync::Arc::new(MemoryMappedState::new()),
            agent_registry: std::sync::Arc::new(AgentRegistry::new()),
            cortical_locks: std::sync::Arc::new(CorticalLockManager::new()),
            fcl_cache: std::sync::Arc::new(FCLWindowCache::new(fcl_window)),
        })
    }

    // ===== Core State Access =====

    /// Get burst engine state
    pub fn get_burst_engine_state(&self) -> BurstEngineState {
        self.core_state.get_burst_engine_state()
    }

    /// Set burst engine state
    pub fn set_burst_engine_state(&self, state: BurstEngineState) {
        self.core_state.set_burst_engine_state(state)
    }

    /// Get genome state
    pub fn get_genome_state(&self) -> GenomeState {
        self.core_state.get_genome_state()
    }

    /// Set genome state
    pub fn set_genome_state(&self, state: GenomeState) {
        self.core_state.set_genome_state(state)
    }

    /// Check if brain is ready
    pub fn is_brain_ready(&self) -> bool {
        self.core_state.is_brain_ready()
    }

    /// Set brain readiness
    pub fn set_brain_ready(&self, ready: bool) {
        self.core_state.set_brain_ready(ready)
    }

    // ===== Agent Management =====

    /// Register a new agent
    pub fn register_agent(&self, info: AgentInfo) -> Result<()> {
        self.agent_registry.register(info)?;
        self.core_state.increment_agent_count();
        Ok(())
    }

    /// Deregister an agent
    pub fn deregister_agent(&self, agent_id: &str) -> Result<()> {
        self.agent_registry.deregister(agent_id)?;
        self.core_state.decrement_agent_count();
        Ok(())
    }

    /// Get all agents
    pub fn get_all_agents(&self) -> Vec<AgentInfo> {
        self.agent_registry.get_all()
    }

    /// Get agent count
    pub fn get_agent_count(&self) -> usize {
        self.agent_registry.count()
    }

    // ===== Cortical Area Locking =====

    /// Try to lock a cortical area
    pub fn try_lock_cortical_area(&self, cortical_area: u32) -> bool {
        self.cortical_locks.try_lock(cortical_area)
    }

    /// Unlock a cortical area
    pub fn unlock_cortical_area(&self, cortical_area: u32) {
        self.cortical_locks.unlock(cortical_area)
    }

    // ===== FCL Cache =====

    /// Get FCL window size for cortical area
    pub fn get_fcl_window(&self, cortical_area: u32) -> usize {
        self.fcl_cache.get(cortical_area)
    }

    /// Set FCL window size for cortical area
    pub fn set_fcl_window(&self, cortical_area: u32, window_size: usize) {
        self.fcl_cache.set(cortical_area, window_size)
    }

    /// Get reference to core state (for direct access to atomic operations)
    pub fn get_core_state(&self) -> &MemoryMappedState {
        &self.core_state
    }

    // ===== Persistence =====

    /// Create a snapshot of current state
    pub fn create_snapshot(&self) -> StateSnapshot {
        StateSnapshot {
            genome_state: self.core_state.get_genome_state() as u8,
            connectome_state: self.core_state.get_connectome_state() as u8,
            burst_engine_state: self.core_state.get_burst_engine_state() as u8,
            agent_count: self.core_state.get_agent_count(),
            burst_frequency: self.core_state.get_burst_frequency(),
            neuron_count: self.core_state.get_neuron_count(),
            synapse_count: self.core_state.get_synapse_count(),
            cortical_area_count: self.core_state.get_cortical_area_count(),
            version: self.core_state.get_version(),
            timestamp: self.core_state.get_last_modified(),
        }
    }

    /// Save state to file
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<()> {
        let snapshot = self.create_snapshot();
        snapshot.save_to_file(path)
    }

    /// Load state from file (returns snapshot)
    pub fn load_from_file(path: &std::path::Path) -> Result<StateSnapshot> {
        StateSnapshot::load_from_file(path)
    }
}

// ===== Singleton Pattern =====

/// Global singleton instance of StateManager
///
/// This is initialized lazily on first access. The initialization is thread-safe
/// and non-blocking. If initialization fails, it will panic (which should never happen).
#[cfg(feature = "std")]
static INSTANCE: Lazy<Arc<RwLock<StateManager>>> = Lazy::new(|| {
    // Initialize StateManager - this should never fail in normal operation
    // StateManager::new() just creates structs, so it's fast and non-blocking
    let state_manager = StateManager::new()
        .or_else(|_| StateManager::with_default_fcl_window(20))
        .expect("Failed to initialize StateManager - this should never happen");
    Arc::new(RwLock::new(state_manager))
});

#[cfg(feature = "std")]
impl StateManager {
    /// Get the global singleton instance of StateManager
    ///
    /// This provides thread-safe access to the shared state manager.
    /// The instance is lazily initialized on first access.
    ///
    /// # Safety
    ///
    /// This method is safe to call from any thread. The singleton is initialized
    /// on first access using `once_cell::sync::Lazy`, which is thread-safe.
    ///
    /// # Example
    ///
    /// ```rust
    /// use feagi_state_manager::StateManager;
    ///
    /// let state = StateManager::instance();
    /// let manager = state.read();
    /// manager.set_fatigue_index(85);
    /// ```
    pub fn instance() -> Arc<RwLock<StateManager>> {
        // Force initialization by accessing the Lazy value
        // This is safe because Lazy::new() is thread-safe and only executes once
        let _ = &*INSTANCE;
        Arc::clone(&INSTANCE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_manager_creation() {
        let state = StateManager::new().unwrap();
        assert_eq!(state.get_agent_count(), 0);
        assert!(!state.is_brain_ready());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_state_manager_agent_registration() {
        let state = StateManager::new().unwrap();

        let agent = AgentInfo::new("agent1".to_string(), AgentType::Sensory);
        state.register_agent(agent).unwrap();

        assert_eq!(state.get_agent_count(), 1);
        assert_eq!(state.get_all_agents().len(), 1);

        state.deregister_agent("agent1").unwrap();
        assert_eq!(state.get_agent_count(), 0);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_state_manager_state_transitions() {
        let state = StateManager::new().unwrap();

        state.set_genome_state(GenomeState::Loaded);
        assert_eq!(state.get_genome_state(), GenomeState::Loaded);

        state.set_burst_engine_state(BurstEngineState::Running);
        assert_eq!(state.get_burst_engine_state(), BurstEngineState::Running);

        state.set_brain_ready(true);
        assert!(state.is_brain_ready());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_state_manager_cortical_locks() {
        let state = StateManager::new().unwrap();

        assert!(state.try_lock_cortical_area(0));
        assert!(!state.try_lock_cortical_area(0)); // Already locked

        state.unlock_cortical_area(0);
        assert!(state.try_lock_cortical_area(0)); // Can lock again
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_state_manager_fcl_cache() {
        let state = StateManager::new().unwrap();

        assert_eq!(state.get_fcl_window(0), 20); // Default

        state.set_fcl_window(0, 30);
        assert_eq!(state.get_fcl_window(0), 30);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_state_manager_persistence() {
        let state = StateManager::new().unwrap();

        state.set_genome_state(GenomeState::Loaded);
        state.set_burst_engine_state(BurstEngineState::Running);

        let agent = AgentInfo::new("agent1".to_string(), AgentType::Sensory);
        state.register_agent(agent).unwrap();

        let snapshot = state.create_snapshot();
        assert_eq!(snapshot.genome_state, GenomeState::Loaded as u8);
        assert_eq!(snapshot.burst_engine_state, BurstEngineState::Running as u8);
        assert_eq!(snapshot.agent_count, 1);

        let temp_path = std::path::Path::new("/tmp/feagi_state_manager_test.bin");
        state.save_to_file(temp_path).unwrap();

        let loaded = StateManager::load_from_file(temp_path).unwrap();
        assert_eq!(loaded.genome_state, GenomeState::Loaded as u8);

        std::fs::remove_file(temp_path).ok();
    }
}
