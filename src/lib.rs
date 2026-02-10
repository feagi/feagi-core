// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

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
//! feagi = "0.0.1"  # Umbrella crate (default: std + full features)
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
//! - **`brain-development`**: Neurogenesis
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
//! feagi = "0.0.1"
//! ```
//!
//! ```rust,no_run
//! use feagi::burst_engine::{backend::CPUBackend, RustNPU};
//! use feagi_npu_runtime::StdRuntime;
//!
//! // Create NPU
//! let mut npu =
//!     RustNPU::<StdRuntime, f32, CPUBackend>::new(StdRuntime, CPUBackend::new(), 100_000, 1_000_000, 20)?;
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
//! feagi = { version = "0.0.1", features = ["burst-engine", "serialization"] }
//! ```
//!
//! ```rust,no_run
//! use feagi::burst_engine::{backend::CPUBackend, DynamicNPU};
//! use feagi::serialization::load_connectome;
//! use feagi_npu_runtime::StdRuntime;
//!
//! // Load pre-trained brain (snapshot usage pending import API refactor)
//! let _snapshot = load_connectome("brain.connectome")?;
//!
//! // Create NPU for inference
//! let mut npu = DynamicNPU::new_f32(StdRuntime, CPUBackend::new(), 100_000, 1_000_000, 20)?;
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
//! feagi = { version = "0.0.1", features = ["wasm", "compute"], default-features = false }
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
//! │  Algorithms: burst-engine, brain-development, plasticity │
//! │  (Pure neural computation, no I/O)                      │
//! └─────────────────────────────────────────────────────────┘
//!                         ↓
//! ┌─────────────────────────────────────────────────────────┐
//! │  I/O: feagi-io, feagi-agent-sdk                         │
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
// Note: feagi_types removed - use feagi_npu_neural::types or feagi_data_structures instead
pub use feagi_npu_neural::types;

// Re-export infrastructure
#[cfg(feature = "state-manager")]
pub use feagi_state_manager as state_manager;

// Re-export algorithms
#[cfg(feature = "burst-engine")]
pub use feagi_npu_burst_engine as burst_engine;

#[cfg(feature = "brain-development")]
pub use feagi_brain_development as bdu;

#[cfg(feature = "plasticity")]
pub use feagi_npu_plasticity as plasticity;

#[cfg(feature = "serialization")]
pub use feagi_io::connectome as serialization;

// Re-export I/O layer
#[cfg(feature = "sensorimotor")]
pub use feagi_io as io;

#[cfg(feature = "agent-sdk")]
pub use feagi_agent as agent;

/// Prelude - commonly used types and traits
pub mod prelude {
    pub use crate::types::*;

    #[cfg(feature = "burst-engine")]
    pub use crate::burst_engine::{BurstResult, RustNPU};

    #[cfg(feature = "state-manager")]
    pub use crate::state_manager::{BurstEngineState, GenomeState, StateManager};

    #[cfg(feature = "brain-development")]
    pub use crate::bdu::connectivity::synaptogenesis::*;

    #[cfg(feature = "plasticity")]
    pub use crate::plasticity::service::{PlasticityConfig, PlasticityService};

    #[cfg(feature = "serialization")]
    pub use crate::serialization::{load_connectome, save_connectome};
    #[cfg(feature = "serialization")]
    pub use feagi_npu_neural::types::connectome::ConnectomeSnapshot;
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
