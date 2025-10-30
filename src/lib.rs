//! # FEAGI - Framework for Evolutionary Artificial General Intelligence
//!
//! FEAGI is a pure neural computation framework for building artificial general intelligence
//! through evolutionary principles. This crate provides the core algorithms without any I/O
//! dependencies.
//!
//! ## Quick Start
//!
//! ```toml
//! [dependencies]
//! feagi = "2.0"  # Default: std + full features
//! ```
//!
//! ## Feature Flags
//!
//! ### Platform Targets
//! - **`std`** (default): Standard Rust (Linux, macOS, Windows, Docker)
//! - **`no_std`**: RTOS/embedded targets (FreeRTOS, Zephyr, bare-metal)
//! - **`wasm`**: WebAssembly support
//!
//! ### Component Selection
//! - **`full`** (default): All components
//! - **`compute`**: Just NPU + state (no I/O)
//! - **`io`**: PNS + agent SDK (requires compute)
//!
//! ### Individual Components
//! - **`burst-engine`**: NPU execution
//! - **`bdu`**: Neurogenesis
//! - **`plasticity`**: Synaptic learning
//! - **`state-manager`**: Runtime state
//! - **`serialization`**: Connectome I/O
//! - **`pns`**: ZMQ/UDP transport
//! - **`agent-sdk`**: Rust agent library
//!
//! ## Usage Examples
//!
//! ### Full FEAGI (all features)
//!
//! ```toml
//! [dependencies]
//! feagi = "2.0"
//! ```
//!
//! ```rust,no_run
//! use feagi::prelude::*;
//!
//! // Create NPU
//! let mut npu = RustNPU::new(100_000, 1_000_000, 20);
//!
//! // Add neurons
//! let neuron_id = npu.add_neuron(
//!     1.0,    // threshold
//!     0.1,    // leak
//!     0.0,    // resting
//!     0,      // type
//!     3,      // refractory
//!     1.0,    // excitability
//!     10,     // consecutive limit
//!     5,      // snooze
//!     true,   // mp accumulation
//!     0,      // cortical area
//!     0, 0, 0 // x, y, z
//! )?;
//!
//! // Run burst
//! let result = npu.process_burst()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Inference Only (no neurogenesis)
//!
//! ```toml
//! [dependencies]
//! feagi = { version = "2.0", features = ["burst-engine", "serialization"] }
//! ```
//!
//! ```rust,no_run
//! use feagi::burst_engine::RustNPU;
//! use feagi::serialization::load_connectome;
//!
//! // Load pre-trained brain
//! let snapshot = load_connectome("brain.connectome")?;
//! let mut npu = RustNPU::import_connectome(snapshot);
//!
//! // Run inference
//! loop {
//!     let result = npu.process_burst()?;
//!     // ... process results
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### WASM Deployment
//!
//! ```toml
//! [dependencies]
//! feagi = { version = "2.0", features = ["wasm", "compute"], default-features = false }
//! ```
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │  Foundation: feagi-types                                │
//! │  (Neuron, Synapse, CorticalArea)                        │
//! └─────────────────────────────────────────────────────────┘
//!                         ↓
//! ┌─────────────────────────────────────────────────────────┐
//! │  Infrastructure: feagi-state-manager                    │
//! │  (Runtime state, lock-free operations)                  │
//! └─────────────────────────────────────────────────────────┘
//!                         ↓
//! ┌─────────────────────────────────────────────────────────┐
//! │  Algorithms: burst-engine, bdu, plasticity              │
//! │  (Pure neural computation, no I/O)                      │
//! └─────────────────────────────────────────────────────────┘
//!                         ↓
//! ┌─────────────────────────────────────────────────────────┐
//! │  I/O: feagi-pns, feagi-agent-sdk                        │
//! │  (ZMQ/UDP transport, agent communication)               │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Platform Support
//!
//! - ✅ Linux (x86_64, ARM64)
//! - ✅ macOS (Intel, Apple Silicon)
//! - ✅ Windows (x86_64)
//! - ✅ Docker / Kubernetes
//! - ✅ RTOS (FreeRTOS, Zephyr) via `no_std`
//! - ✅ WebAssembly via `wasm`
//!
//! ## Performance
//!
//! - **State reads**: 5-20 nanoseconds (lock-free atomic)
//! - **Burst cycle**: 100-1000 Hz (depends on genome size)
//! - **Neurons**: Tested up to 10M neurons
//! - **Synapses**: Tested up to 100M synapses
//!
//! ## Related Crates
//!
//! - **feagi-data-processing**: Foundation data structures
//! - **feagi-io**: I/O layer (PNS, agent SDK) - separate repo
//! - **feagi-py**: Python bindings - separate repo
//! - **feagi-connector**: Python agent SDK - separate repo
//!
//! ## License
//!
//! Apache-2.0

#![cfg_attr(not(feature = "std"), no_std)]

// Re-export foundation
pub use feagi_types as types;

// Re-export infrastructure
#[cfg(feature = "state-manager")]
pub use feagi_state_manager as state_manager;

// Re-export algorithms
#[cfg(feature = "burst-engine")]
pub use feagi_burst_engine as burst_engine;

#[cfg(feature = "bdu")]
pub use feagi_bdu as bdu;

#[cfg(feature = "plasticity")]
pub use feagi_plasticity as plasticity;

#[cfg(feature = "serialization")]
pub use feagi_connectome_serialization as serialization;

// Re-export I/O layer
#[cfg(feature = "pns")]
pub use feagi_pns as pns;

#[cfg(feature = "agent-sdk")]
pub use feagi_agent_sdk as agent_sdk;

/// Prelude - commonly used types and traits
pub mod prelude {
    pub use crate::types::*;
    
    #[cfg(feature = "burst-engine")]
    pub use crate::burst_engine::{RustNPU, BurstResult};
    
    #[cfg(feature = "state-manager")]
    pub use crate::state_manager::{StateManager, BurstEngineState, GenomeState};
    
    #[cfg(feature = "bdu")]
    pub use crate::bdu::connectivity::synaptogenesis::*;
    
    #[cfg(feature = "plasticity")]
    pub use crate::plasticity::service::{PlasticityService, PlasticityConfig};
    
    #[cfg(feature = "serialization")]
    pub use crate::serialization::{save_connectome, load_connectome, ConnectomeSnapshot};
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_facade_imports() {
        // Just test that re-exports work
        use crate::types::*;
        let _neuron_id = NeuronId(0);
    }
}


