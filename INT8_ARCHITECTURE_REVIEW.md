# INT8 Quantization Support: Architecture Review

**Date**: 2025-12-25  
**Scope**: FEAGI INT8 hardware support for Hailo-8 and similar accelerators  
**Status**: 🟡 PARTIALLY IMPLEMENTED - Foundation exists but NOT production-ready

---

## Executive Summary

**What You Built (Good News ✅):**
1. ✅ Generic `NeuralValue` trait with `INT8Value` implementation
2. ✅ Quantization-aware membrane potential storage
3. ✅ Generic `RustNPU<R, T, B>` where `T: NeuralValue` (can be `f32` or `INT8Value`)
4. ✅ Hailo-8 HAL abstraction layer  
5. ✅ Fixed-point leak coefficient (`INT8LeakCoefficient`)

**What's Missing (Bad News ❌):**
1. ❌ **Synaptic contribution calculation ALWAYS uses f32** (line 54-67, `contribution.rs`)
2. ❌ **No INT8 path for weight×conductance multiplication**
3. ❌ **PSP division uses u8 integer division** (precision loss!)
4. ❌ **No scale factors or quantization parameters**
5. ❌ **Hailo backend is a stub** (doesn't actually call HailoRT)

---

## Detailed Analysis

### 1. Generic Architecture (✅ Correct Design)

```rust:107:111:feagi-core/crates/feagi-npu/burst-engine/src/npu.rs
pub struct RustNPU<
    R: Runtime,
    T: NeuralValue,  // ← Can be f32 OR INT8Value!
    B: crate::backend::ComputeBackend<T, R::NeuronStorage<T>, R::SynapseStorage>,
> {
```

**This is excellent!** You can instantiate:
- `RustNPU<StdRuntime, f32, CPUBackend>` for desktop
- `RustNPU<EmbeddedRuntime, INT8Value, HailoBackend>` for embedded

### 2. INT8Value Implementation (✅ Well-Designed)

```rust:128:179:feagi-core/crates/feagi-npu/neural/src/types/numeric.rs
pub struct INT8Value(pub i8);

impl INT8Value {
    pub const MEMBRANE_MIN: f32 = -100.0;
    pub const MEMBRANE_MAX: f32 = 50.0;
    pub const MEMBRANE_RANGE: f32 = Self::MEMBRANE_MAX - Self::MEMBRANE_MIN;
    pub const SCALE: f32 = 254.0;
    pub const RESOLUTION: f32 = Self::MEMBRANE_RANGE / Self::SCALE;
    // ...
}

impl NeuralValue for INT8Value {
    fn from_f32(value: f32) -> Self { /* quantize */ }
    fn to_f32(self) -> f32 { /* dequantize */ }
    fn saturating_add(self, other: Self) -> Self { /* INT8 */ }
    fn mul_leak(self, leak_coefficient: f32) -> Self { /* converts to f32! */ }
    // ...
}
```

**Analysis:**
- ✅ Quantization range: [-100, 50] → [-127, 127] (i8)
- ✅ Resolution: 0.59 per i8 unit (150/254)
- ⚠️ `mul_leak()` converts to f32 internally! (Line 175-178)
- ⚠️ Only works on **CPU with floats**, NOT on pure INT8 hardware!

### 3. **CRITICAL ISSUE: Synaptic Contribution Always Uses f32**

```rust:54:67:feagi-core/crates/feagi-npu/neural/src/synapse/contribution.rs
pub fn compute_synaptic_contribution(
    weight: u8,
    conductance: u8,
    synapse_type: SynapseType,
) -> f32 {  // ← ALWAYS RETURNS f32!
    let w = weight as f32;      // ← Converts to float
    let c = conductance as f32; // ← Converts to float
    let sign = match synapse_type {
        SynapseType::Excitatory => 1.0,
        SynapseType::Inhibitory => -1.0,
    };
    w * c * sign  // ← Float multiplication
}
```

**This function CANNOT run on Hailo/TPU accelerators!**

Called from:

```rust:226:feagi-core/crates/feagi-npu/burst-engine/src/synaptic_propagation.rs
SynapticContribution(compute_synaptic_contribution(weight, psp, synapse_type));
```

**Problem**: Even if `T=INT8Value` in `RustNPU`, the synaptic contribution is ALWAYS computed as `f32`.

### 4. PSP Division Bug (🚨 CRITICAL for Your Issue)

```rust:208:216:feagi-core/crates/feagi-npu/burst-engine/src/synaptic_propagation.rs
// PSP uniformity = false: PSP is divided among all outgoing synapses
let synapse_count = source_synapse_counts.get(&source_neuron).copied().unwrap_or(1);
if synapse_count > 1 {
    // Divide PSP by number of outgoing synapses
    base_psp / synapse_count as u8  // ← u8 INTEGER DIVISION!
} else {
    base_psp
}
```

**This is YOUR BUG!**
- `base_psp` is `u8` (0-255)
- Division: `1u8 / 10u8 = 0u8` (precision loss!)
- Then: `contribution = weight × 0 = 0.0f32`

**Why it might still fire**: If your genome has `conductance >> 1`, the division doesn't truncate to 0.

---

## What's Actually Supported

### Current Support Matrix

| Component | f32 (CPU) | INT8Value (CPU) | INT8 (Hailo) |
|-----------|-----------|-----------------|--------------|
| Membrane potential storage | ✅ | ✅ | ❌ |
| Threshold comparison | ✅ | ✅ | ❌ |
| Leak application | ✅ | ⚠️ (converts to f32) | ❌ |
| Synapse weight×conductance | ✅ | ❌ (uses f32) | ❌ |
| PSP division | 🐛 (float) | 🐛 (u8 int) | ❌ |
| FCL accumulation | ✅ | ✅ | ❌ |
| Hailo backend | N/A | N/A | 🟡 (stub only) |

**Legend:**
- ✅ Fully working
- ⚠️ Works but suboptimal (uses floats internally)
- 🐛 Bug exists
- ❌ Not implemented
- 🟡 Placeholder/stub

---

## What You're Missing for True INT8 Hardware Support

### 1. **INT8 Synaptic Contribution**

Need to add:

```rust
// Option A: Generic contribution calculation
pub fn compute_synaptic_contribution_generic<T: NeuralValue>(
    weight: u8,
    conductance: u8,
    synapse_type: SynapseType,
) -> T {
    // For f32: convert and multiply
    // For INT8Value: use fixed-point math or lookup table
}

// Option B: Backend-specific implementation
trait ComputeBackend {
    type Contribution;
    fn compute_contribution(&self, weight: u8, conductance: u8, sign: i8) -> Self::Contribution;
}
```

### 2. **Fixed-Point PSP Division**

```rust
// Instead of: base_psp / synapse_count  (loses precision)
// Use scaled division:
let scale = 256u16;
let scaled_psp = (base_psp as u16) * scale;  // 1 * 256 = 256
let divided_psp = scaled_psp / (synapse_count as u16);  // 256 / 10 = 25
// Keep in u16 for accumulation, OR divide by scale to get u8
```

### 3. **Quantization Scale Factors**

```rust
pub struct QuantizationConfig {
    pub weight_scale: f32,       // For host-side quantization
    pub activation_scale: f32,   // For host-side quantization
    pub output_scale: f32,       // For dequantization
    pub zero_point: i8,          // Offset for asymmetric quantization
}
```

### 4. **Hailo Backend Implementation**

```rust
impl ComputeBackend for HailoBackend {
    fn process_burst(&mut self, ...) -> Result<...> {
        // 1. Upload quantized data to Hailo
        hailo.upload_neurons(...)?;
        hailo.upload_synapses(...)?;
        
        // 2. Run inference (INT8 ops on Hailo cores)
        hailo.process_burst()?;
        
        // 3. Download results
        hailo.download_neurons(...)?;
        
        Ok(...)
    }
}
```

### 5. **INT8-Only Leak Calculation**

Currently `mul_leak()` converts to f32 (line 175-178). Need:

```rust
impl NeuralValue for INT8Value {
    fn mul_leak(self, leak_coefficient: f32) -> Self {
        // Option A: Fixed-point multiplication (no floats!)
        let leak_i16 = INT8LeakCoefficient::from_f32(leak_coefficient);
        let potential_i32 = (self.0 as i32) * (leak_i16.0 as i32);
        let result = (potential_i32 / INT8LeakCoefficient::SCALE) as i8;
        Self(result.max(-127))
        
        // Option B: Lookup table (fastest, but uses memory)
        // LEAK_TABLE[self.0 as usize][leak_index]
    }
}
```

---

## Answer to "What Am I Missing?"

**You have the FOUNDATION, but not the FULL IMPLEMENTATION:**

1. **Architecture** (generic `RustNPU<R,T,B>`) ✅  
2. **INT8Value type** ✅  
3. **Quantization** ✅  

BUT:

4. **Synaptic contribution** still uses f32 ❌  
5. **PSP division** has precision loss 🐛  
6. **Hailo backend** is a stub ❌  
7. **No scale factors** ❌  
8. **Leak uses f32 internally** ⚠️  

**Bottom line**: Your code will run on a CPU with `T=INT8Value`, but:
- It still uses **floats for multiplication** (synaptic contrib, leak)
- **PSP division bug** (your current issue!)
- Won't run on **pure INT8 hardware** (Hailo, TPU) without major refactoring

---

## Recommendations

### Immediate (Fix Your Bug):
1. **Fix PSP division** to use scaled arithmetic or float before division
2. Check your genome: `conductance` value is likely >> 1

### Short-term (INT8-ready):
1. Make `compute_synaptic_contribution` generic over `T: NeuralValue`
2. Add fixed-point PSP division with scale factors
3. Implement INT8-only leak (no f32 conversion)

### Long-term (Hailo support):
1. Implement Hailo FFI bindings
2. Add quantization-aware training pipeline
3. Create backend-specific test suite
4. Add scale factor management

Want me to fix the PSP division bug first, then we can discuss the path to true INT8 hardware support?

