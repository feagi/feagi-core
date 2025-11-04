# Phase 1: Foundation Methods - Implementation Plan

**Date:** 2025-10-30  
**Scope:** 6 Priority-1 foundation methods for ConnectomeManager  
**Goal:** Enable genome loading, initialization, and persistence

---

## Overview

These 6 methods form the foundation that all other BDU functionality depends on:

| Method | Purpose | Complexity | Dependencies |
|--------|---------|------------|--------------|
| `instance()` | Singleton access | Low | None |
| `__init__()` | Initialization | Medium | NPU interface, state manager |
| `load()` | Load genome from file | High | Deserialization, cortical areas, neurons, synapses |
| `save()` | Save genome to file | High | Serialization, cortical areas, neurons, synapses |
| `prepare_for_new_genome()` | Clear and prep for new genome | Medium | Clear operations, memory calc |
| `resize_for_genome()` | Resize for genome requirements | Medium | Memory calculation |

---

## Current State Analysis

### What Exists in Rust

**`feagi-bdu/src/connectome_manager.rs`:**
- ✅ Basic struct `ConnectomeManager` with:
  - `cortical_areas: HashMap<String, CorticalArea>`
  - `brain_region_hierarchy: BrainRegionHierarchy`
  - `next_cortical_idx: AtomicU32`
  - Bidirectional mapping (ID ↔ index)

**Methods already implemented:**
- ✅ `new()` - Constructor
- ✅ `add_cortical_area()` - Basic CRUD
- ✅ `delete_cortical_area()` - Basic CRUD
- ✅ `get_cortical_area()` - Query
- ✅ `update_cortical_area_properties()` - Update
- ✅ `add_brain_region()` - Hierarchy
- ✅ `delete_brain_region()` - Hierarchy

### What's Missing

- ❌ Singleton pattern (Rust doesn't have class methods, need `OnceCell`)
- ❌ NPU interface integration
- ❌ Genome loading (JSON deserialization)
- ❌ Genome saving (JSON serialization)
- ❌ Clear/reset operations
- ❌ Memory calculation
- ❌ Neuron/synapse persistence

---

## Python Implementation Analysis

### 1. `instance()` - Singleton Pattern

**Python approach:**
```python
class ConnectomeManager:
    _instance = None
    _initialized = False
    
    @classmethod
    def instance(cls, config_or_max_neurons=10_000_000, max_synapses=100_000_000, backend=None):
        if cls._instance is None:
            cls._instance = cls.__new__(cls)
            cls._instance.__init__(config_or_max_neurons, max_synapses, backend)
            cls._initialized = True
        return cls._instance
```

**Rust approach:**
```rust
use std::sync::OnceLock;
use parking_lot::RwLock;

static CONNECTOME_MANAGER: OnceLock<Arc<RwLock<ConnectomeManager>>> = OnceLock::new();

pub fn get_connectome_manager() -> Arc<RwLock<ConnectomeManager>> {
    CONNECTOME_MANAGER.get_or_init(|| {
        Arc::new(RwLock::new(ConnectomeManager::new(/* config */)))
    }).clone()
}
```

### 2. `__init__()` - Initialization

**Key responsibilities:**
- Parse config (dict or int for max_neurons)
- Initialize NPU interface with backend
- Set up cortical areas storage
- Initialize brain region hierarchy
- Set up Morton spatial hash (optional Rust optimization)
- Initialize connectivity rules storage
- Reserve core areas ("_death", "_power")

**Dependencies:**
- NPU interface (already in `feagi-burst-engine`)
- Morton hash (can skip for now, optimize later)
- Config parsing (use `feagi.toml` or JSON)

### 3. `load()` - Genome Loading

**File format** (Python pickle):
```python
save_data = {
    "cortical_areas": {cid: area.to_dict() for cid, area in self.cortical_areas.items()},
    "brain_regions": self.brain_regions.copy(),
    "region_area_map": self.region_area_map.copy(),
    "connectivity_rules": self.connectivity_rules.copy(),
    "cortical_connections": self.cortical_connections.copy(),
    "neuron_data": {...},
    "synapse_data": {...},
    "metadata": {...}
}
```

**Rust approach:**
- **Option A:** Support Python pickle (hard, requires `pyo3`)
- **Option B:** **New JSON format** (recommended for Rust migration)
- **Option C:** Binary format (bincode, serde)

**Recommendation:** Use **JSON** for now (human-readable, debuggable), optimize to binary later.

### 4. `save()` - Genome Saving

**Key operations:**
1. Serialize cortical areas (`CorticalArea.to_dict()`)
2. Serialize brain regions
3. Serialize connectivity rules
4. Serialize neurons (from NPU)
5. Serialize synapses (from NPU)
6. Write metadata (version, counts)

**Challenge:** NPU data is in Rust `feagi-burst-engine`, need FFI or direct access.

### 5. `prepare_for_new_genome()` - Clear and Prep

**Steps:**
1. Detect existing brain state
2. Optionally save current state
3. Clear all cortical areas
4. Clear all neurons/synapses (via NPU)
5. Reset counters (next_cortical_idx, neuron IDs)
6. Ensure brain region structure exists
7. Calculate required memory
8. Resize if needed

### 6. `resize_for_genome()` - Memory Sizing

**Purpose:** Calculate and allocate required capacity based on genome structure.

**Formula:**
```python
neuron_space = sum(area['dimensions']['width'] * area['dimensions']['height'] * area['dimensions']['depth'] 
                   for area in genome['blueprint'])
# Add 20% overhead for dynamic growth
required_neurons = int(neuron_space * 1.2)
```

---

## Implementation Strategy

### Phase 1A: Singleton + Basic Init (Week 1)

**Tasks:**
1. Add `OnceLock` singleton pattern
2. Implement basic `new()` with config parsing
3. Wire up NPU interface (use existing `RustNPU` from `feagi-burst-engine`)
4. Test singleton access

**Deliverable:** `ConnectomeManager::get_instance()` works, NPU is connected.

### Phase 1B: Genome Save/Load (Week 2)

**Tasks:**
1. Define JSON genome format (compatible with Python genome structure)
2. Implement `save_to_json(path: &Path)`
3. Implement `load_from_json(path: &Path)`
4. Serialize/deserialize cortical areas
5. Serialize/deserialize brain regions
6. Test round-trip (save → load → verify)

**Deliverable:** Can load real FEAGI genome JSON files.

### Phase 1C: Neuron/Synapse Persistence (Week 3)

**Tasks:**
1. Add neuron serialization (call NPU methods)
2. Add synapse serialization
3. Implement `prepare_for_new_genome()`
4. Implement `resize_for_genome()`
5. Test with real genomes (different sizes)

**Deliverable:** Full genome load/save with neurons and synapses.

---

## Technical Decisions

### 1. File Format

**Decision:** JSON for genome structure, separate binary file for neuron/synapse data (optional optimization).

**Rationale:**
- JSON is debuggable
- Compatible with Python genome format
- Can optimize to binary later without API changes

### 2. Singleton Pattern

**Decision:** `OnceLock<Arc<RwLock<ConnectomeManager>>>`

**Rationale:**
- `OnceLock`: Thread-safe one-time initialization
- `Arc`: Multiple threads can hold references
- `RwLock`: Multiple readers OR one writer

### 3. NPU Integration

**Decision:** Store `Arc<RwLock<RustNPU>>` in ConnectomeManager

**Rationale:**
- NPU is already in Rust
- Direct access, no FFI overhead
- Shared ownership between BDU and burst engine

### 4. Config Source

**Decision:** Accept `FeagiConfig` struct (from `feagi.toml`)

**Rationale:**
- Consistent with existing config system
- No hardcoded defaults
- Supports dynamic sizing

---

## Success Criteria

**Phase 1 is complete when:**
- ✅ Singleton `get_connectome_manager()` works
- ✅ Can load a real FEAGI genome JSON (e.g., `sample_brains/simple.json`)
- ✅ Loaded genome has correct cortical areas, regions, neurons
- ✅ Can save genome back to JSON
- ✅ Round-trip works: load → save → load → verify identical
- ✅ `prepare_for_new_genome()` clears all data
- ✅ No hardcoded values, all from config
- ✅ All tests pass (no mocking of real data)

---

## Next Steps After Phase 1

With foundation in place, we can proceed to:
- **Phase 2 (P2):** Cortical Area Management (6 methods)
- **Phase 3 (P3):** Neuron Operations (6 methods)
- **Phase 4 (P4):** Connectivity/Synapses (5 methods)

---

## Questions for Review

1. **JSON vs Pickle:** Should we support loading existing Python pickle files, or require conversion to JSON first?
2. **Binary optimization:** Should we plan for binary neuron/synapse data now, or defer?
3. **Config source:** Use `feagi_configuration.toml` or accept JSON config in `load()`?
4. **Testing genomes:** Do we have sample genome files to test with?

**Recommendation:** Start with JSON-only, add pickle support later if needed. Use `feagi_configuration.toml` for system config, genome files for brain structure.




