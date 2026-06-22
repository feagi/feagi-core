# Multi-Model Neuron Architecture

**Status**: Phase 0.5 - Folder Structure Established  
**Version**: 2.1  
**Date**: 2025-10-27  
**Authors**: FEAGI Architecture Team  
**Reviewers**: Core Team  
**Implementation Status**: ✅ Structure only, single model (LIF)

---

## Executive Summary

FEAGI is evolving from a single-neuron-model system to support **multiple neuron models** (LIF, Izhikevich, AdEx, Hodgkin-Huxley) within the same brain, with different cortical areas using different models. This document defines the architecture to achieve this while maintaining:

- ✅ High performance (GPU/CPU/RTOS compatibility)
- ✅ Memory efficiency (zero waste for unused model parameters)
- ✅ Deterministic behavior (same results across all backends)
- ✅ Backward compatibility (existing memory neuron pattern)
- ✅ Type safety (compile-time guarantees)
- ✅ Dynamic brain modification (area creation/deletion at runtime)

**CURRENT STATUS**: Folder structure established with LIF-only placeholders. No multi-model complexity added to runtime code.

**CRITICAL**: This is a major architectural change. See [Section 13: Concerns & Risks](#13-concerns--risks) before implementing full multi-model support.

---

## Table of Contents

1. [Core Principles](#1-core-principles)
2. [ID System: Roaring Bitmaps + Dynamic Ranges](#2-id-system-roaring-bitmaps--dynamic-ranges)
3. [Model-Specific Arrays](#3-model-specific-arrays)
4. [Neuron Router & Lookup Strategy](#4-neuron-router--lookup-strategy)
5. [Model-Aware Fire Candidate List](#5-model-aware-fire-candidate-list)
6. [Synaptic Integration](#6-synaptic-integration)
7. [GPU Backend Strategy](#7-gpu-backend-strategy)
8. [LIF Model Specification](#8-lif-model-specification)
9. [Implementation Phases](#9-implementation-phases)
10. [Migration Path](#10-migration-path)
11. [Folder Structure](#11-folder-structure)
12. [Performance Analysis](#12-performance-analysis)
13. [Concerns & Risks](#13-concerns--risks)
14. [Decision Framework](#14-decision-framework)

---

## 1. Core Principles

### 1.1 Design Philosophy

**Area-Segregated Model Assignment**
- All neurons in a cortical area use the **same neuron model**
- Different cortical areas can use different models
- Biological reasoning: Cortical columns are functionally homogeneous
- Cross-model synapses: **fully supported** (no restrictions on connectivity)

**Model-Specific Arrays**
- Each neuron model has its own dedicated `NeuronArray`
- Zero memory waste: only store parameters that model actually uses
- Extends existing pattern: memory neurons already work this way
- Enables GPU kernel specialization (zero branch divergence)

**Dynamic Brain Support**
- Full runtime modification: cortical areas created/deleted dynamically
- ID recycling: freed neuron IDs reused via Roaring Bitmaps
- Billion-scale: support up to 4 billion neurons (full 32-bit range)
- Zero fragmentation: efficient ID management with free lists

**Performance-First**
- GPU batching by model type (no branch divergence)
- Cache-friendly: compact, model-specific memory layouts
- Hot path optimization: zero lookups in neural dynamics
- Model-aware FCL: group neurons by type during synaptic propagation

### 1.2 Why This Architecture?

#### Problem: Unified Array Wastes Memory
```rust
// ❌ BAD: Unified array with all model parameters
pub struct NeuronArray {
    // LIF parameters (used by some neurons)
    pub leak_coefficients: Vec<f32>,      // 4MB for 1M neurons
    
    // Izhikevich parameters (WASTED if neuron is LIF!)
    pub izh_a: Vec<f32>,                  // 4MB WASTED
    pub izh_b: Vec<f32>,                  // 4MB WASTED
    pub izh_u: Vec<f32>,                  // 4MB WASTED
    // 12MB wasted per unused model!
}
```

#### Solution: Model-Specific Arrays
```rust
// ✅ GOOD: Separate arrays, zero waste
pub struct LIFNeuronArray {
    pub leak_coefficients: Vec<f32>,      // Only what LIF needs
    pub resting_potentials: Vec<f32>,
    // 8MB for 1M LIF neurons
}

pub struct IzhikevichNeuronArray {
    pub a: Vec<f32>,                      // Only what Izhikevich needs
    pub b: Vec<f32>,
    pub u: Vec<f32>,
    // 12MB for 1M Izhikevich neurons
}
```

**Memory Savings**: With 3 models, 1M neurons each:
- Unified: 3M neurons × 60 bytes = **180MB**
- Separate: 1M × 20 bytes × 3 = **60MB** (3x reduction)

---

## 2. ID System: Roaring Bitmaps + Dynamic Ranges

### 2.1 Problem with Fixed Ranges

**Initial proposal used fixed ID ranges, but FEAGI requires:**
- ✅ Dynamic cortical area creation/deletion at runtime
- ✅ Support for 1 billion neurons
- ✅ Efficient ID recycling (no fragmentation)
- ✅ No arbitrary limits per model type

**Fixed ranges fail because:**
- ❌ What if one model needs 25M neurons but range only holds 20M?
- ❌ Wasted space: 10M AdEx range with only 1K neurons used
- ❌ No ID recycling: deleted neurons waste ID space
- ❌ Model migration: changing area model type requires synapse rewiring

### 2.2 Solution: Hybrid Architecture

**Combine dynamic ranges with Roaring Bitmaps for memory efficiency:**

```rust
use roaring::RoaringBitmap;

pub struct NeuronIdManager {
    /// Model type ranges (dynamically allocated, grows as needed)
    /// Example: LIF → Range { start: 10_000_000, end: 35_000_000 }
    model_ranges: HashMap<NeuronArrayType, IdRange>,
    
    /// Free IDs within each range (Roaring Bitmap = 180× compression!)
    /// Only stores FREED IDs, not all allocated ones
    free_ids: HashMap<NeuronArrayType, RoaringBitmap>,
    
    /// Statistics
    total_allocated: usize,
    total_freed: usize,
    
    /// Next available range start for new model types
    next_range_start: u32,
}

#[derive(Debug, Clone)]
struct IdRange {
    start: u32,           // Range start (e.g., 10M)
    end: u32,             // Current range end (grows dynamically)
    capacity: u32,        // Maximum end (1B per model)
    next_id: u32,         // Next ID to allocate within range
}
```

### 2.3 Memory Efficiency: Roaring Bitmaps

**Why Roaring Bitmaps?**

Roaring Bitmaps compress sparse integer sets by 10-100×:

| Approach | 1M Neurons | 100M Neurons | 1B Neurons |
|----------|-----------|--------------|------------|
| **HashMap** (old) | 36 MB | 3.6 GB | **36 GB** ❌ |
| **Roaring Bitmap** | <1 MB | 20 MB | **200 MB** ✅ |
| **Savings** | 36× | 180× | **180×** |

**How it works:**
- Only stores **freed** IDs (not all allocated IDs)
- Automatically compresses contiguous ranges
- Battle-tested (Apache Lucene, Spark, PostgreSQL)

### 2.4 Dynamic Range Allocation

```rust
impl NeuronIdManager {
    /// Allocate neuron ID for a model type
    pub fn allocate_neuron_id(
        &mut self,
        model_type: NeuronArrayType,
        local_index: u32,
    ) -> Option<u32> {
        // Get or create range for this model
        let range = self.model_ranges
            .entry(model_type)
            .or_insert_with(|| self.create_new_range(model_type));
        
        // Try to reuse a freed ID first (locality!)
        if let Some(free_bitmap) = self.free_ids.get_mut(&model_type) {
            if !free_bitmap.is_empty() {
                let recycled_id = free_bitmap.min().unwrap();
                free_bitmap.remove(recycled_id);
                return Some(recycled_id);
            }
        }
        
        // No freed IDs, allocate new from range
        if range.next_id >= range.end {
            if !self.expand_range(model_type) {
                return None;  // Range exhausted
            }
        }
        
        let id = range.next_id;
        range.next_id += 1;
        Some(id)
    }
    
    /// Deallocate neuron ID (add to roaring bitmap for reuse)
    pub fn deallocate_neuron_id(&mut self, global_id: u32) -> bool {
        if let Some(model_type) = self.get_model_type_from_id(global_id) {
            self.free_ids
                .entry(model_type)
                .or_insert_with(RoaringBitmap::new)
                .insert(global_id);
            true
        } else {
            false
        }
    }
    
    /// Batch deallocate cortical area (efficient bulk operation)
    pub fn deallocate_cortical_area(&mut self, neuron_ids: &[u32]) {
        for &id in neuron_ids {
            self.deallocate_neuron_id(id);
        }
    }
}
```

### 2.5 ID Lookup: O(m) Range Check

**Fast type detection without hash table:**

```rust
impl NeuronIdManager {
    /// Get model type from global ID (O(m) where m = # model types ≈ 10)
    #[inline]
    pub fn get_model_type_from_id(&self, global_id: u32) -> Option<NeuronArrayType> {
        for (model_type, range) in &self.model_ranges {
            if global_id >= range.start && global_id < range.end {
                return Some(*model_type);
            }
        }
        None
    }
    
    /// Convert global ID → local index
    #[inline]
    pub fn to_local_index(&self, global_id: u32) -> Option<(NeuronArrayType, u32)> {
        for (model_type, range) in &self.model_ranges {
            if global_id >= range.start && global_id < range.end {
                let local_index = global_id - range.start;
                return Some((*model_type, local_index));
            }
        }
        None
    }
}
```

**Performance**: 2-3 ns per model check × 10 models = **~20 ns per lookup**

### 2.6 Backward Compatibility

**Memory neurons remain unchanged:**

```rust
// Existing memory neuron implementation (50M-100M range)
// No changes needed - continues to work as-is
pub const MEMORY_NEURON_ID_START: u32 = 50_000_000;
pub const MEMORY_NEURON_ID_MAX: u32 = 99_999_999;
```

### 2.7 Billion-Scale Support

**Full 32-bit range available:**
- Each model can grow to 1B neurons (dynamic expansion)
- Total capacity: 4,294,967,296 IDs
- No arbitrary limits
- Zero fragmentation via Roaring Bitmaps

---

## 3. Model-Specific Arrays

### 3.1 LIF Neuron Array

```rust
/// Leaky Integrate-and-Fire Model
pub struct LIFNeuronArray {
    pub capacity: usize,
    pub count: usize,
    
    // Common properties
    pub membrane_potentials: Vec<f32>,
    pub thresholds: Vec<f32>,
    pub refractory_periods: Vec<u16>,
    pub refractory_countdowns: Vec<u16>,
    pub excitabilities: Vec<f32>,
    pub cortical_areas: Vec<u32>,
    pub coordinates: Vec<u32>,  // Flat [x,y,z,x,y,z,...]
    pub valid_mask: Vec<bool>,
    
    // LIF-specific parameters
    pub leak_coefficients: Vec<f32>,
    pub resting_potentials: Vec<f32>,
    pub mp_charge_accumulation: Vec<bool>,
    pub consecutive_fire_limits: Vec<u16>,
    pub snooze_periods: Vec<u16>,
    
    // Mapping
    pub global_neuron_ids: Vec<u32>,  // local_idx -> global_id
}
```

**Memory Layout**: 48 bytes per neuron (12 fields)

### 3.2 Izhikevich Neuron Array

```rust
/// Izhikevich Spiking Model
pub struct IzhikevichNeuronArray {
    pub capacity: usize,
    pub count: usize,
    
    // Common properties
    pub membrane_potentials: Vec<f32>,  // 'v' in Izhikevich
    pub thresholds: Vec<f32>,
    pub refractory_periods: Vec<u16>,
    pub refractory_countdowns: Vec<u16>,
    pub cortical_areas: Vec<u32>,
    pub coordinates: Vec<u32>,
    pub valid_mask: Vec<bool>,
    
    // Izhikevich-specific parameters
    pub a: Vec<f32>,                    // Recovery time constant
    pub b: Vec<f32>,                    // Sensitivity to u
    pub c: Vec<f32>,                    // Reset value
    pub d: Vec<f32>,                    // Reset of recovery variable
    pub u: Vec<f32>,                    // Recovery variable (state)
    
    // Mapping
    pub global_neuron_ids: Vec<u32>,
}
```

**Memory Layout**: 52 bytes per neuron (13 fields)

### 3.3 Adaptive Exponential (AdEx) Array

```rust
/// Adaptive Exponential Integrate-and-Fire Model
pub struct AdExNeuronArray {
    pub capacity: usize,
    pub count: usize,
    
    // Common properties
    pub membrane_potentials: Vec<f32>,
    pub thresholds: Vec<f32>,
    // ... common fields ...
    
    // AdEx-specific parameters
    pub adaptation_currents: Vec<f32>,  // w
    pub delta_t: Vec<f32>,              // Slope factor
    pub tau_w: Vec<f32>,                // Adaptation time constant
    pub a: Vec<f32>,                    // Subthreshold adaptation
    pub b: Vec<f32>,                    // Spike-triggered adaptation
    pub v_reset: Vec<f32>,              // Reset potential
    
    pub global_neuron_ids: Vec<u32>,
}
```

**Memory Layout**: 56 bytes per neuron (14 fields)

### 3.4 Memory Neuron Array

**Already Implemented** (no changes needed)

```rust
/// Memory neurons with lifecycle management
pub struct MemoryNeuronArray {
    // ... existing implementation ...
    pub lifespan_current: Vec<u32>,
    pub lifespan_initial: Vec<u32>,
    pub is_longterm_memory: Vec<bool>,
    // ... pattern matching, aging, etc.
}
```

---

## 4. Neuron Router & Lookup Strategy

### 4.1 The Lookup Problem

**Critical Challenge**: With model-specific arrays, we need efficient routing of global IDs.

**Hot Paths Where Lookups Occur:**
1. **Synaptic Propagation** (per target neuron): Moderate frequency
2. **Neural Dynamics** (per FCL entry): Would be catastrophic if not optimized
3. **Visualization** (periodic): Low frequency

### 4.2 Strategy: Optimize Where It Matters

**Key Insight**: Different hot paths need different optimizations.

#### Option A: Flat Lookup Table (Fastest)

```rust
/// Pre-built dense array for O(1) lookups
pub struct ModelLookupTable {
    /// Dense array: global_id → model_type
    /// Memory: 1 byte × 4B IDs = 4 GB
    model_types: Vec<NeuronArrayType>,
}

impl ModelLookupTable {
    #[inline]
    pub fn get_model_type(&self, global_id: u32) -> NeuronArrayType {
        self.model_types[global_id as usize]  // 2 ns, no branching
    }
}
```

**Pros:**
- ✅ Fastest possible (2 ns)
- ✅ Zero branching (CPU-friendly)
- ✅ Perfect for hot paths

**Cons:**
- ❌ 4 GB memory overhead
- ❌ Must be rebuilt when areas change

**Verdict**: Use for billion-neuron brains where performance critical

#### Option B: Range Check (Memory-Efficient)

```rust
impl NeuronIdManager {
    #[inline]
    pub fn get_model_type_from_id(&self, global_id: u32) -> Option<NeuronArrayType> {
        // O(m) where m = # model types (typically <10)
        for (model_type, range) in &self.model_ranges {
            if global_id >= range.start && global_id < range.end {
                return Some(*model_type);
            }
        }
        None
    }
}
```

**Pros:**
- ✅ Zero memory overhead
- ✅ Works with dynamic ranges
- ✅ Simple implementation

**Cons:**
- ❌ 20 ns per lookup (10× slower)
- ❌ Not cache-friendly

**Verdict**: Use for small-medium brains (<100M neurons)

### 4.3 Hybrid Approach

**Best of both worlds:**

```rust
pub struct NeuronRouter {
    /// Fast path: Optional flat lookup table
    flat_lookup: Option<Vec<NeuronArrayType>>,
    
    /// Fallback: Dynamic range manager
    id_manager: Arc<NeuronIdManager>,
    
    /// Threshold for switching to flat lookup
    neuron_count_threshold: usize,  // Default: 100M
}

impl NeuronRouter {
    #[inline]
    pub fn get_model_type(&self, global_id: u32) -> NeuronArrayType {
        if let Some(lookup_table) = &self.flat_lookup {
            // Fast path: O(1) array access
            lookup_table[global_id as usize]
        } else {
            // Fallback: O(m) range check
            self.id_manager.get_model_type_from_id(global_id)
                .unwrap_or(NeuronArrayType::Invalid)
        }
    }
    
    /// Build flat lookup table when brain grows large
    pub fn build_flat_lookup_table(&mut self) {
        let mut table = vec![NeuronArrayType::Invalid; u32::MAX as usize];
        
        for (model_type, range) in &self.id_manager.model_ranges {
            for id in range.start..range.end {
                table[id as usize] = *model_type;
            }
        }
        
        self.flat_lookup = Some(table);
    }
}
```

### 4.4 Reverse Lookup: Global ID from Local Index

**No storage needed - just calculate!**

```rust
impl NeuronIdManager {
    #[inline]
    pub fn to_global_id(&self, model_type: NeuronArrayType, local_index: u32) -> Option<u32> {
        if let Some(range) = self.model_ranges.get(&model_type) {
            Some(range.start + local_index)  // O(1) addition!
        } else {
            None
        }
    }
}
```

**Performance**: 2 ns (no HashMap needed!)  
**Memory**: 0 bytes (12 GB saved compared to reverse HashMap!)

---

## 5. Model-Aware Fire Candidate List

### 5.1 The Critical Performance Problem

**With model-specific arrays, naive FCL processing would be catastrophic:**

```rust
// ❌ CATASTROPHIC: Lookup for EVERY FCL entry!
for (target_id, potential) in fcl.get_all() {
    let (model, local_idx) = id_manager.lookup(target_id);  // 20 ns × 100K = 2 ms!
    match model {
        LIF => lif_array.membrane_potentials[local_idx] += potential,
        Izhikevich => izh_array.membrane_potentials[local_idx] += potential,
    }
}
```

**Problem**: 100K FCL entries × 20 ns = **2 ms per burst** (at 10K bursts/sec = 20 cores just for lookups!)

### 5.2 Solution: Model-Aware FCL

**Key Insight**: Look up model type ONCE during synaptic propagation, cache in FCL.

```rust
/// Fire Candidate List with model type caching
pub struct FireCandidateList {
    /// Candidates grouped by model type (ZERO lookups in neural dynamics!)
    candidates_by_model: HashMap<NeuronArrayType, Vec<(u32, f32)>>,
    
    /// Cache: neuron_id → model_type (populated during synaptic propagation)
    /// High hit rate: neurons with multiple inputs hit cache
    neuron_model_cache: HashMap<u32, NeuronArrayType>,
}

impl FireCandidateList {
    /// Add candidate with lazy model type lookup
    pub fn add_candidate(
        &mut self,
        neuron_id: u32,
        contribution: f32,
        id_router: &NeuronRouter,
    ) {
        // Check cache first (99% hit rate for neurons with multiple inputs)
        let model_type = *self.neuron_model_cache
            .entry(neuron_id)
            .or_insert_with(|| id_router.get_model_type(neuron_id));
        
        // Add to model-specific list
        self.candidates_by_model
            .entry(model_type)
            .or_insert_with(Vec::new)
            .push((neuron_id, contribution));
    }
    
    /// Get candidates grouped by model (zero-copy iterator)
    pub fn iter_by_model(&self) -> impl Iterator<Item = (NeuronArrayType, &[(u32, f32)])> {
        self.candidates_by_model.iter().map(|(k, v)| (*k, v.as_slice()))
    }
}
```

### 5.3 Performance Analysis

**Synaptic Propagation Phase:**

```
Scenario: 10K neurons fire, 100K synaptic events, 10K unique targets

Lookups needed:
- First synapse to neuron: 20 ns (cache miss)
- Next 9 synapses to same neuron: <1 ns (HashMap hit)

Total: 10K unique × 20 ns = 200 μs ✅
Cache hits: 90K × 1 ns = 90 μs ✅
Total overhead: ~300 μs per burst ✅
```

**Neural Dynamics Phase:**

```rust
// ✅ ZERO LOOKUPS! Pre-grouped by model
for (model_type, candidates) in fcl.iter_by_model() {
    match model_type {
        LIF => {
            for (global_id, potential) in candidates {
                let local_idx = global_id - lif_range.start;  // O(1) subtraction
                lif_array.membrane_potentials[local_idx] += potential;
            }
        }
        Izhikevich => {
            // Same pattern - zero lookups, pure SIMD
        }
    }
}
```

**Total overhead**: 300 μs synaptic + 0 ns neural = **300 μs per burst** ✅

### 5.4 Cross-Model Synapses

**Fully supported - no restrictions!**

```rust
// Visual Cortex (LIF) → Motor Cortex (Izhikevich)
let visual_neuron_lif = 10_000_042;  // LIF neuron fires
let motor_neuron_izh = 30_000_123;   // Izhikevich target

// Synaptic propagation
fcl.add_candidate(motor_neuron_izh, +0.5, &id_router);
  → Lookup: 30_000_123 is in Izhikevich range
  → Add to fcl.candidates_by_model[Izhikevich]

// Neural dynamics
for (global_id, potential) in fcl.get_by_model(Izhikevich) {
    // Process with Izhikevich dynamics
    // Source model doesn't matter - synaptic input is model-agnostic!
}
```

**Key Point**: SOURCE model doesn't matter. TARGET model determines dynamics.

### 5.5 Memory Overhead

```
Typical FCL: 10K unique neurons
  - candidates_by_model: ~100 KB (grouped vectors)
  - neuron_model_cache: 10K × 5 bytes = 50 KB
  Total: ~150 KB ✅

Large FCL: 100K unique neurons
  - candidates_by_model: ~1 MB
  - neuron_model_cache: 500 KB
  Total: ~1.5 MB ✅
```

**Negligible compared to neuron data!**

---

## 6. Synaptic Integration

### 6.1 Synapse Array (Unchanged)

**Synapses use global IDs - work transparently across all models**

```rust
pub struct SynapseArray {
    pub source_neurons: Vec<u32>,       // Global IDs
    pub target_neurons: Vec<u32>,       // Global IDs
    pub weights: Vec<u8>,
    pub postsynaptic_potentials: Vec<u8>,          // PSP values (u8)
    pub types: Vec<u8>,
    
    /// Index: source_global_id → Vec<synapse_indices>
    pub source_index: AHashMap<u32, Vec<usize>>,
}
```

**Key Insight**: Synapse creation/lookup doesn't need to know about neuron models.

### 6.2 Synaptic Propagation with Model-Aware FCL

```rust
impl RustNPU {
    pub fn process_synaptic_propagation(
        &mut self,
        fired_neurons: &[u32],
        id_router: &NeuronRouter,
    ) -> FireCandidateList {
        let mut fcl = FireCandidateList::new();
        
        for &source_global_id in fired_neurons {
            if let Some(synapse_indices) = self.synapse_array.source_index.get(&source_global_id) {
                for &syn_idx in synapse_indices {
                    let target_global_id = self.synapse_array.target_neurons[syn_idx];
                    let weight = self.synapse_array.weights[syn_idx];
                    let psp = self.synapse_array.postsynaptic_potentials[syn_idx];
                    
                    // Calculate contribution (model-agnostic, canonical absolute-u8 contract)
                    // contribution = weight × psp  (both are absolute u8 units 0..255)
                    let contribution = (weight as f32) * (psp as f32);
                    
                    // Accumulate to FCL with model type lookup (cached!)
                    fcl.add_candidate(target_global_id, contribution, id_router);
                }
            }
        }
        
        fcl
    }
}
```

**Critical**: Synaptic input is model-agnostic. Only neural dynamics differ per model.

---

## 6. GPU Backend Strategy

### 6.1 Model-Segregated Batching

**Problem**: GPU hates branch divergence (different code paths per thread)

**Solution**: Process one model at a time with homogeneous kernels

```rust
impl GPUBackend {
    pub fn process_neural_dynamics(&mut self, npu: &mut RustNPU, fcl: &FireCandidateList) -> Vec<u32> {
        let mut all_fired = Vec::new();
        
        // Group FCL entries by model type
        let lif_inputs = fcl.filter_by_type(NeuronArrayType::LIF);
        let izh_inputs = fcl.filter_by_type(NeuronArrayType::Izhikevich);
        
        // Process each model with dedicated kernel (no branching!)
        if !lif_inputs.is_empty() {
            self.upload_lif_data(&npu.lif_neurons);
            let lif_fired = self.launch_lif_kernel(&lif_inputs);
            self.download_lif_results(&mut npu.lif_neurons, &lif_fired);
            all_fired.extend(lif_fired);
        }
        
        if !izh_inputs.is_empty() {
            self.upload_izhikevich_data(&npu.izhikevich_neurons);
            let izh_fired = self.launch_izhikevich_kernel(&izh_inputs);
            self.download_izhikevich_results(&mut npu.izhikevich_neurons, &izh_fired);
            all_fired.extend(izh_fired);
        }
        
        all_fired
    }
}
```

### 6.2 Performance Characteristics

**Kernel Launch Overhead**: ~100μs per kernel

**Typical Brain**: 25 cortical areas, 3 models → 3 kernel launches = 300μs overhead

**Processing Time**: 10-50ms for 1M neurons

**Overhead**: <1% (negligible)

### 6.3 GPU Efficiency

✅ **No branch divergence**: Each kernel processes homogeneous neurons  
✅ **Perfect memory coalescing**: Compact, contiguous model-specific arrays  
✅ **Optimal occupancy**: All threads execute same code path  
✅ **Cache-friendly**: Each kernel only touches its model's data  

---

## 7. LIF Model Specification

### 7.1 Standardized Formula

**All backends MUST implement this exact formula**

```
Membrane Potential Update (per burst):
    V(t+1) = V(t) + I_syn - g_leak * (V(t) - V_rest)

Where:
    I_syn = Σ (weight × psp) for all active synapses
    V = membrane potential
    g_leak = leak_coefficient (0-1)
    V_rest = resting_potential
    weight = synaptic weight (0-1)
    psp = postsynaptic_potential (0-1)

Firing Check:
    if V(t+1) ≥ threshold:
        FIRE
        V = V_reset (or V_rest)
        refractory_countdown = refractory_period + snooze_period

Refractory Period:
    if refractory_countdown > 0:
        refractory_countdown -= 1
        Skip firing check
```

### 7.2 Critical: Remove 10.0 Scale Factor

**Current Bug**: Different backends use different formulas

```rust
// ❌ CPU Backend (has 10.0 multiplier)
let contribution = weight * psp * 10.0;

// ❌ GPU Backend (missing 10.0 multiplier)
let contribution = weight * psp;
```

**Fix**: Standardize to **no arbitrary scaling**

```rust
// ✅ All Backends (standardized)
let contribution = weight * psp;  // Both absolute u8 units (0..255), direct-cast to f32
```

**If scaling needed**: Adjust `psp` values in genome, not hardcode in dynamics.

### 7.3 Terminology Correction

**OLD (incorrect neuroscience term)**:
```rust
pub postsynaptic_potentials: Vec<u8>;  // ✅ PSP values
```

**NEW (correct FEAGI term)**:
```rust
pub postsynaptic_potentials: Vec<u8>;  // ✅ Correct (psp or pstcr_)
```

**Rationale**: In FEAGI, this represents postsynaptic current (pstcr_), not a separate parameter.

---

## 8. Implementation Phases

### Phase 1: Foundation (Week 1-2)
- [ ] Create `NeuronArrayType` enum
- [ ] Implement `NeuronRouter` with range-based lookup
- [ ] Create `LIFNeuronArray` structure
- [ ] Update ID allocation system
- [ ] Write unit tests for router

### Phase 2: LIF Migration (Week 3-4)
- [ ] Migrate existing neurons to `LIFNeuronArray`
- [ ] Update NPU to use router for lookups
- [ ] Fix LIF formula (remove 10.0 scale factor)
- [ ] Standardize across CPU/GPU/WebGPU backends
- [ ] Integration tests for LIF model

### Phase 3: Izhikevich Model (Week 5-6)
- [ ] Implement `IzhikevichNeuronArray`
- [ ] Add Izhikevich dynamics (CPU)
- [ ] Add Izhikevich GPU kernel
- [ ] Genome support for model selection per area
- [ ] Cross-model synapse tests

### Phase 4: AdEx Model (Week 7-8)
- [ ] Implement `AdExNeuronArray`
- [ ] Add AdEx dynamics (CPU)
- [ ] Add AdEx GPU kernel
- [ ] Performance benchmarking

### Phase 5: Polish & Optimization (Week 9-10)
- [ ] GPU batching optimization
- [ ] Memory usage profiling
- [ ] Documentation
- [ ] Migration guide for existing genomes

---

## 9. Migration Path

### 9.1 Backward Compatibility

**Existing genomes work without changes**
- Default model: LIF (if not specified)
- Memory neurons: unchanged (ID range 50M-100M)
- Synapses: transparent (use global IDs)

### 9.2 Opt-In Model Selection

**Genome Configuration**:
```json
{
  "blueprint": {
    "iic400": {
      "cortical_name": "Primary Visual Cortex",
      "neuron_model": "LIF",
      "model_parameters": {
        "leak_coefficient": 0.1,
        "resting_potential": 0.0
      }
    },
    "motor_cortex": {
      "cortical_name": "Motor Output",
      "neuron_model": "Izhikevich",
      "model_parameters": {
        "a": 0.02,
        "b": 0.2,
        "c": -65.0,
        "d": 8.0
      }
    }
  }
}
```

### 9.3 Python API

**Minimal changes required**:
```python
# Existing code continues to work
npu = RustNPUIntegration(connectome_manager)

# New model selection (optional)
npu.set_cortical_area_model("motor_cortex", "Izhikevich", {
    "a": 0.02,
    "b": 0.2,
    "c": -65.0,
    "d": 8.0
})
```

---

## 11. Folder Structure

### 11.1 Rust Crate Organization

```
feagi-core/
├── crates/
│   ├── feagi-types/
│   │   └── src/
│   │       ├── npu.rs                    # Base NeuronArray (current)
│   │       ├── neuron_models/            # NEW: Model-specific arrays
│   │       │   ├── mod.rs
│   │       │   ├── lif.rs                # LIFNeuronArray
│   │       │   ├── izhikevich.rs         # IzhikevichNeuronArray
│   │       │   ├── adex.rs               # AdExNeuronArray
│   │       │   └── traits.rs             # NeuronModel trait
│   │       └── id_manager/               # NEW: ID routing system
│   │           ├── mod.rs
│   │           ├── id_manager.rs         # NeuronIdManager (Roaring Bitmaps)
│   │           ├── router.rs             # NeuronRouter (lookup optimization)
│   │           └── types.rs              # NeuronArrayType enum
│   │
│   ├── feagi-burst-engine/
│   │   └── src/
│   │       ├── fire_structures.rs        # UPDATED: Model-aware FCL
│   │       ├── neural_dynamics/          # NEW: Model-specific dynamics
│   │       │   ├── mod.rs
│   │       │   ├── lif.rs                # LIF dynamics
│   │       │   ├── izhikevich.rs         # Izhikevich dynamics
│   │       │   └── adex.rs               # AdEx dynamics
│   │       ├── backend/
│   │       │   ├── cpu.rs                # UPDATED: Multi-model support
│   │       │   ├── wgpu_backend.rs       # UPDATED: Model-specific kernels
│   │       │   └── shaders/
│   │       │       ├── lif_dynamics.wgsl       # NEW: LIF kernel
│   │       │       ├── izhikevich_dynamics.wgsl # NEW: Izhikevich kernel
│   │       │       └── adex_dynamics.wgsl       # NEW: AdEx kernel
│   │       └── npu.rs                    # UPDATED: Multi-model NPU
│   │
│   └── feagi-plasticity/
│       └── src/
│           ├── neuron_id_manager.rs      # DEPRECATED: Migrate to feagi-types/id_manager
│           └── memory_neuron_array.rs    # UNCHANGED: Existing implementation
│
└── docs/
    └── MULTI_MODEL_NEURON_ARCHITECTURE.md  # THIS DOCUMENT
```

### 11.2 Python Bindings

```
feagi-rust-py-libs/
└── src/
    ├── feagi_python/
    │   ├── mod.rs                        # UPDATED: Multi-model bindings
    │   ├── neuron_models/                # NEW: Model-specific bindings
    │   │   ├── mod.rs
    │   │   ├── lif.rs
    │   │   └── izhikevich.rs
    │   └── id_manager.rs                 # NEW: ID manager bindings
    └── lib.rs

feagi-py/
├── feagi/
│   ├── npu/
│   │   ├── interface.py                  # UPDATED: Multi-model NPU interface
│   │   └── neuron_models/                # NEW: Python-side model config
│   │       ├── __init__.py
│   │       ├── lif.py
│   │       └── izhikevich.py
│   └── bdu/
│       └── embryogenesis/
│           └── neuroembryogenesis.py     # UPDATED: Model-aware neurogenesis
└── tests/
    └── multi_model/                      # NEW: Multi-model tests
        ├── test_id_manager.py
        ├── test_cross_model_synapses.py
        └── test_model_dynamics.py
```

### 11.3 Implementation Sequence

**Phase 0: Foundation (Week 1-2)**
```
1. feagi-types/src/id_manager/
   - Implement NeuronIdManager with Roaring Bitmaps
   - Implement NeuronRouter with range checks
   - Unit tests

2. feagi-types/src/neuron_models/
   - Create LIFNeuronArray structure
   - Create neuron model trait
   - Unit tests
```

**Phase 1: Core Integration (Week 3-4)**
```
3. feagi-burst-engine/src/
   - Update FireCandidateList to be model-aware
   - Implement LIF-specific neural dynamics
   - Integration tests

4. feagi-rust-py-libs/
   - Create Python bindings for ID manager
   - Create Python bindings for LIF model
   - Python integration tests
```

**Phase 2: Additional Models (Week 5-6)**
```
5. Add Izhikevich model
6. Add AdEx model
7. Cross-model synapse tests
```

---

## 12. Performance Analysis

### 12.1 Memory Overhead Summary

**For 1 Billion Neurons:**

| Component | Memory | Notes |
|-----------|--------|-------|
| **Neuron Arrays** | 60 GB | Model-specific (3 models @ 20 GB each) |
| **Roaring Bitmaps** | 200 MB | Free ID tracking (180× compression!) |
| **ID Lookup Table** | 4 GB | Optional (for >100M neurons) |
| **FCL Model Cache** | 0.5 MB | Per-burst, ephemeral |
| **Synapse Array** | 50 GB | Unchanged (independent of models) |
| **Total Overhead** | **4.2 GB** | Only ID management |
| **% Overhead** | **3.5%** | Acceptable for billion-scale |

**Without flat lookup table (<100M neurons):**
- Total Overhead: **200 MB** (only Roaring Bitmaps)
- % Overhead: **0.3%** ✅

### 12.2 Hot Path Performance Analysis

**Synaptic Propagation (Phase 1):**

| Operation | Current System | Multi-Model | Overhead |
|-----------|---------------|-------------|----------|
| Synapse lookup | 10 ns | 10 ns | 0 ns ✅ |
| Target ID fetch | 2 ns | 2 ns | 0 ns ✅ |
| Contribution calc | 5 ns | 5 ns | 0 ns ✅ |
| **Model type lookup** | N/A | **20 ns** | **+20 ns** ⚠️ |
| FCL accumulate | 5 ns | 5 ns | 0 ns ✅ |
| **Per synapse** | **22 ns** | **42 ns** | **+91%** ⚠️ |

**Impact at Scale:**
```
100K synaptic events, 10K unique targets:
  - Lookups: 10K × 20 ns = 200 μs
  - Cache hits: 90K × 1 ns = 90 μs
  Total overhead: 290 μs per burst ✅

At 10,000 bursts/sec: 2.9 seconds CPU time = 3 cores
```

**Acceptable** for <10 model types. Use flat lookup table for >100M neurons.

**Neural Dynamics (Phase 2):**

| Operation | Current System | Multi-Model | Overhead |
|-----------|---------------|-------------|----------|
| FCL iterate | 2 ns | 2 ns | 0 ns ✅ |
| **Model lookup** | N/A | **0 ns** | **0 ns** ✅ |
| Array index calc | 2 ns | 4 ns | +2 ns |
| MP update | 5 ns | 5 ns | 0 ns ✅ |
| Dynamics | 50 ns | 50 ns | 0 ns ✅ |
| **Per neuron** | **59 ns** | **61 ns** | **+3%** ✅ |

**Key Win**: Model-aware FCL eliminates lookups in neural dynamics!

### 12.3 GPU Performance

**Kernel Launch Overhead:**

| Models | Launch Overhead | Processing Time | % Overhead |
|--------|----------------|-----------------|------------|
| 1 (current) | 100 μs | 10 ms | 1.0% |
| 3 (typical) | 300 μs | 10 ms | 3.0% ✅ |
| 10 (extreme) | 1 ms | 10 ms | 10.0% ⚠️ |

**At 10,000 bursts/sec:**
- 3 models: 3 ms/sec = **0.3% GPU time** ✅
- 10 models: 10 ms/sec = **1% GPU time** ✅

**Acceptable overhead** even with many models.

**GPU Efficiency Gains:**
- ✅ Zero branch divergence (homogeneous kernels)
- ✅ Perfect memory coalescing (model-specific arrays)
- ✅ Optimal occupancy (all threads same code path)
- ✅ Better cache utilization (compact data)

**Net Result**: Multi-model GPU may be **faster** than unified despite overhead!

### 12.4 Worst-Case Scenario Analysis

**Pathological Case: 1M Neurons Fire, 10M Synaptic Events**

```
Synaptic Propagation:
  - 1M unique targets
  - Model lookups: 1M × 20 ns = 20 ms ❌
  - With flat table: 1M × 2 ns = 2 ms ✅
  
Neural Dynamics:
  - Model-aware FCL: 0 ns overhead ✅
  - Process 1M neurons: ~50 ms
  
Total: 2 ms + 50 ms = 52 ms per burst
At 10K bursts/sec: Need 520 cores ❌
```

**Mitigation**: Use flat lookup table (4 GB) for billion-neuron brains.

### 12.5 Comparison: Current vs. Multi-Model

**Small Brain (1M neurons, 10M synapses):**

| Metric | Current | Multi-Model | Difference |
|--------|---------|-------------|------------|
| Memory | 180 MB | 60 MB | **-66%** ✅ |
| Burst Time | 1 ms | 1.3 ms | +30% ⚠️ |
| GPU Time | 100 μs | 300 μs | +200% |
| **Verdict** | - | **Acceptable** | Marginal impact |

**Large Brain (100M neurons, 1B synapses):**

| Metric | Current | Multi-Model (no table) | Multi-Model (with table) |
|--------|---------|----------------------|------------------------|
| Memory | 18 GB | 6.2 GB | **10.2 GB** |
| Burst Time | 100 ms | 120 ms | **102 ms** |
| GPU Time | 10 ms | 30 ms | 30 ms |
| **Verdict** | - | ⚠️ Overhead matters | **✅ Recommended** |

**Billion-Scale Brain (1B neurons, 10B synapses):**

| Metric | Current | Multi-Model (with table) |
|--------|---------|------------------------|
| Memory | 180 GB | **64 GB** (-64%) |
| Burst Time | 1000 ms | **1020 ms** (+2%) |
| **Verdict** | - | **✅ Essential** |

---

## 13. Concerns & Risks

### 13.1 Critical Risks ⚠️⚠️⚠️

#### Risk 1: Premature Optimization

**Concern**: Are we solving a problem that doesn't exist yet?

**Questions:**
- Do you have concrete plans to use multiple neuron models?
- Is 60% memory reduction (180 MB → 60 MB) critical at 1M scale?
- Are you hitting GPU branch divergence issues now?
- Is current LIF model insufficient?

**If answers are "no/marginal/no/no"**: **DON'T IMPLEMENT YET**

**Recommendation**: Wait until you have:
- ✅ Concrete need for Izhikevich or AdEx
- ✅ Evidence that memory is a bottleneck
- ✅ Proof that GPU branching is slow

#### Risk 2: Implementation Complexity

**Estimated Effort:**
- +4,300 lines of Rust code
- +1,500 lines of Python bindings
- 250+ new unit/integration tests
- 4-6 weeks development time
- High bug risk in production system

**Current system works!** Adding complexity risks breaking what exists.

#### Risk 3: Lookup Overhead at Billion-Scale

**Problem**: Without flat lookup table, billion-neuron systems may need **2000+ cores** just for lookups.

**Mitigation**: Mandatory flat lookup table (4 GB) for >100M neurons.

**Trade-off**: 4 GB RAM vs. 2000 CPU cores → **RAM is cheaper!**

### 13.2 Medium Risks ⚠️⚠️

#### Risk 4: GPU Kernel Proliferation

**Issue**: Each model needs dedicated GPU kernel.

**Impact**:
- 3 models = 300 μs overhead per burst ✅
- 10 models = 1 ms overhead per burst ⚠️

**Limitation**: WebGPU doesn't support function pointers → can't unify kernels.

**Mitigation**: Limit to 3-5 core models, document overhead.

#### Risk 5: Brain Snapshot Compatibility

**Problem**: Changing model-specific arrays breaks existing brain snapshots.

**Impact**: All saved brains need migration scripts.

**Mitigation**:
- Implement snapshot versioning
- Provide automatic migration tools
- Keep legacy loader for old snapshots

### 13.3 Low Risks ⚠️

#### Risk 6: Roaring Bitmap Memory Growth

**Concern**: What if 50% of neurons get deleted?

**Analysis**:
```
1B neurons, 500M freed:
  - Roaring Bitmap: ~500 MB (worst case)
  - Still < 1 GB ✅
```

**Verdict**: Acceptable even in worst case.

#### Risk 7: Cross-Model Synapse Performance

**Concern**: Do cross-model synapses have overhead?

**Analysis**: No! Synaptic contribution is model-agnostic. Zero overhead.

**Verdict**: Not a concern.

### 13.4 Risk Mitigation Summary

| Risk | Severity | Mitigation | Status |
|------|----------|------------|--------|
| Premature optimization | ⚠️⚠️⚠️ | Wait for concrete need | **BLOCK** |
| Implementation complexity | ⚠️⚠️⚠️ | Phased rollout | **CAUTION** |
| Lookup overhead | ⚠️⚠️⚠️ | Flat lookup table (4 GB) | **SOLVED** |
| GPU kernel overhead | ⚠️⚠️ | Limit to 5 models | **ACCEPTABLE** |
| Snapshot compat | ⚠️⚠️ | Migration tools | **SOLVABLE** |
| Roaring Bitmap growth | ⚠️ | Monitor, acceptable | **LOW** |
| Cross-model synapses | ⚠️ | None needed | **NOT A CONCERN** |

---

## 14. Decision Framework

### 14.1 When to Implement This

**✅ PROCEED if ALL of these are true:**
1. You need Izhikevich or AdEx model for specific research
2. Memory is constrained (<10 GB available for 1B neurons)
3. You have 4-6 weeks dedicated development time
4. Team can handle 4,300 lines of code review
5. Testing infrastructure can handle 250+ new tests

**❌ DON'T PROCEED if ANY of these are true:**
1. No immediate need for alternative models
2. Memory is abundant (>50 GB available)
3. Current LIF model meets all requirements
4. Team focused on other priorities
5. Production system stability is critical

### 14.2 Phased Implementation Strategy

**Recommended Approach: Incremental**

**Phase 0: Documentation Only (NOW)**
- ✅ Write architecture document (THIS DOCUMENT)
- ✅ Review with team
- ✅ Identify concrete use cases
- ❌ No implementation yet

**Phase 1: Foundation (If Needed)**
- Implement ID manager with Roaring Bitmaps
- Implement neuron router
- NO model-specific arrays yet
- Measure overhead in isolation
- **Go/No-Go Decision**: Is overhead acceptable?

**Phase 2: Single Alternative Model (If Phase 1 succeeds)**
- Add ONLY Izhikevich (not LIF, not AdEx)
- Keep existing NeuronArray as LIF
- Test in production
- Measure real-world performance
- **Go/No-Go Decision**: Does it solve real problem?

**Phase 3: Full Multi-Model (If Phase 2 succeeds)**
- Migrate LIF to model-specific array
- Add AdEx and other models
- Full GPU support
- Production rollout

### 14.3 Success Metrics

**Must achieve ALL of these in Phase 1:**
- ✅ Lookup overhead <500 μs per burst
- ✅ Memory overhead <1 GB
- ✅ Zero regressions in existing tests
- ✅ No production issues for 1 week

**Must achieve ALL of these in Phase 2:**
- ✅ Alternative model works correctly
- ✅ Cross-model synapses work
- ✅ Performance within 10% of baseline
- ✅ Solves identified use case

**Only proceed to Phase 3 if Phases 1&2 succeed.**

### 14.4 Alternative: Defer Until Rust Migration

**Consider**: Is this better done during full Rust NPU rewrite?

**Pros**:
- Clean slate implementation
- No backward compatibility burden
- Can optimize from scratch

**Cons**:
- Delays multi-model support
- May never happen if Python works

**Recommendation**: If Rust NPU rewrite planned within 6 months, **wait**. Otherwise, proceed with phased approach.

---

## 15. Success Criteria

### 15.1 Functional Requirements
- ✅ Multiple models in same brain
- ✅ Different models per cortical area
- ✅ Synapses work across model boundaries
- ✅ Brain snapshots save/load correctly
- ✅ Dynamic area creation/deletion works
- ✅ ID recycling prevents fragmentation

### 15.2 Performance Requirements
- ✅ Lookup overhead <300 μs per burst (typical case)
- ✅ Memory overhead <4 GB (billion-neuron brain)
- ✅ GPU efficiency >90% (minimal branch divergence)
- ✅ No more than 3% overhead vs. single-model

### 15.3 Quality Requirements
- ✅ Deterministic: Same results on CPU/GPU/RTOS
- ✅ Type-safe: Compile-time model validation
- ✅ Testable: Each model has unit tests
- ✅ Documented: Architecture docs + API docs
- ✅ Backward compatible: Existing brains work

---

## 16. References

### Code References
- **Current Memory Neuron Implementation**: `feagi-core/crates/feagi-plasticity/src/memory_neuron_array.rs`
- **Current Neuron ID Manager**: `feagi-core/crates/feagi-plasticity/src/neuron_id_manager.rs`
- **GPU Backend Design**: `feagi-core/crates/feagi-burst-engine/docs/GPU_IMPLEMENTATION.md`
- **Fire Candidate List**: `feagi-core/crates/feagi-burst-engine/src/fire_structures.rs`
- **Neural Dynamics**: `feagi-core/crates/feagi-burst-engine/src/neural_dynamics.rs`

### External References
- **Roaring Bitmaps**: https://github.com/RoaringBitmap/roaring-rs
- **LIF Neuron Model**: [Wikipedia - Integrate-and-fire neuron](https://en.wikipedia.org/wiki/Biological_neuron_model#Leaky_integrate-and-fire)
- **Izhikevich Model**: Izhikevich, E.M. (2003). "Simple model of spiking neurons". IEEE Transactions on Neural Networks, 14(6), 1569-1572.
- **AdEx Model**: Brette & Gerstner (2005). "Adaptive exponential integrate-and-fire model". Journal of Neurophysiology, 94(5), 3637-3642.

---

## 17. Open Questions

1. **Heterogeneous Synapses**: Should synapse properties vary based on source/target model?
2. **Dynamic Model Switching**: Support changing cortical area model at runtime (requires synapse rewiring)?
3. **Custom Models**: Plugin system for user-defined models?
4. **Plasticity**: How do STDP and other plasticity rules work across models?
5. **Flat Lookup Table Threshold**: At what neuron count does 4 GB overhead become worthwhile?

---

## Appendix A: Quick Reference Tables

### A.1 Memory Overhead Summary

| Brain Size | Roaring Bitmaps | Flat Lookup | Total Overhead | % of Neuron Data |
|------------|----------------|-------------|----------------|------------------|
| 1M neurons | <1 MB | Not needed | **<1 MB** | <1% |
| 10M neurons | ~10 MB | Not needed | **~10 MB** | <1% |
| 100M neurons | ~100 MB | 4 GB | **~4.1 GB** | ~40% ⚠️ |
| 1B neurons | ~200 MB | 4 GB | **~4.2 GB** | ~7% ✅ |

**Recommendation**: Use flat lookup table for >100M neurons.

### A.2 Performance Overhead Summary

| Brain Size | Synaptic Overhead | Neural Overhead | Total per Burst | @ 10K bursts/sec |
|------------|------------------|----------------|----------------|------------------|
| 1M neurons | ~30 μs | 0 ns | **~30 μs** | 0.3 sec/sec |
| 10M neurons | ~100 μs | 0 ns | **~100 μs** | 1 sec/sec |
| 100M neurons (no table) | ~1 ms | 0 ns | **~1 ms** | 10 sec/sec ❌ |
| 100M neurons (with table) | ~100 μs | 0 ns | **~100 μs** | 1 sec/sec ✅ |
| 1B neurons (with table) | ~200 μs | 0 ns | **~200 μs** | 2 sec/sec ✅ |

**Recommendation**: Mandatory flat lookup table for >50M neurons.

### A.3 Implementation Checklist

**Phase 0: Planning** (Completed)
- ✅ Architecture document complete
- ✅ Folder structure established
- ⬜ Team review and approval
- ⬜ Identify concrete use case
- ⬜ Go/No-Go decision

**Phase 1: ID Manager** (2 weeks)
- ⬜ Implement `NeuronIdManager` with Roaring Bitmaps
- ⬜ Implement `NeuronRouter`
- ⬜ Add `roaring` crate dependency
- ⬜ Unit tests (30 tests)
- ⬜ Performance benchmarks
- ⬜ Go/No-Go decision

**Phase 2: First Alternative Model** (3 weeks)
- ⬜ Implement `IzhikevichNeuronArray`
- ⬜ Update `FireCandidateList` to be model-aware
- ⬜ Implement Izhikevich dynamics
- ⬜ Python bindings
- ⬜ Integration tests (50 tests)
- ⬜ Production testing (1 week)
- ⬜ Go/No-Go decision

**Phase 3: Full Multi-Model** (4 weeks)
- ⬜ Migrate LIF to model-specific array
- ⬜ Implement AdEx model
- ⬜ GPU kernel support (all models)
- ⬜ Brain snapshot migration tools
- ⬜ Full test suite (250 tests)
- ⬜ Documentation update
- ⬜ Production rollout

---

## Appendix B: Critical Decision Point

### **STOP: Read This Before Implementing**

**This architecture document describes a major refactor.** Before proceeding:

1. **Answer these questions honestly:**
   - Do you have a concrete, immediate need for Izhikevich or AdEx neurons?
   - Is memory a proven bottleneck (not theoretical)?
   - Do you have 4-6 weeks of dedicated development time?
   - Is the team prepared for 4,300+ lines of code review?

2. **If ANY answer is "no":**
   - ❌ **DON'T IMPLEMENT**
   - ✅ **Keep this document for future reference**
   - ✅ **Focus on other priorities**

3. **If ALL answers are "yes":**
   - ✅ **Start with Phase 1 only**
   - ✅ **Measure overhead before continuing**
   - ✅ **Require Go/No-Go approval at each phase**

**Remember**: The current system works. Premature optimization is the root of all evil.

---

**Status**: Phase 0.5 - Folder Structure Established  
**Implementation**: ✅ Structure created, LIF-only placeholders  
**Runtime Impact**: ✅ Zero - no multi-model complexity added  
**Next Action**: Team review, then decide on Phase 1 (ID Manager)  
**Document Version**: 2.1  
**Last Updated**: 2025-10-27  

**End of Architecture Document**

