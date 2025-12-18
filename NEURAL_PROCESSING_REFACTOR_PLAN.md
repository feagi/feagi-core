# FEAGI Neural Processing Architecture Refactor Plan

**Date**: January 2025  
**Author**: Senior System Architect Review  
**Status**: Comprehensive Design Proposal  
**Scope**: Storage, Runtime, Backend Abstraction for Multi-Architecture Support

---

## Executive Summary

This document proposes a comprehensive refactor of FEAGI's neural processing crates to achieve proper hardware abstraction, enabling support for embedded systems (ESP32), desktop/server (std), HPC clusters (NVIDIA DGX), and future platforms (WASM, RTOS).

**Key Changes**:
1. **Storage Abstraction**: Move SoA (System of Arrays) from `feagi-types` to runtime crates via `Runtime` trait
2. **Runtime Integration**: Make `feagi-burst-engine` generic over `Runtime` trait
3. **Compute Backend**: Enhance existing `ComputeBackend` trait for multi-GPU (DGX) support
4. **Crate Consolidation**: Merge `feagi-types` + `feagi-synapse` → `feagi-neural` (reduce from 19 to 17 crates)

**Impact**: Enables true cross-platform support from ESP32 (2K neurons) to NVIDIA DGX (100M+ neurons) with zero code duplication.

---

## Table of Contents

1. [Current Architecture Analysis](#current-architecture-analysis)
2. [Problems Identified](#problems-identified)
3. [Proposed Architecture](#proposed-architecture)
4. [Neuron Models Architecture](#neuron-models-architecture)
5. [Detailed Refactor Plan](#detailed-refactor-plan)
6. [Migration Strategy](#migration-strategy)
7. [Testing Strategy](#testing-strategy)
8. [Future Expansion](#future-expansion)
9. [Example: WASM Runtime](#example-wasm-runtime)
10. [Risk Assessment](#risk-assessment)

---

## Current Architecture Analysis

### Current Crate Structure

```
feagi-core/
├── feagi-types/              ⚠️  Contains Vec-based storage + basic types (TO MERGE into feagi-neural)
├── feagi-neural/             ⚠️  Platform-agnostic but split from synapse and types
├── feagi-synapse/            ⚠️  Should be merged into feagi-neural
├── feagi-burst-engine/       ❌ Directly uses feagi-types::NeuronArray (std-only)
├── feagi-runtime-std/         ⚠️  Has own NeuronArray but NOT used by burst-engine
├── feagi-runtime-embedded/    ⚠️  Has own NeuronArray but NOT used by burst-engine
├── feagi-embedded/           ✅ Uses feagi-runtime-embedded (correct)
├── feagi-plasticity/         ✅ Independent (no storage dependency)
└── feagi-backend/            ✅ ComputeBackend trait exists (good foundation)
```

### Current Data Flow

```
feagi-burst-engine::RustNPU
  └── neuron_array: RwLock<feagi-types::NeuronArray<T>>  ❌ Vec-based, std-only
  └── synapse_array: RwLock<feagi-types::SynapseArray>    ❌ Vec-based, std-only
  └── backend: Mutex<Box<dyn ComputeBackend<T>>>         ✅ Good abstraction
```

**Problem**: `RustNPU` is locked to `std` because it directly holds `feagi-types::NeuronArray` which uses `Vec`.

### Current Runtime Crates (Unused)

```rust
// feagi-runtime-std/src/neuron_array.rs
pub struct NeuronArray<T: NeuralValue> {
    pub membrane_potentials: Vec<T>,  // ✅ Correct for std
    // ...
}

// feagi-runtime-embedded/src/neuron_array.rs
pub struct NeuronArray<T: NeuralValue, const N: usize> {
    pub membrane_potentials: [T; N],  // ✅ Correct for no_std
    // ...
}
```

**Problem**: These exist but `feagi-burst-engine` doesn't use them.

---

## Problems Identified

### 1. **Storage in Wrong Layer** ❌

**Current**: `feagi-types` contains `NeuronArray` and `SynapseArray` with `Vec<T>` (std-only)

**Impact**:
- Locks storage to `std` (can't use on ESP32)
- `feagi-burst-engine` can't run on embedded systems
- Runtime crates exist but aren't integrated

**Root Cause**: SoA (System of Arrays) is a **storage concern**, not a **type definition**. Types should define *what* a neuron is, not *how* we store many neurons.

**Additional Issue**: `feagi-types` itself is small and always used with `feagi-neural` → should be merged.

### 2. **Burst Engine Not Generic Over Runtime** ❌

**Current**: `RustNPU` directly holds `feagi-types::NeuronArray<T>`

```rust
pub struct RustNPU<T: NeuralValue> {
    pub(crate) neuron_array: RwLock<NeuronArray<T>>,  // ❌ Concrete type
    pub(crate) synapse_array: RwLock<SynapseArray>,   // ❌ Concrete type
}
```

**Impact**: Can't swap runtime implementations (std vs embedded vs CUDA)

### 3. **Runtime Crates Not Integrated** ⚠️

**Current**: `feagi-runtime-std` and `feagi-runtime-embedded` have their own `NeuronArray`/`SynapseArray` but:
- Not used by `feagi-burst-engine`
- Not used by any other crate except `feagi-embedded` (which uses embedded version)

**Impact**: Code duplication, maintenance burden

### 4. **No HPC Runtime Support** ❌

**Current**: No runtime crate for CUDA/GPU storage

**Impact**: Can't efficiently run on NVIDIA DGX (data must stay in GPU VRAM)

### 5. **Compute Backend Not Fully Utilized** ⚠️

**Current**: `ComputeBackend` trait exists but:
- Only handles compute (synaptic propagation, neural dynamics)
- Doesn't handle storage (data transfer, memory management)
- No multi-GPU support

**Impact**: Can't efficiently use multiple GPUs on DGX

---

## Proposed Architecture

### Architecture Principles

1. **Separation of Concerns**:
   - **Types & Algorithms** (`feagi-neural`): Define *what* neurons/synapses *are* AND *how* they compute (merged)
   - **Storage** (`feagi-runtime-*`): Define *how* we store *many* neurons/synapses (SoA implementations)
   - **Orchestration** (`feagi-burst-engine`): Define *how* we *coordinate* (burst cycle)

2. **Trait-Based Abstraction**:
   - `Runtime` trait: Abstracts storage (SoA implementation)
   - `ComputeBackend` trait: Abstracts compute (CPU/GPU/CUDA)

3. **Zero-Cost Abstractions**:
   - Traits compile to direct function calls (no runtime overhead)
   - Generic code monomorphizes per runtime (optimized per platform)

### Proposed Crate Structure (Updated)

**Note**: After analysis, **feagi-types should be merged into feagi-neural** to reduce crate count and improve cohesion.

```
feagi-core/
├── feagi-neural/                  ✅ ALL neural computation (types + algorithms + models)
│   ├── types/                     🔄 MERGED from feagi-types
│   ├── synapse/                   🔄 MERGED from feagi-synapse
│   ├── dynamics/                  ✅ Existing
│   ├── models/                    🔄 MOVED from burst-engine
│
├── feagi-runtime/                  🆕 Trait definitions (Runtime, Storage traits)
│   └── traits.rs                   🆕 Runtime trait, Storage trait
│
├── feagi-runtime-std/              ✅ Vec-based storage (std)
│   ├── neuron_array.rs            ✅ Move from feagi-types
│   ├── synapse_array.rs           ✅ Move from feagi-types
│   └── runtime.rs                  🆕 Runtime implementation
│
├── feagi-runtime-embedded/         ✅ Fixed-array storage (no_std)
│   ├── neuron_array.rs            ✅ Keep existing
│   ├── synapse_array.rs            ✅ Keep existing
│   └── runtime.rs                  🆕 Runtime implementation
│
├── feagi-runtime-cuda/             🆕 CUDA storage (GPU VRAM)
│   ├── neuron_array.rs            🆕 CudaSlice-based SoA
│   ├── synapse_array.rs           🆕 CudaSlice-based SoA
│   └── runtime.rs                  🆕 Runtime implementation
│
├── feagi-burst-engine/             ✅ Generic over Runtime trait
│   └── npu.rs                      🔄 RustNPU<R: Runtime, T: NeuralValue>
│
├── feagi-backend/                  ✅ ComputeBackend trait (enhanced)
│   ├── cpu.rs                      ✅ CPU backend
│   ├── cuda.rs                     ✅ CUDA backend (enhanced for multi-GPU)
│   └── wgpu.rs                     ✅ WGPU backend
│
├── feagi-embedded/                 ✅ Uses feagi-runtime-embedded (no change)
└── feagi-plasticity/               ✅ Independent (no change)
```

### Proposed Trait Hierarchy

```rust
// feagi-runtime/traits.rs

/// Runtime trait: Abstracts storage and platform capabilities
pub trait Runtime: Send + Sync {
    type NeuronArray: NeuronStorage;
    type SynapseArray: SynapseStorage;
    
    /// Create neuron array with capacity
    fn create_neuron_array<T: NeuralValue>(&self, capacity: usize) -> Self::NeuronArray;
    
    /// Create synapse array with capacity
    fn create_synapse_array(&self, capacity: usize) -> Self::SynapseArray;
    
    /// Platform capabilities
    fn supports_parallel(&self) -> bool;
    fn supports_simd(&self) -> bool;
    fn memory_limit(&self) -> Option<usize>;
}

/// Neuron storage trait: Abstracts SoA for neurons
pub trait NeuronStorage: Send + Sync {
    type Value: NeuralValue;
    
    /// Get membrane potentials slice
    fn membrane_potentials(&self) -> &[Self::Value];
    fn membrane_potentials_mut(&mut self) -> &mut [Self::Value];
    
    /// Get thresholds slice
    fn thresholds(&self) -> &[Self::Value];
    
    /// Get leak coefficients slice
    fn leak_coefficients(&self) -> &[f32];
    
    /// Get refractory countdowns slice
    fn refractory_countdowns(&self) -> &[u16];
    fn refractory_countdowns_mut(&mut self) -> &mut [u16];
    
    /// Get count
    fn count(&self) -> usize;
    fn capacity(&self) -> usize;
    
    /// Add neuron (returns index)
    fn add_neuron(&mut self, /* params */) -> Result<usize>;
    
    /// Batch add neurons
    fn add_neurons_batch(&mut self, /* params */) -> Result<Vec<usize>>;
    
    // ... other neuron properties
}

/// Synapse storage trait: Abstracts SoA for synapses
pub trait SynapseStorage: Send + Sync {
    /// Get source neurons slice
    fn source_neurons(&self) -> &[u32];
    
    /// Get target neurons slice
    fn target_neurons(&self) -> &[u32];
    
    /// Get weights slice
    fn weights(&self) -> &[u8];
    
    /// Get postsynaptic potentials slice
    fn postsynaptic_potentials(&self) -> &[u8];
    
    /// Get count
    fn count(&self) -> usize;
    fn capacity(&self) -> usize;
    
    /// Add synapse
    fn add_synapse(&mut self, /* params */) -> Result<usize>;
    
    // ... other synapse properties
}
```

### Proposed Burst Engine Structure

```rust
// feagi-burst-engine/src/npu.rs

use feagi_runtime::Runtime;

/// Generic NPU over runtime
pub struct RustNPU<R: Runtime, T: NeuralValue> {
    // Runtime-provided storage
    pub(crate) neuron_array: RwLock<R::NeuronArray>,
    pub(crate) synapse_array: RwLock<R::SynapseArray>,
    
    // Runtime instance (for creating new arrays if needed)
    runtime: Arc<R>,
    
    // Fire structures (platform-agnostic)
    pub(crate) fire_structures: Mutex<FireStructures>,
    
    // Compute backend (CPU/GPU/CUDA)
    pub(crate) backend: Mutex<Box<dyn ComputeBackend<T>>>,
    
    // ... other fields
}

impl<R: Runtime, T: NeuralValue> RustNPU<R, T> {
    pub fn new(
        runtime: Arc<R>,
        neuron_capacity: usize,
        synapse_capacity: usize,
        backend_type: BackendType,
    ) -> Result<Self> {
        let neuron_array = runtime.create_neuron_array::<T>(neuron_capacity);
        let synapse_array = runtime.create_synapse_array(synapse_capacity);
        
        // ... rest of initialization
    }
    
    pub fn process_burst(&self) -> Result<BurstResult> {
        // Lock storage
        let mut neuron_array = self.neuron_array.write()?;
        let synapse_array = self.synapse_array.read()?;
        
        // Get compute backend
        let mut backend = self.backend.lock()?;
        
        // Process burst (backend works with storage via traits)
        backend.process_burst(
            &*neuron_array,
            &*synapse_array,
            // ...
        )
    }
}
```

### Proposed Runtime Implementations

#### 1. Standard Runtime (std)

```rust
// feagi-runtime-std/src/runtime.rs

pub struct StdRuntime;

impl Runtime for StdRuntime {
    type NeuronArray = StdNeuronArray;
    type SynapseArray = StdSynapseArray;
    
    fn create_neuron_array<T: NeuralValue>(&self, capacity: usize) -> Self::NeuronArray {
        StdNeuronArray::new(capacity)
    }
    
    fn create_synapse_array(&self, capacity: usize) -> Self::SynapseArray {
        StdSynapseArray::new(capacity)
    }
    
    fn supports_parallel(&self) -> bool { true }
    fn supports_simd(&self) -> bool { true }
    fn memory_limit(&self) -> Option<usize> { None }
}

// feagi-runtime-std/src/neuron_array.rs
pub struct StdNeuronArray<T: NeuralValue> {
    pub membrane_potentials: Vec<T>,
    pub thresholds: Vec<T>,
    // ... (moved from feagi-types)
}

impl<T: NeuralValue> NeuronStorage for StdNeuronArray<T> {
    type Value = T;
    
    fn membrane_potentials(&self) -> &[T] { &self.membrane_potentials }
    fn membrane_potentials_mut(&mut self) -> &mut [T] { &mut self.membrane_potentials }
    // ... implement all trait methods
}
```

#### 2. Embedded Runtime (no_std)

```rust
// feagi-runtime-embedded/src/runtime.rs

pub struct EmbeddedRuntime;

impl Runtime for EmbeddedRuntime {
    type NeuronArray = EmbeddedNeuronArray;
    type SynapseArray = EmbeddedSynapseArray;
    
    fn create_neuron_array<T: NeuralValue, const N: usize>(&self) -> Self::NeuronArray {
        EmbeddedNeuronArray::new()
    }
    
    // ... (similar to std)
}

// feagi-runtime-embedded/src/neuron_array.rs
pub struct EmbeddedNeuronArray<T: NeuralValue, const N: usize> {
    pub membrane_potentials: [T; N],
    // ... (existing implementation)
}

impl<T: NeuralValue, const N: usize> NeuronStorage for EmbeddedNeuronArray<T, N> {
    // ... implement trait methods
}
```

#### 3. CUDA Runtime (GPU VRAM)

```rust
// feagi-runtime-cuda/src/runtime.rs

pub struct CudaRuntime {
    device: Arc<CudaDevice>,
}

impl Runtime for CudaRuntime {
    type NeuronArray = CudaNeuronArray;
    type SynapseArray = CudaSynapseArray;
    
    fn create_neuron_array<T: NeuralValue>(&self, capacity: usize) -> Self::NeuronArray {
        CudaNeuronArray::new(self.device.clone(), capacity)
    }
    
    // ...
}

// feagi-runtime-cuda/src/neuron_array.rs
pub struct CudaNeuronArray<T: NeuralValue> {
    pub membrane_potentials: CudaSlice<T>,  // GPU memory
    pub thresholds: CudaSlice<T>,
    // ...
}

impl<T: NeuralValue> NeuronStorage for CudaNeuronArray<T> {
    fn membrane_potentials(&self) -> &[T] {
        // Download from GPU (or use pinned memory)
        self.membrane_potentials.as_slice()
    }
    
    fn membrane_potentials_mut(&mut self) -> &mut [T] {
        // Upload to GPU (or use pinned memory)
        self.membrane_potentials.as_mut_slice()
    }
    
    // ... implement trait methods
}
```

### Enhanced Compute Backend for Multi-GPU

```rust
// feagi-backend/src/cuda_backend.rs

pub struct CUDABackend {
    devices: Vec<Arc<CudaDevice>>,  // Multiple GPUs
    neuron_partitions: Vec<(usize, usize)>,  // Per-GPU neuron ranges
    synapse_partitions: Vec<(usize, usize)>, // Per-GPU synapse ranges
}

impl<T: NeuralValue> ComputeBackend<T> for CUDABackend {
    fn process_synaptic_propagation(
        &mut self,
        fired_neurons: &[u32],
        synapse_array: &dyn SynapseStorage,  // ✅ Trait-based
        fcl: &mut FireCandidateList,
    ) -> Result<usize> {
        // Partition work across GPUs
        let chunks: Vec<_> = self.devices.iter().zip(self.synapse_partitions.iter())
            .map(|(device, (start, end))| {
                // Process synapses[start..end] on this GPU
                self.process_synapses_on_device(device, start, end, fired_neurons)
            })
            .collect();
        
        // Merge results into FCL
        // ...
    }
    
    fn process_neural_dynamics(
        &mut self,
        fcl: &FireCandidateList,
        neuron_array: &mut dyn NeuronStorage,  // ✅ Trait-based
        burst_count: u64,
    ) -> Result<(Vec<u32>, usize, usize)> {
        // Partition neurons across GPUs
        // ...
    }
}
```

---

## Should feagi-types Be Merged?

### Analysis: Yes - Merge into feagi-neural ✅

**After Refactor, feagi-types contains**:
- ✅ `NeuronId`, `SynapseId` (simple wrappers)
- ✅ `NeuralValue` trait
- ✅ `SynapseType`, `Synapse` (single item)
- ✅ `Dimensions`, error types
- ❌ `CorticalAreaId` (to be removed - use feagi_data_structures::CorticalID)
- ❌ `NeuronArray`, `SynapseArray` (moving to runtime crates)

**Result**: Very small crate (~500 LOC) with just basic types and traits.

### Recommendation: Merge feagi-types → feagi-neural

**Justification**:
1. ✅ **Related content**: Types that define neurons + algorithms that process them belong together
2. ✅ **Precedent**: feagi_data_structures does the same (types + processors)
3. ✅ **Reduces crates**: 19 → 17 crates (cleaner)
4. ✅ **Logical cohesion**: "All neural computation in one place"
5. ✅ **no_std compatible**: Both are no_std, so no conflict
6. ✅ **Everyone uses both**: No consumer uses feagi-types without feagi-neural

**New Structure**:
```
feagi-neural/
├── types/              🔄 MERGED from feagi-types
│   ├── ids.rs          (NeuronId, SynapseId)
│   ├── values.rs       (NeuralValue trait, INT8Value, Precision)
│   ├── synapse.rs      (SynapseType, Synapse, SynapticWeight, etc.)
│   ├── spatial.rs      (Dimensions, Position)
│   └── error.rs        (FeagiError, Result<T>)
├── synapse/            🔄 MERGED from feagi-synapse
│   ├── contribution.rs
│   └── weight.rs
├── dynamics.rs         ✅ Existing
├── firing.rs           ✅ Existing
├── utils.rs            ✅ Existing
└── models/             🔄 MOVED from burst-engine
    ├── traits.rs
    ├── lif.rs
    └── izhikevich.rs
```

**Import Example**:
```rust
// Before (3 crates)
use feagi_types::{NeuronId, SynapseType, NeuralValue};
use feagi_synapse::compute_contribution;
use feagi_neural::update_membrane_potential;

// After (1 crate)
use feagi_neural::{
    types::{NeuronId, SynapseType, NeuralValue},
    synapse::compute_contribution,
    dynamics::update_membrane_potential,
    models::LeakyIntegrateAndFire,
};
```

**New Hierarchy**:
```
feagi_data_structures (genome layer - external)
  ↓ used by
feagi-neural (ALL neural computation: types + algorithms + models)
  ↓ used by
feagi-runtime-* (storage implementations)
  ↓ used by
feagi-burst-engine (orchestration)
```

**Crate Count Reduction**:
- Before refactor: 19 crates
- After merging feagi-types + feagi-synapse into feagi-neural: 17 crates
- After moving I/O to feagi-io: 14 crates

**Benefits**:
1. **Simpler**: One import for all neural types/algorithms
2. **Cohesive**: Related code together
3. **Fewer dependencies**: Consumers depend on 1 crate instead of 3
4. **Cleaner architecture**: Clear layers (genome → neural → runtime → execution)

### Updated Layer Architecture

```
┌────────────────────────────────────────────────┐
│ feagi_data_structures (Genome Layer - External)│
│  - CorticalID (authoritative)                  │
│  - Genomic structures                          │
│  - NeuronVoxelXYZP (for I/O)                   │
└────────────────────┬───────────────────────────┘
                     │ used by
┌────────────────────▼───────────────────────────┐
│ feagi-neural (Neural Computation - All in One) │
│  ├── types/      (NeuronId, SynapseType, etc.) │
│  ├── synapse/    (synaptic algorithms)         │
│  ├── dynamics/   (membrane potential)          │
│  └── models/     (LIF, Izhikevich, etc.)       │
└────────────────────┬───────────────────────────┘
                     │ used by
┌────────────────────▼───────────────────────────┐
│ feagi-runtime (Trait Definitions)              │
│  - Runtime trait                               │
│  - Storage traits                              │
└────────────────────┬───────────────────────────┘
                     │ used by
┌────────────────────▼───────────────────────────┐
│ feagi-runtime-* (Storage Implementations)      │
│  - runtime-std (Vec-based)                     │
│  - runtime-embedded (array-based)              │
│  - runtime-cuda (GPU VRAM)                     │
└────────────────────┬───────────────────────────┘
                     │ used by
┌────────────────────▼───────────────────────────┐
│ feagi-burst-engine (Orchestration)             │
│  - Burst cycle coordination                    │
│  - Generic over Runtime + NeuronModel          │
└────────────────────────────────────────────────┘
```

---

## Neuron Models Architecture

### Where Do Neuron Models Fit?

**Neuron models** (Leaky Integrate-and-Fire, Izhikevich, Hodgkin-Huxley, etc.) are **platform-agnostic algorithms** that define the computational behavior of neurons.

### Current State (Scattered Architecture)

**Neuron models** currently live in `feagi-burst-engine/src/neuron_models/`:
**Synaptic algorithms** currently live in `feagi-synapse/`:
**Neural algorithms** currently live in `feagi-neural/`:

This creates unnecessary separation for tightly coupled logic.

```rust
// feagi-burst-engine/src/neuron_models/traits.rs
pub trait NeuronModel {
    type Parameters;
    
    fn model_name(&self) -> &'static str;
    
    fn compute_synaptic_contribution(&self, weight: f32, psp: f32, synapse_type: SynapseType) -> f32;
    
    fn update_membrane_potential(&self, potential: &mut f32, leak: f32, resting: f32, input: f32);
    
    fn check_firing(&self, potential: f32, threshold: f32, excitability: f32, burst: u64) -> bool;
    
    fn post_fire_reset(&self, potential: &mut f32, resting: f32);
}

// feagi-burst-engine/src/neuron_models/lif.rs
pub struct LeakyIntegrateAndFire;

impl NeuronModel for LeakyIntegrateAndFire {
    type Parameters = LIFParameters;
    
    fn model_name(&self) -> &'static str { "Leaky Integrate-and-Fire (LIF)" }
    
    fn update_membrane_potential(&self, potential: &mut f32, leak: f32, resting: f32, input: f32) {
        // Apply leak: V(t+1) = V(t) + leak * (resting - V(t)) + input
        *potential += leak * (resting - *potential) + input;
    }
    
    // ... other methods
}
```

**Problem**: 
1. Neuron models are in `feagi-burst-engine` but should be platform-agnostic
2. Synaptic logic is in separate `feagi-synapse` crate but always used with `feagi-neural`
3. Neuron models need BOTH synaptic and neural logic - they shouldn't be split

### Proposed Architecture

**Merge `feagi-synapse` into `feagi-neural` and add neuron models**:

**Justification for Merging feagi-synapse**:
1. ✅ **Always used together**: 0 out of 4 consumers use them separately
2. ✅ **Same abstraction level**: Both are platform-agnostic, no_std algorithms
3. ✅ **Neuron models need both**: LIF model requires synaptic contribution AND membrane dynamics
4. ✅ **Small size**: Combined ~1,005 LOC (still tiny)
5. ✅ **Cleaner dependencies**: One crate for all neural computation

**New Structure**:

```
feagi-neural/
├── lib.rs               ✅ Main entry point
├── synapse/             🔄 MERGED from feagi-synapse
│   ├── mod.rs           🔄 Move from feagi-synapse
│   ├── contribution.rs  🔄 Move from feagi-synapse (synaptic contribution calc)
│   └── weight.rs        🔄 Move from feagi-synapse (weight handling)
├── dynamics.rs          ✅ Existing: membrane potential updates
├── firing.rs            ✅ Existing: firing condition checks
├── utils.rs             ✅ Existing: helper functions
└── models/              🆕 Neuron model trait system (uses synapse + dynamics)
    ├── mod.rs           🆕 Re-exports
    ├── traits.rs        🔄 Move from burst-engine
    ├── lif.rs           🔄 Move from burst-engine
    ├── izhikevich.rs    🔄 Move from burst-engine (if exists)
    └── adaptive.rs      🔄 Move from burst-engine (if exists)
```

**Key Changes**:
- ❌ **Remove**: `feagi-synapse` as separate crate
- ➕ **Add**: `feagi-neural/src/synapse/` submodule (merged from feagi-synapse)
- ➕ **Add**: `feagi-neural/src/models/` submodule (moved from burst-engine)
- ✅ **Result**: Single crate for all neural/synaptic computation

### Architecture Layers for Neuron Models

```
┌─────────────────────────────────────────────────────────────┐
│ feagi_data_structures (GENOME LAYER - external)             │
│   - CorticalID (authoritative, 8-byte)                      │
│   - Genomic structures                                       │
└─────────────────────────────────────────────────────────────┘
                            ↑ uses
┌─────────────────────────────────────────────────────────────┐
│ feagi-neural (ALL NEURAL COMPUTATION)                       │
│   ┌─────────────────────────────────────────────────────┐   │
│   │ types/ (MERGED from feagi-types)                     │   │
│   │   - NeuronId, SynapseId                              │   │
│   │   - NeuralValue trait (f32, INT8Value, etc.)         │   │
│   │   - SynapseType enum (Excitatory, Inhibitory)        │   │
│   │   - Error types                                      │   │
│   └─────────────────────────────────────────────────────┘   │
│   ┌─────────────────────────────────────────────────────┐   │
│   │ synapse/ (MERGED from feagi-synapse)                 │   │
│   │   - compute_synaptic_contribution()                  │   │
│   │   - weight handling                                  │   │
│   │   - PSP calculations                                 │   │
│   └─────────────────────────────────────────────────────┘   │
│   ┌─────────────────────────────────────────────────────┐   │
│   │ models/ (neuron models use types + synapse + dynamics) │
│   │   - NeuronModel trait                                │   │
│   │   - LeakyIntegrateAndFire (LIF)                      │   │
│   │   - Izhikevich                                       │   │
│   │   - AdaptiveExponentialIF                            │   │
│   │   - HodgkinHuxley (future)                           │   │
│   └─────────────────────────────────────────────────────┘   │
│   - dynamics.rs (membrane potential updates)                 │
│   - firing.rs (firing condition checks)                      │
│   - utils.rs (helper functions)                              │
└─────────────────────────────────────────────────────────────┘
                            ↑ uses
┌─────────────────────────────────────────────────────────────┐
│ feagi-runtime-* (STORAGE IMPLEMENTATIONS)                   │
│   - NeuronArray (Vec, fixed array, CudaSlice, etc.)         │
│   - SynapseArray                                             │
│   - Runtime trait implementation                             │
└─────────────────────────────────────────────────────────────┘
                            ↑ uses
┌─────────────────────────────────────────────────────────────┐
│ feagi-burst-engine (ORCHESTRATION)                          │
│   - RustNPU (coordinates burst cycle)                       │
│   - Selects neuron model at runtime or compile-time         │
│   - Uses Runtime trait for storage                          │
│   - Uses ComputeBackend trait for compute                   │
└─────────────────────────────────────────────────────────────┘
```

### Usage Example: Selecting Neuron Model

```rust
use feagi_neural::models::{NeuronModel, LeakyIntegrateAndFire, Izhikevich};
use feagi_neural::synapse;  // ✅ Now in same crate
use feagi_runtime_std::StdRuntime;
use feagi_burst_engine::RustNPU;

// Option 1: Compile-time selection (zero overhead)
fn create_npu_lif() -> RustNPU<StdRuntime, f32, LeakyIntegrateAndFire> {
    let runtime = Arc::new(StdRuntime);
    let model = LeakyIntegrateAndFire::default();
    
    RustNPU::new_with_model(
        runtime,
        1_000_000,  // neurons
        10_000_000, // synapses
        model,
        BackendType::Auto,
    ).unwrap()
}

// Option 2: Runtime selection (dynamic dispatch)
fn create_npu_dynamic(model_name: &str) -> RustNPU<StdRuntime, f32, Box<dyn NeuronModel>> {
    let runtime = Arc::new(StdRuntime);
    
    let model: Box<dyn NeuronModel> = match model_name {
        "lif" => Box::new(LeakyIntegrateAndFire::default()),
        "izhikevich" => Box::new(Izhikevich::default()),
        _ => panic!("Unknown model"),
    };
    
    RustNPU::new_with_model(
        runtime,
        1_000_000,
        10_000_000,
        model,
        BackendType::Auto,
    ).unwrap()
}
```

### Neuron Model Trait Design (Enhanced)

```rust
// feagi-neural/src/models/traits.rs

/// Core trait for neuron model computational behavior
pub trait NeuronModel: Send + Sync {
    /// Model-specific parameters
    type Parameters: Default + Clone;
    
    /// Get model name for logging/debugging
    fn model_name(&self) -> &'static str;
    
    /// Compute synaptic contribution (weight × PSP)
    fn compute_synaptic_contribution(
        &self,
        weight: f32,
        psp: f32,
        synapse_type: SynapseType,
    ) -> f32;
    
    /// Update membrane potential (core dynamics)
    fn update_membrane_potential<T: NeuralValue>(
        &self,
        potential: &mut T,
        leak: f32,
        resting: T,
        input: T,
        params: &Self::Parameters,
    );
    
    /// Check if neuron should fire
    fn check_firing<T: NeuralValue>(
        &self,
        potential: T,
        threshold: T,
        excitability: f32,
        burst: u64,
        params: &Self::Parameters,
    ) -> bool;
    
    /// Reset potential after firing
    fn post_fire_reset<T: NeuralValue>(
        &self,
        potential: &mut T,
        resting: T,
        params: &Self::Parameters,
    );
    
    /// Get default parameters
    fn default_parameters(&self) -> Self::Parameters {
        Self::Parameters::default()
    }
}
```

### Why Merge feagi-synapse into feagi-neural?

**From [CRATE_STRUCTURE_ANALYSIS.md](CRATE_STRUCTURE_ANALYSIS.md)**:
> **Decision**: **MERGE neural + synapse → feagi-neural**
> 
> **Justification**:
> - Both are no_std, pure computation
> - ALWAYS used together (4/4 consumers use both, 0/4 use alone)
> - Combined = 1,005 LOC (still tiny)
> - Same purpose: core neural computation primitives
> - No circular dependency risk

**Additional Reasons**:
1. **Neuron models need both**: LIF model computes synaptic contributions AND membrane dynamics
2. **Platform-Agnostic**: All logic works the same on ESP32, desktop, and DGX
3. **Pure Computation**: No I/O, no allocation, no platform dependencies
4. **no_std Compatible**: Can be used in embedded systems
5. **Reusable**: Can be used by burst-engine, plasticity, BDU, etc.
6. **Testable**: Easy to unit test in isolation
7. **Simpler Dependencies**: One import instead of two

### Integration with Burst Engine

```rust
// feagi-burst-engine/src/npu.rs

pub struct RustNPU<R: Runtime, T: NeuralValue, M: NeuronModel> {
    // Runtime-provided storage
    neuron_array: RwLock<R::NeuronArray>,
    synapse_array: RwLock<R::SynapseArray>,
    
    // Neuron model (algorithm)
    model: Arc<M>,
    model_params: Vec<M::Parameters>,  // Per-neuron parameters
    
    // Runtime & backend
    runtime: Arc<R>,
    backend: Mutex<Box<dyn ComputeBackend<T>>>,
    
    // ... other fields
}

impl<R: Runtime, T: NeuralValue, M: NeuronModel> RustNPU<R, T, M> {
    pub fn process_burst(&self) -> Result<BurstResult> {
        // Lock storage
        let mut neurons = self.neuron_array.write()?;
        let synapses = self.synapse_array.read()?;
        
        // Use model for dynamics
        for idx in fcl.iter() {
            let params = &self.model_params[idx];
            
            // Update potential using model
            self.model.update_membrane_potential(
                &mut neurons.membrane_potentials_mut()[idx],
                neurons.leak_coefficients()[idx],
                neurons.resting_potentials()[idx],
                fcl.get_potential(idx),
                params,
            );
            
            // Check firing using model
            if self.model.check_firing(
                neurons.membrane_potentials()[idx],
                neurons.thresholds()[idx],
                neurons.excitabilities()[idx],
                burst_count,
                params,
            ) {
                fired.push(idx);
                
                // Reset using model
                self.model.post_fire_reset(
                    &mut neurons.membrane_potentials_mut()[idx],
                    neurons.resting_potentials()[idx],
                    params,
                );
            }
        }
        
        // ... rest of burst cycle
    }
}
```

### Neuron Models Roadmap

**Phase 1 (Current)**: Move existing models to `feagi-neural`
- ✅ LeakyIntegrateAndFire (LIF) - already implemented
- ✅ NeuronModel trait - already defined

**Phase 2 (Near-term)**: Add common models
- 🆕 Izhikevich (simple + efficient)
- 🆕 AdaptiveExponentialIF (AdEx)
- 🆕 QuadraticIF (QIF)

**Phase 3 (Future)**: Add advanced models
- 🔮 Hodgkin-Huxley (HH) - biologically accurate
- 🔮 FitzHugh-Nagumo (FHN)
- 🔮 Multi-compartment models

**Phase 4 (Research)**: Add learning models
- 🔮 Spike Response Model (SRM)
- 🔮 Generalized LIF with adaptation
- 🔮 Stochastic models

---

## Detailed Refactor Plan

### Phase 1: Create Runtime Trait Crate (Week 1)

**Goal**: Define abstraction layer without breaking existing code

**Tasks**:
1. Create `feagi-runtime` crate with trait definitions
2. Define `Runtime`, `NeuronStorage`, `SynapseStorage` traits
3. Add comprehensive documentation and examples
4. Add unit tests for trait contracts

**Files Created**:
- `feagi-runtime/Cargo.toml`
- `feagi-runtime/src/lib.rs`
- `feagi-runtime/src/traits.rs`
- `feagi-runtime/src/error.rs`
- `feagi-runtime/README.md`

**Dependencies**: None (pure trait definitions)

**Testing**: Trait contract tests (ensure all methods are callable)

---

### Phase 2: Consolidate Crates and Move Storage (Week 2)

**Goal**: Merge feagi-types + feagi-synapse → feagi-neural, move storage to runtime crates

**Tasks**:

#### 2.1 Extract storage from feagi-types (preparation)
- Identify `NeuronArray` and `SynapseArray` in `feagi-types/src/npu.rs`
- Document what remains: `NeuronId`, `SynapseId`, `NeuralValue`, `SynapseType`, error types
- Prepare for merge into feagi-neural

#### 2.2 Merge feagi-types, feagi-synapse into feagi-neural and move neuron models

**MAJOR CONSOLIDATION**: Merge 3 crates into 1

- **Merge feagi-types**: Move `feagi-types/src/` to `feagi-neural/src/types/`
  - Exclude: `CorticalAreaId` (remove - use feagi_data_structures::CorticalID)
  - Exclude: `NeuronArray`, `SynapseArray` (moving to runtime crates)
  - Exclude: `cortical_id_decoder.rs`, `cortical_type_adapter.rs` (no longer needed)
  - Include: `NeuronId`, `SynapseId`, `NeuralValue`, `SynapseType`, `Dimensions`, error types
- **Merge feagi-synapse**: Move `feagi-synapse/src/` to `feagi-neural/src/synapse/`
- **Move neuron models**: Move `feagi-burst-engine/src/neuron_models/` to `feagi-neural/src/models/`
- Update imports in all consumers:
  - `feagi-types::*` → `feagi-neural::types::*`
  - `feagi-synapse::*` → `feagi-neural::synapse::*`
  - `feagi-burst-engine::neuron_models::*` → `feagi-neural::models::*`
- Update `feagi-neural/Cargo.toml`:
  - Add feagi_data_structures dependency (for CorticalID)
  - No other new dependencies
- Ensure no_std compatibility for all merged code
- Update tests
- **Remove feagi-types and feagi-synapse crates** from workspace

#### 2.3 Update feagi-runtime-std
- Move `NeuronArray` and `SynapseArray` from `feagi-types` to `feagi-runtime-std`
- Rename to `StdNeuronArray` and `StdSynapseArray`
- Implement `NeuronStorage` and `SynapseStorage` traits
- Implement `Runtime` trait for `StdRuntime`
- Update tests

#### 2.4 Update feagi-runtime-embedded
- Rename existing `NeuronArray` to `EmbeddedNeuronArray`
- Implement `NeuronStorage` and `SynapseStorage` traits
- Implement `Runtime` trait for `EmbeddedRuntime`
- Update tests

**Files Modified**:
- `feagi-neural/src/types/` (merge from feagi-types, excluding CorticalAreaId and storage)
- `feagi-neural/src/synapse/` (merge from feagi-synapse crate)
- `feagi-neural/src/models/` (move from burst-engine)
- `feagi-neural/src/lib.rs` (add types + synapse + models modules)
- `feagi-neural/Cargo.toml` (add feagi_data_structures dependency)
- `feagi-burst-engine/src/lib.rs` (remove neuron_models, use feagi-neural::models)
- `feagi-burst-engine/Cargo.toml` (replace feagi-types + feagi-synapse with feagi-neural)
- `feagi-types/src/npu.rs` (move NeuronArray/SynapseArray to runtime crates, then delete crate)
- `feagi-runtime-std/src/neuron_array.rs` (move from feagi-types)
- `feagi-runtime-std/src/synapse_array.rs` (move from feagi-types)
- `feagi-runtime-std/src/runtime.rs` (new)
- `feagi-runtime-std/Cargo.toml` (replace feagi-types + feagi-synapse with feagi-neural)
- `feagi-runtime-embedded/src/neuron_array.rs` (rename, add traits)
- `feagi-runtime-embedded/src/synapse_array.rs` (rename, add traits)
- `feagi-runtime-embedded/src/runtime.rs` (new)
- `feagi-runtime-embedded/Cargo.toml` (replace feagi-types + feagi-synapse with feagi-neural)
- All consumers: Replace `feagi-types::*` with `feagi-neural::types::*`

**Files Deleted**:
- `feagi-types/` (entire crate merged into feagi-neural)
- `feagi-synapse/` (entire crate merged into feagi-neural)

**Workspace Updates**:
- `feagi-core/Cargo.toml` (remove feagi-types and feagi-synapse from members)

**Breaking Changes**: 
1. Yes - `feagi-types` crate removed (merged into `feagi-neural`)
2. Yes - `feagi-synapse` crate removed (merged into `feagi-neural`)
3. Yes - `CorticalAreaId` removed (use `feagi_data_structures::CorticalID`)

**Migration Guide**: 
```rust
// Type migration
// Before
use feagi_types::{NeuronId, SynapseType, NeuralValue};

// After
use feagi_neural::types::{NeuronId, SynapseType, NeuralValue};

// Storage migration
// Before
use feagi_types::{NeuronArray, SynapseArray};

// After
use feagi_runtime_std::{StdNeuronArray, StdSynapseArray};
// OR
use feagi_runtime_embedded::{EmbeddedNeuronArray, EmbeddedSynapseArray};

// Synapse logic migration
// Before
use feagi_synapse::{compute_contribution, weight_scaling};

// After
use feagi_neural::synapse::{compute_contribution, weight_scaling};

// CorticalID migration
// Before
use feagi_types::CorticalAreaId;

// After
use feagi_data_structures::genomic::cortical_area::CorticalID;
```

---

### Phase 3: Make Burst Engine Generic Over Runtime (Week 3-4)

**Goal**: Refactor `RustNPU` to be generic over `Runtime` trait

**Tasks**:

#### 3.1 Update RustNPU Structure
```rust
// Before
pub struct RustNPU<T: NeuralValue> {
    neuron_array: RwLock<NeuronArray<T>>,  // ❌ Concrete
}

// After
pub struct RustNPU<R: Runtime, T: NeuralValue, M: NeuronModel> {
    neuron_array: RwLock<R::NeuronArray>,  // ✅ Generic over runtime
    runtime: Arc<R>,
    model: Arc<M>,  // ✅ Generic over neuron model
    model_params: Vec<M::Parameters>,  // Per-neuron parameters
}
```

#### 3.2 Update All Methods
- Change `&mut NeuronArray<T>` to `&mut dyn NeuronStorage`
- Change `&SynapseArray` to `&dyn SynapseStorage`
- Update all internal methods to use trait methods
- Update tests

#### 3.3 Update ComputeBackend Trait
- Change `process_synaptic_propagation` to accept `&dyn SynapseStorage`
- Change `process_neural_dynamics` to accept `&mut dyn NeuronStorage`
- Update all backend implementations (CPU, WGPU, CUDA)

**Files Modified**:
- `feagi-burst-engine/src/npu.rs` (major refactor)
- `feagi-burst-engine/src/backend/mod.rs` (update trait)
- `feagi-burst-engine/src/backend/cpu.rs` (update implementation)
- `feagi-burst-engine/src/backend/wgpu_backend.rs` (update implementation)
- `feagi-burst-engine/src/backend/cuda_backend.rs` (update implementation)
- All test files

**Breaking Changes**: Yes - `RustNPU` now requires `Runtime` and `NeuronModel` generic parameters

**Migration Guide**:
```rust
// Before
let npu = RustNPU::<f32>::new(/* ... */)?;

// After (with explicit model)
use feagi_neural::models::LeakyIntegrateAndFire;
let runtime = Arc::new(StdRuntime);
let model = LeakyIntegrateAndFire::default();
let npu = RustNPU::<StdRuntime, f32, LeakyIntegrateAndFire>::new_with_model(
    runtime,
    /* ... */,
    model,
)?;

// Or (with default LIF model)
let npu = RustNPU::<StdRuntime, f32>::new(runtime, /* ... */)?;
```

---

### Phase 4: Create CUDA Runtime (Week 5)

**Goal**: Add CUDA runtime for GPU VRAM storage

**Tasks**:
1. Create `feagi-runtime-cuda` crate
2. Implement `CudaNeuronArray` with `CudaSlice<T>`
3. Implement `CudaSynapseArray` with `CudaSlice<T>`
4. Implement `Runtime` trait for `CudaRuntime`
5. Add tests (unit tests with mocked CUDA, integration tests with real GPU)

**Files Created**:
- `feagi-runtime-cuda/Cargo.toml`
- `feagi-runtime-cuda/src/lib.rs`
- `feagi-runtime-cuda/src/runtime.rs`
- `feagi-runtime-cuda/src/neuron_array.rs`
- `feagi-runtime-cuda/src/synapse_array.rs`
- `feagi-runtime-cuda/README.md`

**Dependencies**: `cudarc` (already in workspace)

**Testing**: Requires NVIDIA GPU for integration tests

---

### Phase 5: Enhance Compute Backend for Multi-GPU (Week 6)

**Goal**: Support multiple GPUs on NVIDIA DGX

**Tasks**:
1. Update `CUDABackend` to handle multiple devices
2. Implement work partitioning (neurons/synapses per GPU)
3. Implement result merging (FCL aggregation)
4. Add multi-GPU configuration
5. Add benchmarks

**Files Modified**:
- `feagi-burst-engine/src/backend/cuda_backend.rs` (major enhancement)
- `feagi-burst-engine/src/backend/mod.rs` (add multi-GPU config)

**New Features**:
- Automatic GPU detection and selection
- Workload balancing across GPUs
- Inter-GPU communication (if needed)

---

### Phase 6: Update All Consumers (Week 7)

**Goal**: Update all projects that use feagi-core

**Tasks**:

#### 6.1 Update feagi-rust
- Change from `feagi-types::*` to `feagi-neural::types::*`
- Change from `feagi-types::NeuronArray` to `feagi-runtime-std::StdNeuronArray`
- Update `RustNPU` usage to include `StdRuntime`
- Run tests

#### 6.2 Update feagi-inference-engine
- Same changes as feagi-rust
- Run tests

#### 6.3 Update feagi-embedded
- Already uses `feagi-runtime-embedded` (no change needed)
- Verify integration still works

#### 6.4 Update feagi-bdu
- Update to use `Runtime` trait
- Run tests

**Files Modified**:
- All consumer projects (feagi-rust, feagi-inference-engine, etc.)

---

### Phase 7: Documentation and Examples (Week 8)

**Goal**: Comprehensive documentation and migration guides

**Tasks**:
1. Update all crate READMEs
2. Create migration guide for consumers
3. Add examples for each runtime
4. Update architecture documentation

**Files Created/Updated**:
- `feagi-runtime/README.md`
- `feagi-runtime-std/README.md`
- `feagi-runtime-embedded/README.md`
- `feagi-runtime-cuda/README.md`
- `feagi-core/docs/RUNTIME_ARCHITECTURE.md`
- `feagi-core/docs/MIGRATION_GUIDE.md`

---

## Migration Strategy

### Backward Compatibility

**Challenge**: Breaking changes to `feagi-types`, `feagi-synapse`, and `feagi-burst-engine`

**Solution**: Provide migration shims and clear migration path

#### Migration Shim (Temporary)

```rust
// feagi-neural/src/compat.rs (deprecated compatibility layer)

#[deprecated(note = "Use feagi-neural::types::NeuronId instead")]
pub use crate::types::NeuronId;

#[deprecated(note = "Use feagi-neural::synapse::compute_contribution instead")]
pub use crate::synapse::compute_contribution;

#[deprecated(note = "Use feagi-runtime-std::StdNeuronArray instead")]
pub type NeuronArray<T> = feagi_runtime_std::StdNeuronArray<T>;
```

**Timeline**: Keep shims for 2 major versions, then remove

### Phased Rollout

1. **Phase 1-2**: Internal changes (no external impact)
2. **Phase 3**: Breaking changes (require consumer updates)
3. **Phase 4-5**: New features (optional, no breaking changes)
4. **Phase 6**: Consumer migration (coordinated with teams)
5. **Phase 7**: Documentation (ongoing)

### Communication Plan

1. **Announcement**: Document breaking changes in CHANGELOG
2. **Migration Guide**: Step-by-step instructions for each consumer
3. **Support**: Provide assistance during migration
4. **Timeline**: 2-month migration window

---

## Testing Strategy

### Unit Tests

**Runtime Traits**:
- Test trait contracts (all methods callable)
- Test trait bounds (Send + Sync)
- Test error handling

**Runtime Implementations**:
- Test `StdRuntime` with various capacities
- Test `EmbeddedRuntime` with const generics
- Test `CudaRuntime` with mocked CUDA (unit) and real GPU (integration)

**Burst Engine**:
- Test `RustNPU` with each runtime
- Test compute backend with each runtime
- Test error propagation

### Integration Tests

**Cross-Runtime Tests**:
- Run same burst cycle on std vs embedded vs CUDA
- Verify identical results (within floating-point tolerance)
- Performance benchmarks

**Multi-GPU Tests**:
- Test work partitioning
- Test result merging
- Test load balancing

### Performance Tests

**Benchmarks**:
- Compare std vs embedded vs CUDA performance
- Compare single-GPU vs multi-GPU
- Memory usage profiling

---

## Future Expansion

### WASM Runtime

```rust
// feagi-runtime-wasm/src/runtime.rs

pub struct WasmRuntime;

impl Runtime for WasmRuntime {
    type NeuronArray = WasmNeuronArray;
    type SynapseArray = WasmSynapseArray;
    // ...
}

// feagi-runtime-wasm/src/neuron_array.rs
pub struct WasmNeuronArray<T: NeuralValue> {
    // Uses WebAssembly.Memory or wasm-bindgen
    pub membrane_potentials: js_sys::Float32Array,
    // ...
}
```

**Use Cases**: Browser-based neural networks, Node.js inference

### RTOS Runtime

```rust
// feagi-runtime-rtos/src/runtime.rs

pub struct FreeRTOSRuntime {
    heap: FreeRTOSHeap,
}

impl Runtime for FreeRTOSRuntime {
    // Uses FreeRTOS heap allocation
    // ...
}
```

**Use Cases**: Real-time embedded systems with RTOS

### Distributed Runtime

```rust
// feagi-runtime-distributed/src/runtime.rs

pub struct DistributedRuntime {
    nodes: Vec<Node>,
    partition_strategy: PartitionStrategy,
}

impl Runtime for DistributedRuntime {
    // Neurons/synapses distributed across nodes
    // MPI or gRPC for communication
    // ...
}
```

**Use Cases**: HPC clusters, cloud deployments

### Hybrid Runtime

```rust
// feagi-runtime-hybrid/src/runtime.rs

pub struct HybridRuntime {
    cpu_runtime: StdRuntime,
    gpu_runtime: CudaRuntime,
    split_strategy: SplitStrategy,
}

impl Runtime for HybridRuntime {
    // Some neurons on CPU, some on GPU
    // Automatic workload balancing
    // ...
}
```

**Use Cases**: Optimal performance on mixed CPU/GPU systems

---

## Example: WASM Runtime

### Question: "If I want to create a FEAGI WASM app, which neural processing crates would I use?"

**Answer**: For a WASM application, you'd use a subset of the neural processing crates optimized for browser execution.

### Current State (Before Refactor)

**Good news**: WASM supports `std`, so you can use existing crates today:

```toml
# Cargo.toml for WASM app
[dependencies]
feagi-neural = { path = "../../feagi-core/crates/feagi-neural" }  # All types + algorithms + models
feagi-burst-engine = { path = "../../feagi-core/crates/feagi-burst-engine", default-features = false }
wasm-bindgen = "0.2"
js-sys = "0.3"
```

```rust
// src/lib.rs
use feagi_burst_engine::RustNPU;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmBrain {
    npu: RustNPU<f32>,
}

#[wasm_bindgen]
impl WasmBrain {
    #[wasm_bindgen(constructor)]
    pub fn new(neurons: usize, synapses: usize) -> Self {
        let npu = RustNPU::new(neurons, synapses, /* ... */).unwrap();
        Self { npu }
    }
    
    pub fn process_burst(&mut self) -> usize {
        self.npu.process_burst().unwrap().neuron_count
    }
}
```

**Limitations**:
- Uses `Vec<T>` (heap allocation in WASM)
- No access to WebAssembly.Memory optimizations
- No SharedArrayBuffer support
- Must serialize data for JS access

### After Refactor (Optimized WASM Runtime)

With the refactor, you'd create a `feagi-runtime-wasm` crate:

```rust
// feagi-runtime-wasm/src/runtime.rs

pub struct WasmRuntime {
    memory: js_sys::WebAssembly::Memory,
}

impl Runtime for WasmRuntime {
    type NeuronArray = WasmNeuronArray;
    type SynapseArray = WasmSynapseArray;
    
    fn create_neuron_array<T: NeuralValue>(&self, capacity: usize) -> Self::NeuronArray {
        WasmNeuronArray::new(self.memory.clone(), capacity)
    }
    
    fn create_synapse_array(&self, capacity: usize) -> Self::SynapseArray {
        WasmSynapseArray::new(self.memory.clone(), capacity)
    }
    
    fn supports_parallel(&self) -> bool { true }  // SharedArrayBuffer
    fn supports_simd(&self) -> bool { true }      // WASM SIMD128
    fn memory_limit(&self) -> Option<usize> { Some(2 * 1024 * 1024 * 1024) }  // 2GB browser limit
}

// feagi-runtime-wasm/src/neuron_array.rs
pub struct WasmNeuronArray<T: NeuralValue> {
    // Direct WebAssembly.Memory access (zero-copy with JS)
    membrane_potentials: js_sys::Float32Array,
    thresholds: js_sys::Float32Array,
    leak_coefficients: js_sys::Float32Array,
    // ... other properties as typed arrays
}

impl<T: NeuralValue> NeuronStorage for WasmNeuronArray<T> {
    type Value = T;
    
    fn membrane_potentials(&self) -> &[T] {
        // Zero-copy view into WASM memory
        unsafe { 
            std::slice::from_raw_parts(
                self.membrane_potentials.buffer().data() as *const T,
                self.membrane_potentials.length() as usize
            )
        }
    }
    
    fn membrane_potentials_mut(&mut self) -> &mut [T] {
        // Zero-copy mutable view
        unsafe {
            std::slice::from_raw_parts_mut(
                self.membrane_potentials.buffer().data() as *mut T,
                self.membrane_potentials.length() as usize
            )
        }
    }
    
    // ... other trait methods
}
```

### Your WASM App (After Refactor)

```rust
// src/lib.rs
use feagi_runtime_wasm::WasmRuntime;
use feagi_neural::models::LeakyIntegrateAndFire;
use feagi_burst_engine::RustNPU;
use wasm_bindgen::prelude::*;
use std::sync::Arc;

#[wasm_bindgen]
pub struct WasmBrain {
    npu: RustNPU<WasmRuntime, f32, LeakyIntegrateAndFire>,
    runtime: Arc<WasmRuntime>,
}

#[wasm_bindgen]
impl WasmBrain {
    #[wasm_bindgen(constructor)]
    pub fn new(neurons: usize, synapses: usize) -> Result<WasmBrain, JsValue> {
        // Initialize panic hook for better error messages
        console_error_panic_hook::set_once();
        
        // Create WASM-specific runtime
        let runtime = Arc::new(WasmRuntime::new()
            .map_err(|e| JsValue::from_str(&e.to_string()))?);
        
        // Create neuron model (LIF)
        let model = LeakyIntegrateAndFire::default();
        
        // Create NPU with WASM runtime and LIF model
        let npu = RustNPU::new_with_model(
            runtime.clone(),
            neurons,
            synapses,
            model,
            BackendType::CPU,  // WASM = CPU only (no GPU in browser... yet)
        ).map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        Ok(WasmBrain { npu, runtime })
    }
    
    pub fn process_burst(&mut self) -> Result<usize, JsValue> {
        self.npu.process_burst()
            .map(|r| r.neuron_count)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    
    /// Zero-copy access to neuron membrane potentials from JavaScript
    pub fn get_membrane_potentials(&self) -> js_sys::Float32Array {
        let neurons = self.npu.neuron_array.read().unwrap();
        // Return typed array that JS can read directly (no copying)
        js_sys::Float32Array::from(neurons.membrane_potentials())
    }
    
    /// Zero-copy access to fired neurons mask
    pub fn get_fired_neurons(&self) -> js_sys::Uint8Array {
        let fire_queue = self.npu.get_current_fire_queue();
        // Convert to boolean mask for JS
        let mut mask = vec![0u8; self.npu.neuron_capacity()];
        for neuron_id in fire_queue.iter() {
            mask[neuron_id.0 as usize] = 1;
        }
        js_sys::Uint8Array::from(&mask[..])
    }
    
    /// Inject sensory input (zero-copy from JS typed array)
    pub fn inject_sensory(&mut self, neuron_ids: js_sys::Uint32Array, values: js_sys::Float32Array) {
        let ids: Vec<u32> = neuron_ids.to_vec();
        let vals: Vec<f32> = values.to_vec();
        
        for (id, val) in ids.iter().zip(vals.iter()) {
            self.npu.inject_sensory(NeuronId(*id), *val).unwrap();
        }
    }
}
```

### JavaScript Usage

```javascript
import init, { WasmBrain } from './pkg/feagi_wasm.js';

async function main() {
    // Initialize WASM module
    await init();
    
    // Create brain (10K neurons, 100K synapses)
    const brain = new WasmBrain(10000, 100000);
    
    // Inject sensory input (zero-copy)
    const sensorNeurons = new Uint32Array([0, 1, 2, 3, 4]);
    const sensorValues = new Float32Array([1.5, 1.2, 0.8, 1.0, 1.3]);
    brain.inject_sensory(sensorNeurons, sensorValues);
    
    // Process burst
    const fired = brain.process_burst();
    console.log(`Neurons fired: ${fired}`);
    
    // Zero-copy access to WASM memory
    const potentials = brain.get_membrane_potentials();
    console.log(`First neuron potential: ${potentials[0]}`);
    
    // Get fired neurons mask
    const firedMask = brain.get_fired_neurons();
    const firedNeurons = [];
    for (let i = 0; i < firedMask.length; i++) {
        if (firedMask[i] === 1) firedNeurons.push(i);
    }
    console.log(`Fired neurons: ${firedNeurons}`);
    
    // Visualization (WebGL, Three.js, etc.)
    visualizeBrain(potentials, firedMask);
}

main();
```

### Crates Used for WASM App

| Crate | Purpose | Required? |
|-------|---------|-----------|
| **feagi-neural** | ALL neural computation (types + algorithms + models) | ✅ Required |
| **feagi-runtime** | Runtime trait definitions | ✅ Required |
| **feagi-runtime-wasm** | WASM storage (typed arrays) | ✅ Required |
| **feagi-burst-engine** | Burst orchestration | ✅ Required |
| **feagi-runtime-std** | Desktop runtime | ❌ Skip |
| **feagi-runtime-embedded** | ESP32 runtime | ❌ Skip |
| **feagi-runtime-cuda** | GPU runtime | ❌ Skip |
| **feagi-bdu** | Runtime neuron creation | ⚠️  Optional |
| **feagi-plasticity** | Learning/STDP | ⚠️  Optional |
| **feagi-evo** | Genome editing | ❌ Skip |
| **feagi-io** | I/O layer | ❌ Skip (browser has own I/O) |

### Benefits of WASM Runtime vs Std

| Feature | std (Vec) | WASM Runtime |
|---------|-----------|--------------|
| **Memory** | WASM heap | WebAssembly.Memory (configurable, growable) |
| **JS Access** | Must serialize | Zero-copy typed arrays |
| **SharedArrayBuffer** | ❌ No | ✅ Multi-threaded WASM |
| **Memory Overhead** | Higher (Vec metadata) | Lower (raw arrays) |
| **Performance** | Good | Better (SIMD via WASM128) |
| **Memory Control** | Automatic (GC) | Manual (growable linear memory) |

### WASM Architecture Diagram

```
┌─────────────────────────────────────────────────┐
│ JavaScript / Browser                             │
│   - UI rendering (Canvas, WebGL)                │
│   - User input events                            │
│   - Sensory data injection                       │
│   - Visualization (Three.js, D3.js)             │
└───────────────────┬─────────────────────────────┘
                    │ wasm_bindgen (zero-copy)
┌───────────────────▼─────────────────────────────┐
│ Your WASM App (Rust → .wasm)                    │
│   - WasmBrain struct (exported to JS)           │
│   - JS bindings (zero-copy typed arrays)        │
└───────────────────┬─────────────────────────────┘
                    │ uses
┌───────────────────▼─────────────────────────────┐
│ feagi-burst-engine<WasmRuntime, f32, LIF>       │
│   - Generic burst processing                     │
│   - Works with any Runtime                       │
│   - Uses any NeuronModel                         │
└───────────────────┬─────────────────────────────┘
                    │ uses
┌───────────────────▼─────────────────────────────┐
│ feagi-runtime-wasm                               │
│   - WasmNeuronArray (Float32Array)              │
│   - WasmSynapseArray (Uint32Array)              │
│   - Zero-copy JS access                          │
│   - SharedArrayBuffer support                    │
└───────────────────┬─────────────────────────────┘
                    │ uses
┌───────────────────▼─────────────────────────────┐
│ feagi-neural                                     │
│   - LeakyIntegrateAndFire model                 │
│   - Platform-agnostic algorithms                 │
│   - no_std compatible                            │
└──────────────────────────────────────────────────┘
```

### WASM Runtime Implementation Timeline

**Phase 8** (Week 9-10): Implement WASM runtime after core refactor

**Tasks**:
1. Create `feagi-runtime-wasm` crate
2. Implement `WasmNeuronArray` with typed arrays
3. Implement `WasmSynapseArray` with typed arrays
4. Add zero-copy JS bindings
5. Add SharedArrayBuffer support (multi-threaded WASM)
6. Add WASM SIMD128 optimizations
7. Create example WASM app with visualization
8. Performance benchmarks vs std

**Dependencies**:
- `wasm-bindgen` - Rust/JS interop
- `js-sys` - JavaScript bindings
- `web-sys` - Web APIs (if needed)

**Testing**:
- Unit tests (node.js environment)
- Integration tests (headless browser via wasm-pack test)
- Performance benchmarks (vs std runtime)

---

## Risk Assessment

### High Risk

1. **Breaking Changes**: Consumers must update code
   - **Mitigation**: Migration shims, clear documentation, phased rollout

2. **Performance Regression**: Trait indirection might slow down code
   - **Mitigation**: Zero-cost abstractions (monomorphization), benchmarks

3. **Complexity**: More generic code is harder to understand
   - **Mitigation**: Comprehensive documentation, examples, code reviews

### Medium Risk

1. **CUDA Integration**: Complex GPU memory management
   - **Mitigation**: Use `cudarc` crate (mature), extensive testing

2. **Multi-GPU Coordination**: Work partitioning and result merging
   - **Mitigation**: Start with simple round-robin, iterate based on benchmarks

3. **Embedded Compatibility**: Ensure no_std works correctly
   - **Mitigation**: Continuous testing on ESP32 hardware

### Low Risk

1. **Documentation**: Keeping docs up to date
   - **Mitigation**: Documentation as part of PR process

2. **Test Coverage**: Ensuring all code paths tested
   - **Mitigation**: Code coverage tools, integration tests

---

## Success Criteria

### Functional Requirements

- ✅ `feagi-burst-engine` runs on ESP32 (embedded runtime)
- ✅ `feagi-burst-engine` runs on desktop (std runtime)
- ✅ `feagi-burst-engine` runs on NVIDIA DGX (CUDA runtime, multi-GPU)
- ✅ All existing tests pass
- ✅ Performance within 5% of current implementation (no regression)

### Non-Functional Requirements

- ✅ Zero code duplication (storage implementations only in runtime crates)
- ✅ Clear separation of concerns (types vs storage vs compute)
- ✅ Comprehensive documentation
- ✅ Migration path for all consumers

### Future Readiness

- ✅ Easy to add new runtimes (WASM, RTOS, distributed)
- ✅ Easy to add new compute backends
- ✅ Architecture supports 10+ platforms

---

## Timeline

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| Phase 1: Runtime Trait | 1 week | None |
| Phase 2: Move Storage | 1 week | Phase 1 |
| Phase 3: Generic Burst Engine | 2 weeks | Phase 2 |
| Phase 4: CUDA Runtime | 1 week | Phase 3 |
| Phase 5: Multi-GPU Backend | 1 week | Phase 4 |
| Phase 6: Update Consumers | 1 week | Phase 3 |
| Phase 7: Documentation | 1 week | All phases |

**Total**: 8 weeks

**Critical Path**: Phase 1 → Phase 2 → Phase 3 → Phase 6

---

## Conclusion

This refactor plan provides a clear path to proper hardware abstraction for FEAGI's neural processing crates. The proposed architecture:

1. **Separates concerns**: Types, storage, algorithms, orchestration
2. **Enables multi-platform**: ESP32, desktop, HPC (DGX), future platforms
3. **Maintains performance**: Zero-cost abstractions, no runtime overhead
4. **Reduces duplication**: Single source of truth per runtime
5. **Future-proof**: Easy to extend with new runtimes and backends

The migration is significant but manageable with a phased approach and clear communication. The end result is a more maintainable, extensible, and powerful architecture that supports FEAGI's long-term vision.

---

**Next Steps**:
1. Review and approve this plan
2. Create GitHub issues for each phase
3. Assign team members to phases
4. Begin Phase 1 implementation

---

**Copyright 2025 Neuraville Inc.**  
**Licensed under Apache License 2.0**

