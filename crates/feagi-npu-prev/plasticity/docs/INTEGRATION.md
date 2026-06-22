# Plasticity Service Integration - Implementation Summary

## Completed Tasks (1-5)

### ✅ Task 1: Memory Stats Cache
- Created `memory_stats_cache.rs` with event-driven architecture
- `Arc<RwLock<HashMap<String, MemoryAreaStats>>>` for thread-safe access
- Helper functions: `on_neuron_created`, `on_neuron_deleted`, `init_memory_area`, `get_stats_snapshot`

### ✅ Task 2: PlasticityService Integration with Stats Cache
- Updated `PlasticityService::new()` to accept `MemoryStatsCache`
- Added `memory_area_names: Arc<Mutex<HashMap<u32, String>>>` mapping
- Integrated cache updates in:
  - `create_memory_neuron` → calls `on_neuron_created`
  - `age_memory_neurons` → calls `on_neuron_deleted` for died neurons
- Updated `register_memory_area` to accept `area_name: String` and initialize cache

### ✅ Task 3: Health Check Integration
- Added `memory_stats_cache: Option<MemoryStatsCache>` to `ApiState`
- Updated `feagi-api/Cargo.toml` to include `feagi-npu-plasticity` dependency
- Modified `get_health_check` endpoint to read from cache using `get_stats_snapshot`
- Returns memory stats as `HashMap<String, HashMap<String, serde_json::Value>>`

### ✅ Task 4: Plasticity Lifecycle Manager
- Created `lifecycle_manager.rs` with `PlasticityLifecycleManager`
- Features:
  - Dynamically starts/stops plasticity service based on memory area count
  - `register_memory_area()` → auto-starts service if first area
  - `unregister_memory_area()` → auto-stops service if count reaches 0
  - `notify_burst(timestep)` → forwards to service
  - `drain_commands()` → retrieves commands for NPU processing

### ✅ Task 5: Command Consumer Logic
- Added `drain_commands()` method to `PlasticityService`
- **Consumer responsibility**: a burst-engine integration layer (or other runtime orchestrator) must call `drain_commands()` and apply the resulting commands.
- Commands ready for **target NPU integration**:
  - `RegisterMemoryNeuron` → (target) register memory neuron into NPU neuron storage (or a dedicated memory-neuron execution path)
  - `InjectMemoryNeuronToFCL` → (target) add neuron to Fire Candidate List (only valid once neuron ID semantics are compatible)
  - `UpdateWeightsDelta` → apply STDP weight changes (future)

---

## Remaining Tasks (6-8) - Integration Steps

### Task 6: Add Plasticity Thread to BurstLoopRunner

**File**: `feagi-core/crates/feagi-npu/burst-engine/src/burst_loop_runner.rs`

**Required Changes**:

1. **Add plasticity manager to BurstLoopRunner struct**:
```rust
pub struct BurstLoopRunner {
    // ... existing fields ...
    
    /// Plasticity lifecycle manager (optional, only when memory areas exist)
    pub plasticity_manager: Option<Arc<Mutex<PlasticityLifecycleManager>>>,
}
```

2. **Update constructor to accept plasticity config**:
```rust
pub fn new<V, M>(
    npu: Arc<Mutex<DynamicNPU>>,
    viz_publisher: Option<Arc<Mutex<V>>>,
    motor_publisher: Option<Arc<Mutex<M>>>,
    frequency_hz: f64,
    plasticity_config: Option<(PlasticityConfig, MemoryStatsCache)>,  // NEW
) -> Self {
    let plasticity_manager = plasticity_config.map(|(config, cache)| {
        Arc::new(Mutex::new(PlasticityLifecycleManager::new(config, cache)))
    });
    
    // ... rest of initialization ...
}
```

3. **Notify plasticity service after each burst**:
In the burst loop (around line 500-600), after `npu.run_burst()`:
```rust
// Notify plasticity service
if let Some(ref plasticity_mgr) = self.plasticity_manager {
    let mgr = plasticity_mgr.lock().unwrap();
    mgr.notify_burst(burst_count);
}
```

4. **Process plasticity commands after burst**:
```rust
// Drain and process plasticity commands
if let Some(ref plasticity_mgr) = self.plasticity_manager {
    let commands = {
        let mgr = plasticity_mgr.lock().unwrap();
        mgr.drain_commands()
    };
    
    if !commands.is_empty() {
        self.process_plasticity_commands(commands)?;
    }
}
```

5. **Implement command processor**:
```rust
fn process_plasticity_commands(&self, commands: Vec<PlasticityCommand>) -> Result<(), String> {
    let mut npu = self.npu.lock().unwrap();
    
    for cmd in commands {
        match cmd {
            PlasticityCommand::RegisterMemoryNeuron {
                neuron_id,
                area_idx,
                threshold,
                membrane_potential,
            } => {
                // Register memory neuron in NPU's neuron array
                npu.register_neuron(neuron_id, area_idx, threshold, membrane_potential)?;
            }

            PlasticityCommand::MemoryNeuronConvertedToLtm { .. } => {
                // Optional: create an associative twin neuron for LTM (design-dependent)
            }
            
            PlasticityCommand::InjectMemoryNeuronToFCL {
                neuron_id,
                area_idx,
                membrane_potential,
                pattern_hash,
                is_reactivation,
            } => {
                // Inject to Fire Candidate List
                npu.inject_to_fcl(neuron_id, area_idx, membrane_potential)?;
            }
            
            PlasticityCommand::UpdateWeightsDelta { .. } => {
                // TODO: Implement STDP weight updates
                warn!("STDP weight updates not yet implemented");
            }
            
            PlasticityCommand::UpdateStateCounters { .. } => {
                // Stats tracking only, no NPU action needed
            }
        }
    }
    
    Ok(())
}
```

---

### Task 7: Genome Change Detection for Auto-Start/Stop

**Objective**: Detect when memory areas are added/removed from the genome and update the plasticity service accordingly.

**Integration Point**: Wherever the genome is applied to the NPU (likely in the `GenomeService` or when applying genome changes).

**Required API Calls**:

1. **When a memory cortical area is added**:
```rust
if let Some(ref plasticity_mgr) = burst_loop_runner.plasticity_manager {
    let mut mgr = plasticity_mgr.lock().unwrap();
    mgr.register_memory_area(
        area_idx,
        area_name.clone(),
        temporal_depth,
        upstream_areas,
        Some(lifecycle_config),
    );
}
```

2. **When a memory cortical area is removed**:
```rust
if let Some(ref plasticity_mgr) = burst_loop_runner.plasticity_manager {
    let mut mgr = plasticity_mgr.lock().unwrap();
    mgr.unregister_memory_area();
}
```

**Where to Look**:
- `feagi-core/crates/feagi-services/src/genome_service.rs` - genome application logic
- `feagi-core/crates/feagi-api/src/endpoints/genome.rs` - genome API endpoints
- `feagi-core/crates/feagi-brain-development/` - genome loading/parsing

**Search Strategy**:
```bash
# Find where cortical areas are created
grep -r "create.*cortical.*area" feagi-core/crates/
grep -r "apply_genome" feagi-core/crates/
grep -r "CorticalAreaType::Memory" feagi-core/crates/
```

---

### Task 8: Testing Memory Neuron Creation and Visualization

**Objective**: End-to-end verification that memory neurons are created, registered, and visualized in Brain Visualizer.

**Test Scenario**:
1. Load a genome with at least one memory cortical area
2. Connect upstream cortical areas (e.g., vision) with activity patterns
3. Verify:
   - Plasticity service auto-starts
   - Memory neurons are created (check `memory_area_stats` in health check)
   - Memory neurons appear in Type 11 packets (visualization stream)
   - Memory sphere size in BV reflects neuron count

**Manual Test Steps**:
```bash
# 1. Start FEAGI with a memory-enabled genome
cd feagi-core
cargo run --bin feagi-desktop --features services

# 2. Check health endpoint for memory stats
curl http://localhost:8000/v1/system/health | jq '.memory_area_stats'

# Expected output:
# {
#   "mem_00": {
#     "neuron_count": "5",
#     "created_total": "5",
#     "deleted_total": "0",
#     "last_updated": "1704067200000"
#   }
# }

# 3. Connect to visualization stream and verify memory neurons appear
# (This requires Brain Visualizer running and connected)
```

**Automated Test (if runtime infrastructure exists)**:
```rust
#[test]
fn test_memory_neuron_end_to_end() {
    // 1. Create NPU with memory area
    // 2. Create plasticity manager
    // 3. Register memory area
    // 4. Inject upstream activity pattern
    // 5. Run burst
    // 6. Drain commands and verify RegisterMemoryNeuron
    // 7. Check memory stats cache
    // 8. Verify neuron registered in NPU
}
```

---

## Summary of Changes Made

### New Files:
- `feagi-npu/plasticity/src/memory_stats_cache.rs`
- `feagi-npu/plasticity/src/lifecycle_manager.rs`
- `feagi-npu/plasticity/docs/INTEGRATION.md` (this file)

### Modified Files:
- `feagi-npu/plasticity/src/service.rs` - stats cache integration, drain_commands method
- `feagi-npu/plasticity/src/memory_neuron_array.rs` - added `get_cortical_area_id()`
- `feagi-npu/plasticity/src/lib.rs` - re-exported lifecycle manager
- `feagi-npu/plasticity/Cargo.toml` - added parking_lot, serde, tracing
- `feagi-api/src/transports/http/server.rs` - added memory_stats_cache to ApiState
- `feagi-api/src/endpoints/system.rs` - integrated stats cache into health check
- `feagi-api/Cargo.toml` - added feagi-npu-plasticity dependency

### Architecture Benefits:
- ✅ **Event-driven**: Stats updated on neuron creation/deletion, not via expensive queries
- ✅ **Dynamic Lifecycle**: Plasticity service only runs when memory areas exist
- ✅ **Thread-safe**: All shared state uses Arc/Mutex/RwLock
- ✅ **Decoupled**: Command queue separates plasticity logic from NPU modifications
- ✅ **Testable**: Clear separation of concerns, mockable interfaces
- ✅ **O(1) Health Check**: No NPU queries, just cache read

---

## Next Steps for User/Team

1. **Complete Task 6**: Integrate `PlasticityLifecycleManager` into `BurstLoopRunner`
2. **Complete Task 7**: Hook plasticity registration into genome application logic
3. **Complete Task 8**: End-to-end testing and validation
4. **Optional Enhancements**:
   - Add Prometheus metrics for plasticity stats
   - Implement STDP weight update processing
   - Add plasticity configuration to genome TOML
   - Create unit tests for lifecycle manager

---

**Status**: ✅ Core infrastructure complete (Tasks 1-5)
**Remaining**: Integration into runtime (Tasks 6-8) - requires deeper knowledge of NPU/genome loading code paths.

