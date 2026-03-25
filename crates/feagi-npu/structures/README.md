# feagi-runtime

**Runtime abstraction traits for cross-platform FEAGI neural processing**

Part of [FEAGI](https://github.com/feagi/feagi-core) - Framework for Evolutionary AGI

---

## Overview

`feagi-runtime` defines the **trait abstraction layer** that enables FEAGI's burst engine to run on different platforms without code changes.

**This crate contains ONLY trait definitions** - no concrete implementations.

### Supported Platforms (via runtime implementations)

| Runtime | Platform | Storage | Target |
|---------|----------|---------|--------|
| `feagi-runtime-std` | Desktop/Server | `Vec<T>` (dynamic) | x86_64, ARM64 |
| `feagi-runtime-embedded` | ESP32, Arduino, STM32 | `[T; N]` (fixed) | Embedded |
| `feagi-runtime-cuda` | NVIDIA GPU | `CudaSlice<T>` (GPU VRAM) | CUDA |
| `feagi-runtime-wasm` | Browser | `Float32Array` (typed) | WASM |

---

## Core Traits

### 1. Runtime Trait

Defines platform capabilities and creates storage:

```rust
pub trait Runtime: Send + Sync {
    type NeuronStorage<T: NeuralValue>: NeuronStorage<Value = T>;
    type SynapseStorage: SynapseStorage;
    
    fn create_neuron_storage<T: NeuralValue>(&self, capacity: usize) -> Result<Self::NeuronStorage<T>>;
    fn create_synapse_storage(&self, capacity: usize) -> Result<Self::SynapseStorage>;
    
    fn supports_parallel(&self) -> bool;
    fn memory_limit(&self) -> Option<usize>;
}
```

### 2. NeuronStorage Trait

Abstracts System-of-Arrays (SoA) for neurons:

```rust
pub trait NeuronStorage: Send + Sync {
    type Value: NeuralValue;
    
    fn membrane_potentials(&self) -> &[Self::Value];
    fn thresholds(&self) -> &[Self::Value];
    fn leak_coefficients(&self) -> &[f32];
    // ... 20+ properties
    
    fn add_neuron(&mut self, /* ... */) -> Result<usize>;
    fn add_neurons_batch(&mut self, /* ... */) -> Result<Vec<usize>>;
}
```

### 3. SynapseStorage Trait

Abstracts System-of-Arrays (SoA) for synapses:

```rust
pub trait SynapseStorage: Send + Sync {
    fn source_neurons(&self) -> &[u32];
    fn target_neurons(&self) -> &[u32];
    fn weights(&self) -> &[u8];
    // ... other properties
    
    fn add_synapse(&mut self, /* ... */) -> Result<usize>;
}
```

---

## Usage

### For Platform Implementers

Implement these traits for your platform:

```rust
use feagi_runtime::{Runtime, NeuronStorage, SynapseStorage};

// Your platform-specific storage
pub struct MyNeuronArray<T: NeuralValue> {
    membrane_potentials: MyPlatformVector<T>,
    // ...
}

impl<T: NeuralValue> NeuronStorage for MyNeuronArray<T> {
    type Value = T;
    
    fn membrane_potentials(&self) -> &[T] {
        self.membrane_potentials.as_slice()
    }
    
    // ... implement all required methods
}

// Your runtime
pub struct MyRuntime;

impl Runtime for MyRuntime {
    type NeuronStorage<T: NeuralValue> = MyNeuronArray<T>;
    type SynapseStorage = MySynapseArray;
    
    fn create_neuron_storage<T: NeuralValue>(&self, capacity: usize) -> Result<Self::NeuronStorage<T>> {
        Ok(MyNeuronArray::new(capacity))
    }
    
    fn supports_parallel(&self) -> bool { true }
    fn memory_limit(&self) -> Option<usize> { Some(1024 * 1024) }
}
```

### For Application Developers

Use concrete runtime implementations:

```rust
use feagi_runtime_std::StdRuntime;
use feagi_burst_engine::RustNPU;
use std::sync::Arc;

// Desktop runtime
let runtime = Arc::new(StdRuntime);
let npu = RustNPU::new(runtime, 1_000_000, 10_000_000)?;
npu.process_burst()?;
```

---

## Design Principles

### 1. Zero-Cost Abstractions

Traits compile to direct function calls (monomorphization):

```rust
// Generic code
fn process<R: Runtime>(runtime: &R) {
    let storage = runtime.create_neuron_storage(1000)?;
    // ... uses storage
}

// Compiles to platform-specific code (no vtables)
process(&StdRuntime);      // → Vec-based code
process(&EmbeddedRuntime); // → array-based code
```

### 2. Platform-Agnostic Burst Engine

Same burst processing code works everywhere:

```rust
// feagi-burst-engine works with any Runtime
pub struct RustNPU<R: Runtime, T: NeuralValue> {
    neuron_storage: RwLock<R::NeuronStorage<T>>,
    synapse_storage: RwLock<R::SynapseStorage>,
    runtime: Arc<R>,
}

// Used on desktop
let desktop_npu = RustNPU::<StdRuntime, f32>::new(/*...*/);

// Same code, different platform
let embedded_npu = RustNPU::<EmbeddedRuntime, f32>::new(/*...*/);
```

### 3. Type Safety

Compile-time guarantees for platform compatibility:

```rust
// This won't compile - embedded runtime has fixed capacity
let embedded = EmbeddedRuntime;
embedded.create_neuron_storage(1_000_000)?;  // ❌ Compile error

// This is correct
const MAX_NEURONS: usize = 10_000;
embedded.create_neuron_storage(MAX_NEURONS)?;  // ✅ OK
```

---

## Trait Contract

### NeuronStorage Contract

All implementations MUST guarantee:
1. ✅ Slice lengths match `count()` (not `capacity()`)
2. ✅ Thread-safe (Send + Sync)
3. ✅ Mutable slices are exclusive (Rust borrow checker enforces)
4. ✅ `add_neuron()` increments `count()`
5. ✅ Coordinates stored as flat array: `[x0, y0, z0, x1, y1, z1, ...]`

### SynapseStorage Contract

All implementations MUST guarantee:
1. ✅ Slice lengths match `count()`
2. ✅ Thread-safe (Send + Sync)
3. ✅ `source_neurons[i]` and `target_neurons[i]` are valid neuron IDs
4. ✅ Weights and PSPs are 0-255 (u8 range)

---

## Testing

Trait contracts are tested in each runtime implementation:

```bash
# Test std runtime
cd feagi-runtime-std && cargo test

# Test embedded runtime
cd feagi-runtime-embedded && cargo test

# Test CUDA runtime
cd feagi-runtime-cuda && cargo test
```

---

## Related Crates

| Crate | Purpose |
|-------|---------|
| `feagi-runtime` (this crate) | Trait definitions |
| `feagi-runtime-std` | Desktop/server implementation |
| `feagi-runtime-embedded` | ESP32/embedded implementation |
| `feagi-runtime-cuda` | NVIDIA GPU implementation |
| `feagi-runtime-wasm` | Browser/WASM implementation |
| `feagi-burst-engine` | Uses Runtime trait |

---

## Implementation Checklist

When implementing a new runtime:

- [ ] Create new crate `feagi-runtime-{platform}`
- [ ] Implement `NeuronStorage` trait
- [ ] Implement `SynapseStorage` trait
- [ ] Implement `Runtime` trait
- [ ] Add unit tests (all trait methods)
- [ ] Add integration tests (with feagi-burst-engine)
- [ ] Add benchmarks (vs CPU baseline)
- [ ] Document platform limitations
- [ ] Update this README with new runtime

---

## Contributing

New runtime implementations welcome! See [CONTRIBUTING.md](../../../CONTRIBUTING.md)

**Platforms we'd love to support**:
- FreeRTOS (real-time embedded)
- Distributed (MPI/gRPC clusters)
- Apple Neural Engine (ANE)
- Google TPU (Edge TPU)

---

## License

Licensed under Apache License 2.0

Copyright © 2025 Neuraville Inc.

---

## Links

- [FEAGI Project](https://github.com/feagi/feagi)
- [Documentation](https://docs.feagi.org)
- [Neural Processing Refactor Plan](../../NEURAL_PROCESSING_REFACTOR_PLAN.md)

