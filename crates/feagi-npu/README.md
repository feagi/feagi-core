# FEAGI Neural Processing Unit (NPU)

This directory contains the core Neural Processing Unit subsystem of FEAGI, organized as a cohesive set of related crates.

## Crates

### Foundation
- **`neural/`** → [`feagi-npu-neural`](https://crates.io/crates/feagi-npu-neural)
  - Core types, algorithms, and neuron models
  - Platform-agnostic, no_std compatible
  - Foundation for all neural computation

### Runtime Abstraction
- **`runtime/`** → [`feagi-npu-runtime`](https://crates.io/crates/feagi-npu-runtime)
  - Runtime trait definitions (Runtime, NeuronStorage, SynapseStorage)
  - Platform-agnostic abstraction layer
  - no_std compatible

### Runtime Implementations
- **`runtime-std/`** → [`feagi-npu-runtime-std`](https://crates.io/crates/feagi-npu-runtime-std)
  - Desktop/Server implementation using `Vec`, `HashMap`
  - Rayon-based parallelism
  - Full std library support

- **`runtime-embedded/`** → [`feagi-npu-runtime-embedded`](https://crates.io/crates/feagi-npu-runtime-embedded)
  - Embedded/RTOS implementation using fixed arrays
  - no_std compatible
  - ESP32, Arduino, STM32, RPi Pico support

### Execution Engine
- **`burst-engine/`** → [`feagi-npu-burst-engine`](https://crates.io/crates/feagi-npu-burst-engine)
  - High-performance NPU execution engine
  - Burst loop runner
  - CPU, WGPU, and CUDA backends
  - Fire ledger, fire queue sampling
  - Sensory input processing

### Learning & Plasticity
- **`plasticity/`** → [`feagi-npu-plasticity`](https://crates.io/crates/feagi-npu-plasticity)
  - STDP (Spike-Timing-Dependent Plasticity)
  - Synaptic weight updates
  - Memory formation algorithms
  - Online learning mechanisms

## Architecture

```
┌─────────────────────────────────────────┐
│ feagi-npu-burst-engine                  │  ← Execution Layer (Inference)
│ (NPU runner, backends, burst loop)      │
└──────────┬──────────────────────────────┘
           │
           ├─→ feagi-npu-neural (algorithms)
           ├─→ feagi-npu-runtime (trait abstraction)
           └─→ feagi-npu-runtime-std (optional)

┌─────────────────────────────────────────┐
│ feagi-npu-plasticity                    │  ← Learning Layer (Training)
│ (STDP, weight updates, memory)          │
└──────────┬──────────────────────────────┘
           │
           └─→ feagi-npu-neural (types)
                    ↓
┌─────────────────────────────────────────┐
│ feagi-npu-runtime                       │  ← Abstraction Layer
│ (Runtime, NeuronStorage, SynapseStorage)│
└──────────┬──────────────────────────────┘
           │
           └─→ feagi-npu-neural (types)
                    ↓
┌─────────────────────────────────────────┐
│ feagi-npu-neural                        │  ← Foundation Layer
│ (Types, algorithms, neuron models)      │
└─────────────────────────────────────────┘
           │
           └─→ NO INTERNAL DEPENDENCIES ✅

Runtime Implementations:
┌────────────────────────┐  ┌──────────────────────────┐
│ feagi-npu-runtime-std  │  │ feagi-npu-runtime-       │
│ (Desktop/Server)       │  │ embedded (ESP32/RTOS)    │
└────────────────────────┘  └──────────────────────────┘
           │                            │
           └──────────┬─────────────────┘
                      ↓
           feagi-npu-runtime (implements traits)
```

## Usage

### Desktop/Server Application (Inference Only)
```toml
[dependencies]
feagi-npu-neural = "2.0"
feagi-npu-runtime-std = "2.0"
feagi-npu-burst-engine = "2.3"
```

```rust
use feagi_npu_neural::types::NeuronId;
use feagi_npu_runtime::Runtime;
use feagi_npu_runtime_std::StdRuntime;
use feagi_npu_burst_engine::RustNPU;

let runtime = StdRuntime::new();
let backend = CPUBackend::new();
let npu = RustNPU::new(runtime, backend, 1000, 10000, 20)?;
```

### Desktop/Server Application (With Learning)
```toml
[dependencies]
feagi-npu-neural = "2.0"
feagi-npu-runtime-std = "2.0"
feagi-npu-burst-engine = "2.3"
feagi-npu-plasticity = "2.0"  # Add learning
```

```rust
use feagi_npu_neural::types::NeuronId;
use feagi_npu_runtime_std::StdRuntime;
use feagi_npu_burst_engine::RustNPU;
use feagi_npu_plasticity::service::PlasticityService;

// NPU for inference
let npu = RustNPU::new(...)?;

// Plasticity for learning
let plasticity = PlasticityService::new(...);
```

### Embedded Application (ESP32, Arduino, etc.)
```toml
[dependencies]
feagi-npu-neural = { version = "2.0", default-features = false }
feagi-npu-runtime-embedded = { version = "2.0", default-features = false }
```

```rust
use feagi_npu_neural::types::NeuronId;
use feagi_npu_runtime_embedded::EmbeddedRuntime;

let runtime = EmbeddedRuntime::new();
// Use with feagi-embedded for platform-specific HALs
```

## Versioning

All NPU crates follow the **hybrid versioning strategy**:
- **Major versions synchronized** (all 2.x) → Compatibility guaranteed
- **Minor/patch versions independent** → Independent evolution

Example:
- `feagi-npu-neural` 2.0.0 (stable foundation)
- `feagi-npu-runtime` 2.0.0 (stable traits)
- `feagi-npu-runtime-std` 2.1.0 (optimization updates)
- `feagi-npu-burst-engine` 2.3.5 (active development)

**Compatibility Rule**: Any combination of 2.x versions works together.

## Contributing

See individual crate READMEs for detailed documentation.

## License

Apache-2.0 - See LICENSE file in repository root.

