# Fine-Grained Locking Refactoring

## Status: IN PROGRESS

**Goal**: Replace coarse `Arc<Mutex<RustNPU>>` with fine-grained internal locking for concurrent hot-path operations.

## Changes Made

### 1. Structure Refactoring ✅
- Wrapped `neuron_array` in `RwLock<NeuronArray>`
- Wrapped `synapse_array` in `RwLock<SynapseArray>`
- Grouped fire structures in `Mutex<FireStructures>`
- Wrapped `area_id_to_name` in `RwLock<HashMap>`
- Wrapped `propagation_engine` in `RwLock<>`
- Changed `burst_count` to `AtomicU64`
- Changed `power_amount` to `AtomicU32` (f32 bits)

### 2. Methods Requiring Updates

#### Lock Access Pattern:
```rust
// OLD:
self.neuron_array.method()

// NEW (read):
self.neuron_array.read().unwrap().method()

// NEW (write):
self.neuron_array.write().unwrap().method()
```

#### Methods to Update (by category):

**Neuron Operations** (~15 methods):
- `add_neuron()` - needs `.write()`
- `add_neurons_batch()` - needs `.write()`
- `batch_coordinate_lookup()` - needs `.read()`
- `get_neuron_count()` - needs `.read()`
- `get_neurons_by_cortical_area()` - needs `.read()`
- `update_neuron_property()` - needs `.write()`
- ... (see full list in npu.rs)

**Synapse Operations** (~10 methods):
- `add_synapse()` - needs `.write()`
- `add_synapses_batch()` - needs `.write()`
- `remove_synapses_from_sources()` - needs `.write()`
- `get_synapse_count()` - needs `.read()`
- ... (see full list in npu.rs)

**Fire Structure Operations** (~8 methods):
- `inject_to_fcl()` - needs fire_structures `.lock()`
- `inject_sensory_batch()` - needs fire_structures `.lock()`
- `sample_fire_queue()` - needs fire_structures `.lock()`
- `get_current_fcl()` - needs fire_structures `.lock()`
- ... (see full list in npu.rs)

**Propagation Engine Operations** (~3 methods):
- neuron_to_area updates - needs propagation_engine `.write()`
- ... (see full list in npu.rs)

**Atomic Operations** (already done ✅):
- `get_burst_count()` - uses atomic load
- `increment_burst_count()` - uses atomic fetch_add
- `set_power_amount()` - uses atomic store
- `get_power_amount()` - uses atomic load

### 3. External Access Points

**In `burst_loop_runner.rs`**:
- Needs public accessor methods for burst loop
- Currently fails because `neuron_array` is private
- Solution: Add `pub(crate) fn get_neuron_array_read()` methods

**In Python bindings (`feagi-rust-py-libs`)**:
- Currently wraps RustNPU in `Arc<Mutex<>>`
- After refactoring: Can use `Arc<>` (no outer mutex needed!)
- Huge win: Python FFI calls no longer require outer mutex

## Next Steps

1. ✅ Remove duplicate `get_burst_count()` at line 656
2. Add accessor methods for burst_loop_runner
3. Update all neuron operations to use `.read()`/`.write()`
4. Update all synapse operations to use `.read()`/`.write()`
5. Update all fire structure operations to use `.lock()`
6. Update propagation engine accesses
7. Remove outer `Arc<Mutex<>>` from Python bindings
8. Test with high-resolution video

## Performance Benefits

**Before** (coarse locking):
```
Sensory Injection: BLOCKS EVERYTHING  
Burst Processing: BLOCKS EVERYTHING
API Query: BLOCKS EVERYTHING
→ Result: Serial execution, API unresponsive
```

**After** (fine-grained locking):
```
Sensory Injection: locks only fire_structures
Burst Processing: reads neurons/synapses concurrently
API Query: reads stats atomically (no lock!)
→ Result: Concurrent execution, API responsive
```

## Estimation

- **Total methods to update**: ~50-60
- **Compilation errors**: ~30-40
- **Time**: 2-3 hours of systematic refactoring
- **Risk**: Medium (many changes, but mechanical)
- **Benefit**: High (fixes API unresponsiveness, enables true concurrency)

