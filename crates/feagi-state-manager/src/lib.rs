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
//! ```rust,no_run
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

// Platform-specific imports
#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "no_std")]
extern crate alloc;

// Module structure
pub mod core_state;        // Memory-mapped atomic state
pub mod agent_registry;    // Agent management
pub mod cortical_locks;    // Cortical locking
pub mod fcl_cache;         // FCL window size cache
pub mod events;            // Event streaming
pub mod persistence;       // State save/load

// Re-exports
pub use core_state::{
    MemoryMappedState,
    BurstEngineState,
    GenomeState,
    ConnectomeState,
    ServiceState,
};

#[cfg(feature = "std")]
pub use agent_registry::{AgentRegistry, AgentInfo};

#[cfg(feature = "std")]
pub use persistence::{save_state, load_state};

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

/// Main state manager
pub struct StateManager {
    // TODO: Implement full state manager
    _phantom: std::marker::PhantomData<()>,
}

impl StateManager {
    /// Create a new state manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            _phantom: std::marker::PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_manager_creation() {
        let _state = StateManager::new().unwrap();
    }
}
