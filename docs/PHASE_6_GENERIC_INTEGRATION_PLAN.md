# Phase 6: Full Generic Integration Plan

**Status**: 🔵 IN PROGRESS  
**Start Date**: November 4, 2025  
**Goal**: Make entire NPU storage system generic over `T: NeuralValue` for INT8/FP32/FP16 support

---

## Architecture Overview

```
Genome (quantization_precision: "int8")
  ↓
Neuroembryogenesis (parses, dispatches)
  ↓
ConnectomeManager<T: NeuralValue>
  ↓
RustNPU<T: NeuralValue>
  ├── NeuronArray<T> (feagi-types)
  ├── SynapseArray (u8 weights - no change needed)
  └── Processing (uses feagi-neural<T>, feagi-synapse)
```

---

## Step-by-Step Integration

### Step 1: Make feagi-types::NeuronArray Generic ⚡ STARTING

**File**: `feagi-types/src/npu.rs`

**Changes**:
```rust
// BEFORE
pub struct NeuronArray {
    pub membrane_potentials: Vec<f32>,
    pub thresholds: Vec<f32>,
    ...
}

// AFTER
pub struct NeuronArray<T: NeuralValue> {
    pub membrane_potentials: Vec<T>,
    pub thresholds: Vec<T>,
    pub leak_coefficients: Vec<f32>,  // Stay f32
    pub resting_potentials: Vec<T>,
    ...
}
```

**Impact**: ~300 lines to update

---

### Step 2: Make feagi-types::SynapseArray Review

**File**: `feagi-types/src/npu.rs`

**Decision**: SynapseArray uses `u8` for weights (0-255). This is already quantized!

**Conclusion**: NO CHANGE NEEDED - weights are already 8-bit.

---

### Step 3: Make RustNPU Generic

**File**: `feagi-burst-engine/src/npu.rs`

**Changes**:
```rust
// BEFORE
pub struct RustNPU {
    pub(crate) neuron_array: RwLock<NeuronArray>,  // f32
    ...
}

// AFTER
pub struct RustNPU<T: NeuralValue> {
    pub(crate) neuron_array: RwLock<NeuronArray<T>>,
    ...
}
```

**Impact**: ~2,600 lines to update

---

### Step 4: Update Neural Dynamics Processing

**File**: `feagi-burst-engine/src/neural_dynamics.rs`

**Status**: ✅ Already uses generic `feagi-neural` functions!

**Verification needed**: Ensure compatibility with generic `NeuronArray<T>`

---

### Step 5: Update Synaptic Propagation

**File**: `feagi-burst-engine/src/synaptic_propagation.rs`

**Status**: ✅ Already uses generic `feagi-synapse` functions!

**Verification needed**: Ensure compatibility with generic types

---

### Step 6: Make ConnectomeManager Generic

**File**: `feagi-bdu/src/connectome_manager.rs`

**Changes**:
```rust
// BEFORE
pub struct ConnectomeManager {
    npu: Option<Arc<Mutex<RustNPU>>>,  // f32
    ...
}

// AFTER
pub struct ConnectomeManager<T: NeuralValue> {
    npu: Option<Arc<Mutex<RustNPU<T>>>>,
    ...
}
```

**Impact**: ~3,085 lines to update

---

### Step 7: Wire Up Type Dispatch in Neuroembryogenesis

**File**: `feagi-bdu/src/neuroembryogenesis.rs`

**Changes**:
```rust
match quant_spec.precision {
    Precision::FP32 => {
        self.develop_with_type::<f32>(genome, &quant_spec)?
    }
    Precision::INT8 => {
        self.develop_with_type::<INT8Value>(genome, &quant_spec)?
    }
    ...
}

fn develop_with_type<T: NeuralValue>(
    &mut self,
    genome: &RuntimeGenome,
    quant_spec: &QuantizationSpec
) -> BduResult<()> {
    // Type-specific connectome construction
}
```

---

## Type Aliases for Public API

After generic conversion, add type aliases:

```rust
// feagi-types/src/npu.rs
pub type NeuronArrayF32 = NeuronArray<f32>;
pub type NeuronArrayINT8 = NeuronArray<INT8Value>;
pub type NeuronArrayF16 = NeuronArray<f16>;  // Future

// feagi-burst-engine/src/npu.rs
pub type RustNPUF32 = RustNPU<f32>;
pub type RustNPUINT8 = RustNPU<INT8Value>;
pub type RustNPUF16 = RustNPU<f16>;  // Future

// feagi-bdu/src/connectome_manager.rs
pub type ConnectomeManagerF32 = ConnectomeManager<f32>;
pub type ConnectomeManagerINT8 = ConnectomeManager<INT8Value>;
```

---

## Testing Strategy

1. **Incremental Compilation**: Test after each step
2. **FP32 Regression Tests**: Ensure existing behavior unchanged
3. **INT8 Smoke Tests**: Basic functionality
4. **End-to-End Tests**: Genome → Connectome → Burst

---

## Estimated Timeline

- **Step 1**: 2-3 hours (NeuronArray generic)
- **Step 2**: Review only (no changes)
- **Step 3**: 4-6 hours (RustNPU generic)
- **Step 4-5**: Verification (1 hour)
- **Step 6**: 6-8 hours (ConnectomeManager generic)
- **Step 7**: 2-3 hours (Type dispatch)
- **Testing**: 2-3 hours

**Total**: 1-2 days

---

**Starting with Step 1 now...**
