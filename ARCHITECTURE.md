# FEAGI Core Architecture

## Overview

`feagi-core` contains the pure neural computation components of FEAGI 2.0 - all algorithms that process neural activity without any I/O dependencies.

## Repository Structure

This repository is a dedicated workspace for core FEAGI components, maintained separately from:
- **feagi-data-processing** (foundation library, separate repo)
- **feagi-io** (I/O layer with feagi-pns, separate repo)
- **feagi-py** (Python bindings and orchestration, separate repo)
- **feagi-connector** (agent SDK, separate repo)
- **brain-visualizer** (3D visualization, separate repo)

## Crate Hierarchy

### Foundation Layer
- **`feagi-types`**: Core data structures (Neuron, Synapse, CorticalArea, etc.)
  - No dependencies except std/no_std primitives
  - Used by all other crates

### Infrastructure Layer
- **`feagi-state-manager`**: Runtime state management
  - **NEW**: Multi-platform state management (std, no_std, wasm)
  - Lock-free atomic operations for high-frequency access
  - Agent registry, cortical locking, FCL window cache
  - Depends on: `feagi-types`, `feagi-data-processing`

### Algorithm Layer (Core Neural Computation)
- **`feagi-burst-engine`**: NPU execution & inference
  - Burst loop runner
  - Synaptic propagation
  - Neural dynamics
  - Fire structures (FCL, Fire Queue, Fire Ledger)
  - Depends on: `feagi-types`, `feagi-state-manager`

- **`feagi-bdu`**: Neurogenesis (Brain Development Unit)
  - Cortical area creation
  - Synaptogenesis (connectivity rules)
  - Morphology patterns (projector, expander, etc.)
  - Spatial hashing (Morton encoding)
  - Depends on: `feagi-types`, `feagi-state-manager`

- **`feagi-plasticity`**: Synaptic learning
  - STDP (Spike-Timing-Dependent Plasticity)
  - Pattern detection (temporal patterns)
  - Memory neuron management
  - Depends on: `feagi-types`, `feagi-state-manager`

- **`feagi-connectome-serialization`**: Connectome persistence
  - Binary serialization format
  - LZ4 compression
  - Checksum validation
  - Depends on: `feagi-types`

### I/O Layer (TODO: Move to `feagi-io` repo)
- **`feagi-pns`**: Peripheral Nervous System
  - ZMQ/UDP transport for agent communication
  - Agent registry & heartbeat
  - Sensory injection, motor output, visualization streaming
  - **Status**: Should be moved to separate `feagi-io` repository

- **`feagi-agent-sdk`**: Rust agent SDK
  - Client library for building agents in Rust
  - ZMQ connection management
  - Sensory/motor data structures
  - **Status**: Should be moved to `feagi-io` or `feagi-connector` repository

### Applications
- **`feagi-inference-engine`**: Standalone inference binary
  - Command-line tool for running inference without Python
  - Loads pre-trained connectomes
  - Communicates with agents via ZMQ
  - Depends on: All core crates

## Dependency Graph

```
feagi-data-processing (external, foundation)
        ↓
    feagi-types
        ↓
    feagi-state-manager
        ↓
    ┌───────────────────┬────────────────────┬──────────────────┐
    ↓                   ↓                    ↓                  ↓
feagi-burst-engine  feagi-bdu        feagi-plasticity  feagi-connectome-serialization
    ↓                   ↓                    ↓                  ↓
    └───────────────────┴────────────────────┴──────────────────┘
                            ↓
                    feagi-pns (I/O layer)
                    feagi-agent-sdk (I/O layer)
                            ↓
                    feagi-inference-engine (application)
```

## Platform Support

### Targets
1. **std** (default): Linux, macOS, Windows, Docker
2. **no_std**: RTOS, embedded (FreeRTOS, Zephyr, bare-metal)
3. **wasm**: WebAssembly (single-threaded)
4. **wasm-threaded**: WebAssembly with Web Workers

### Conditional Compilation Strategy

```rust
// Core atomics work everywhere
#[cfg(feature = "std")]
use parking_lot::RwLock;

#[cfg(feature = "no_std")]
use spin::RwLock;

#[cfg(all(target_family = "wasm", not(feature = "wasm-threaded")))]
use std::cell::RefCell;  // Single-threaded WASM

#[cfg(all(target_family = "wasm", feature = "wasm-threaded"))]
use wasm_sync::Mutex;    // Multi-threaded WASM
```

## Migration Roadmap

### Completed
- ✅ feagi-types (foundation)
- ✅ feagi-burst-engine (algorithm)
- ✅ feagi-bdu (algorithm)
- ✅ feagi-plasticity (algorithm)
- ✅ feagi-connectome-serialization (persistence)
- ✅ feagi-pns (I/O, needs migration)
- ✅ feagi-inference-engine (application)

### In Progress
- 🚧 feagi-state-manager (NEW, infrastructure layer)
  - Skeleton created
  - Modules stubbed out
  - Implementation in progress

### Planned
- 📋 Move `feagi-pns` to `feagi-io` repository
- 📋 Move `feagi-agent-sdk` to `feagi-io` or `feagi-connector` repository
- 📋 Publish all crates to crates.io with synchronized versions

## Design Principles

### 1. Pure Computation
All core crates contain ONLY neural computation algorithms:
- ✅ Neuron dynamics
- ✅ Synaptic propagation
- ✅ Pattern detection
- ✅ Neurogenesis rules
- ❌ No network I/O
- ❌ No agent management (except state tracking)
- ❌ No Python dependencies

### 2. Platform Agnostic
Code must work on:
- Desktop (Linux, macOS, Windows)
- Cloud (Docker, Kubernetes)
- Embedded (Raspberry Pi, NVIDIA Jetson)
- RTOS (FreeRTOS, Zephyr)
- WASM (browser, Node.js)

### 3. Deterministic
- Fixed-size data structures where possible
- No heap allocation in hot paths (RTOS compatibility)
- Atomic operations for lock-free synchronization
- Predictable performance characteristics

### 4. Zero-Copy Where Possible
- Use `Arc<T>` for shared ownership
- Memory-mapped state for cross-process access
- Direct buffer references (no unnecessary cloning)

### 5. Rust/RTOS Compatible
- No Python in critical paths
- No dynamic dispatch in hot loops
- Static typing throughout
- Minimal dependencies

## Publishing Strategy

### Version Synchronization
All crates will be published with synchronized versions:
- `feagi-types = "2.0.0"`
- `feagi-state-manager = "2.0.0"`
- `feagi-burst-engine = "2.0.0"`
- etc.

### Crates.io Namespace
```
feagi-types
feagi-state-manager
feagi-burst-engine
feagi-bdu
feagi-plasticity
feagi-connectome-serialization
feagi-pns (after migration to feagi-io)
```

### Publishing Order
1. `feagi-types` (no dependencies)
2. `feagi-state-manager` (depends on types)
3. Core algorithms (depend on state-manager)
4. Applications (depend on core)

## Testing Strategy

### Unit Tests
Each crate has comprehensive unit tests:
```bash
cd crates/feagi-burst-engine
cargo test
```

### Integration Tests
Cross-crate integration tests in `tests/`:
```bash
cargo test --workspace
```

### Benchmarks
Performance benchmarks for critical paths:
```bash
cargo bench --workspace
```

### Architecture Compliance
Automated checks for:
- No Python dependencies in core
- Platform-agnostic code
- Proper feature gating

## Contributing

### Adding a New Crate
1. Place in appropriate layer (foundation/infrastructure/algorithm/I/O/application)
2. Update this ARCHITECTURE.md
3. Update root Cargo.toml workspace members
4. Follow naming convention: `feagi-{component-name}`
5. Add comprehensive tests

### Modifying Dependencies
1. Core crates should NOT depend on I/O layer
2. All platform-specific code must be feature-gated
3. Keep dependency tree minimal

### Code Review Checklist
- [ ] No hardcoded network addresses
- [ ] No hardcoded timeouts
- [ ] Platform-agnostic paths
- [ ] Proper error handling (no panics in production paths)
- [ ] Tests pass on all platforms
- [ ] Documentation updated

## References

- [State Manager Proposal](/tmp/RUST_STATE_MANAGER_MIGRATION_PROPOSAL.md)
- [FEAGI 2.0 Architecture Rules](/.cursorrules)
- [Contribution Guidelines](/CONTRIBUTING.md)

---

**Last Updated**: 2025-10-28  
**Status**: In Active Development

