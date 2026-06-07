# Memory Area: Membrane Potential-Aware Encoding

Design proposal for adding optional MP (membrane potential) learning to episodic memory areas, enabling pattern reconstruction with intensity fidelity (e.g., autoencoders, image regeneration from learned patterns).

---

## 1. Problem Statement

Current episodic memory captures **which** neurons fired (binary identity) and **where** they are (voxel coordinates), but discards **how strongly** they fired (membrane potential at fire time). This makes the memory sufficient for pattern recognition but incapable of faithful signal reconstruction.

| Current storage | Missing for reconstruction |
|----------------|---------------------------|
| `Vec<(u32, u32, u32)>` coords per frame | Per-coordinate membrane potential at encoding time |
| Fixed replay potential (`firing_threshold + increment`) | Variable per-coordinate replay potential |

---

## 2. Design Decisions

### 2.1 Mode Toggle

Memory areas gain a boolean property `mp_learning_enabled` (default: `false`).

- **Off (default)**: Behavior unchanged. `ReplayFrame` stores coords only. Replay uses fixed potential from `MemoryReplayTarget`.
- **On**: `ReplayFrame` stores coords with associated MP values. Replay injects per-coordinate potentials.

This property is defined in the genome (memory cortical area properties) and propagated through `MemoryAreaConfig` at registration time.

### 2.2 MP Averaging on Reactivation (EMA with alpha=0.5)

When `mp_learning_enabled` is true and a known pattern is reactivated (same hash detected again):

```
stored_mp[i] = (stored_mp[i] + new_mp[i]) / 2.0
```

This is an exponential moving average with alpha=0.5. Recent exposures dominate:

| Reactivation count | Weight of most recent exposure | Weight of first exposure |
|-------------------|-------------------------------|------------------------|
| 1 | 50% | 50% |
| 3 | 50% | 12.5% |
| 5 | 50% | 3.1% |
| 10 | 50% | ~0.1% |

This produces fast adaptation to current input while retaining some history.

### 2.3 Future Extensibility

The averaging mode (alpha=0.5) is the initial implementation. Future parameters may allow configurable alpha or alternative strategies (true running mean, weighted decay, etc.). This document does not design those extensions -- they are noted as future work.

---

## 3. Data Structure Changes

### 3.1 `ReplayFrame` (plasticity service)

```rust
#[derive(Debug, Clone)]
pub struct ReplayFrame {
    pub offset: u32,
    pub upstream_area_idx: u32,
    pub coords: Vec<(u32, u32, u32)>,
    /// Per-coordinate membrane potential at encoding time.
    /// Present only when mp_learning_enabled=true for the memory area.
    /// Length matches `coords` when present.
    pub membrane_potentials: Option<Vec<f32>>,
}
```

### 3.2 `MemoryReplayFrame` (NPU burst-engine)

```rust
#[derive(Debug, Clone)]
pub struct MemoryReplayFrame {
    pub offset: u32,
    pub upstream_area_idx: u32,
    pub coords: Vec<(u32, u32, u32)>,
    /// Per-coordinate membrane potentials for MP-aware replay.
    /// When Some, replay injects each coordinate at its stored potential
    /// instead of the fixed twin target potential.
    pub membrane_potentials: Option<Vec<f32>>,
}
```

### 3.3 `MemoryAreaConfig`

```rust
#[derive(Debug, Clone)]
pub struct MemoryAreaConfig {
    pub temporal_depth: u32,
    pub upstream_areas: Vec<u32>,
    pub mp_learning_enabled: bool,
}
```

### 3.4 `ReplayInjection` (fire_structures)

```rust
pub struct ReplayInjection {
    pub target_burst: u64,
    pub twin_area_idx: u32,
    pub coords: Vec<(u32, u32, u32)>,
    /// When None, use the fixed MemoryReplayTarget.potential for all coords.
    /// When Some, inject each coordinate at its stored potential.
    pub potentials: ReplayPotentialMode,
}

pub enum ReplayPotentialMode {
    /// All coordinates replayed at a single fixed potential (current behavior).
    Fixed(f32),
    /// Each coordinate replayed at its own stored potential.
    PerCoordinate(Vec<f32>),
}
```

---

## 4. Encoding Pipeline Changes

### 4.1 `build_replay_frames` (when `mp_learning_enabled = true`)

Current implementation resolves neuron_id -> coordinates via `get_neuron_coordinates()`. The MP-aware path additionally queries the membrane potential at fire time for each neuron from the `FireLedger` or `FireQueue` archive.

Source of MP data: The `FireQueue` already carries `membrane_potential` per `FiringNeuron`. The `FireLedger` currently archives only `RoaringBitmap` (IDs). To support MP-aware encoding, the `FireLedger` needs an optional parallel structure that maps `neuron_id -> f32` for bursts within the temporal window, active only for upstream areas feeding MP-aware memory areas.

```rust
/// Extended fire history entry when MP learning is enabled for downstream memory.
pub struct MpAwareBurstRecord {
    pub bitmap: RoaringBitmap,
    pub membrane_potentials: AHashMap<u32, f32>,
}
```

The `PlasticityService::build_replay_frames` path becomes:

```rust
if mp_learning_enabled {
    let coords_with_mp: Vec<((u32, u32, u32), f32)> = bitmap
        .iter()
        .filter_map(|neuron_id| {
            let coord = npu_lock.get_neuron_coordinates(neuron_id)?;
            let mp = mp_record.get(&neuron_id)?;
            Some((coord, *mp))
        })
        .collect();
    // Split into parallel vecs
    let (coords, mps): (Vec<_>, Vec<_>) = coords_with_mp.into_iter().unzip();
    // ...
}
```

### 4.2 Reactivation with MP Averaging

When a pattern hash matches an existing memory neuron and `mp_learning_enabled = true`:

1. Retrieve the stored replay frames for the neuron
2. For each frame, pair stored MPs with newly observed MPs by coordinate
3. Apply `stored = (stored + new) / 2.0` per coordinate
4. Replace stored replay frames with updated values

Since coordinates are guaranteed identical (same hash = same fired neuron set = same coords after sorting), the parallel vectors align directly by index.

---

## 5. Replay Pipeline Changes

### 5.1 `schedule_memory_replay_from_fire_queue`

When building `ReplayInjection`:

```rust
let potential_mode = match &frame.membrane_potentials {
    Some(mps) => ReplayPotentialMode::PerCoordinate(mps.clone()),
    None => ReplayPotentialMode::Fixed(target.potential),
};
```

### 5.2 Twin Area Injection

When processing `ReplayInjection` during the target burst:

- `ReplayPotentialMode::Fixed(p)`: Current behavior -- all coords injected at potential `p`.
- `ReplayPotentialMode::PerCoordinate(mps)`: Each coordinate injected at its corresponding `mps[i]` value.

---

## 6. Genome / Registration Interface

### 6.1 Flat Genome Key

New flat genome entry per memory cortical area:

```
_____10c-{cortical_id}-cx-mplrn-b
```

| Segment | Value |
|---------|-------|
| Scope | `cx` (cortical-area level) |
| Suffix | `mplrn` (membrane potential learning) |
| Type | `-b` (boolean) |
| Default | `false` |

Sits alongside existing memory property keys:

| Property | Flat key suffix |
|----------|----------------|
| `is_mem_type` | `memory-b` |
| `longterm_mem_threshold` | `mem__t-i` |
| `lifespan_growth_rate` | `mem_gr-i` |
| `init_lifespan` | `mem_ls-i` |
| `temporal_depth` | `tmpdpt-i` |
| **`mp_learning_enabled`** | **`mplrn-b`** |

### 6.2 Genome Wiring (3 files)

1. **`converter_flat_full.rs`** -- Add `"mplrn-b"` -> `"mp_learning_enabled"` in `PROPERTY_MAPPINGS`.
2. **`converter_hierarchical_to_flat.rs`** -- Reverse mapping for genome saves.
3. **`genome/parser.rs`** -- Add `pub mp_learning_enabled: Option<bool>` to `RawCorticalArea`.

### 6.3 Memory Area Properties (runtime struct)

Add `mp_learning_enabled: bool` to `MemoryAreaProperties` in `feagi-evolutionary/src/plasticity_detector.rs`. Default: `false`.

Extraction in `extract_memory_properties()`:

```rust
mp_learning_enabled: properties
    .get("mp_learning_enabled")
    .and_then(|v| v.as_bool())
    .unwrap_or(false),
```

### 6.4 `register_memory_area` Signature

```rust
pub fn register_memory_area(
    &self,
    area_idx: u32,
    area_name: String,
    temporal_depth: u32,
    upstream_areas: Vec<u32>,
    lifecycle_config: Option<MemoryNeuronLifecycleConfig>,
    mp_learning_enabled: bool,
) -> bool
```

### 6.5 FireLedger Configuration

When `mp_learning_enabled = true`, the FireLedger window configuration for upstream areas must enable MP archival for those areas. This is a per-area setting -- upstream areas feeding only non-MP memory areas do not pay the storage cost.

---

## 7. API Layer Changes

### 7.1 `CorticalAreaInfo` DTO (`feagi-services/src/types/dtos.rs`)

Add field with serde rename consistent with existing memory fields:

```rust
#[serde(rename = "mp_learning_enabled", skip_serializing_if = "Option::is_none")]
pub mp_learning_enabled: Option<bool>,
```

Populated from `extract_memory_properties()` -- present only for memory areas.

### 7.2 `MemoryCorticalAreaParamsResponse` (memory inspector endpoint)

Add `mp_learning_enabled: bool` to the runtime memory parameters response.

### 7.3 PUT update path

`PUT /v1/cortical_area/cortical_area` already accepts arbitrary property keys that get merged into the cortical area's properties HashMap. The key `"mp_learning_enabled"` will be accepted and persisted to genome on save.

---

## 8. Brain Visualizer (BV) Changes

### 8.1 `CorticalPropertyMemoryParameters.gd`

Add field and parsing for `mp_learning_enabled`:

```gdscript
var mp_learning_enabled: bool = false

# In FEAGI_apply_detail_dictionary:
if "mp_learning_enabled" in data.keys():
    mp_learning_enabled = data["mp_learning_enabled"]
```

### 8.2 `AdvancedCorticalProperties.gd` -- Memory Section

Add a checkbox/toggle control for MP Learning in the memory parameters region:

```gdscript
_connect_control_to_update_button(_check_mp_learning, "mp_learning_enabled", _button_memory_send)
```

Refresh from cache:

```gdscript
_update_control_with_value_from_areas(_check_mp_learning, "memory_parameters", "mp_learning_enabled")
```

### 8.3 `AdvancedCorticalProperties.tscn`

Add a `CheckBox` node labeled "MP Learning" to the Memory collapsible section, below temporal_depth.

### 8.4 `PartSpawnCorticalAreaMemory.gd` (create memory area dialog)

Add checkbox to creation form. Include in `get_memory_parameters_for_api()` output.

### 8.5 Field Name Mapping

| Genome `properties` | API JSON | BV cache | BV PUT key |
|---------------------|----------|----------|------------|
| `mp_learning_enabled` | `mp_learning_enabled` | `mp_learning_enabled` | `mp_learning_enabled` |

No `neuron_` prefix needed -- this is a memory-area-level behavior toggle, not a per-neuron parameter.

---

## 9. Performance Considerations

| Concern | Mitigation |
|---------|-----------|
| Memory overhead of `Option<Vec<f32>>` per frame | Zero cost when `None` (mode off). When on: one f32 per fired neuron per temporal frame -- bounded by upstream area dimensions. |
| FireLedger MP archival cost | Only active for upstream areas feeding MP-aware memory. Uses `AHashMap<u32, f32>` keyed by neuron_id -- sparse, only fired neurons stored. |
| EMA computation on reactivation | O(n) where n = total coords across all frames for that neuron. Negligible relative to pattern detection cost. |
| Replay injection branching | Single match on `ReplayPotentialMode` -- branch predictor friendly for homogeneous workloads. |

---

## 10. Interaction with Existing Mechanisms

| Mechanism | Impact |
|-----------|--------|
| Pattern hash computation | **Unchanged** -- hash is still from neuron IDs only. MP values do not affect pattern identity. |
| Lifecycle (aging, LTM conversion) | **Unchanged** -- MP data persists with the replay frames regardless of lifecycle state. |
| Associative STDP path | **Unchanged** -- associative memory uses its own sparse LIF state independent of episodic replay. |
| Twin area creation | **Unchanged** -- twin dimensions and morphology remain the same. The fixed `potential` field on `MemoryReplayTarget` becomes a fallback for the `Fixed` mode. |
| Episodic precedence (Section 7 of base design) | **Unchanged**. |

---

## 11. Testing Requirements

1. **Unit**: `ReplayFrame` with/without MPs; EMA averaging correctness over multiple reactivations.
2. **Integration**: End-to-end encode-replay cycle verifying per-coordinate potentials arrive at twin area.
3. **Regression**: Existing memory tests pass unchanged when `mp_learning_enabled = false`.
4. **Benchmark**: Memory overhead and replay latency with MP learning on vs. off across various upstream area sizes.

---

## 12. Implementation Order

1. Add `mp_learning_enabled` to flat genome converters + `RawCorticalArea` + `MemoryAreaProperties`.
2. Add field to `MemoryAreaConfig` and propagate through `register_memory_area`.
3. Extend `FireLedger` with optional MP archival per tracked area.
4. Extend `ReplayFrame` / `MemoryReplayFrame` with `Option<Vec<f32>>`.
5. Modify `build_replay_frames` to capture MPs when mode is on.
6. Implement EMA averaging on reactivation path.
7. Extend `ReplayInjection` with `ReplayPotentialMode`.
8. Modify twin injection to use per-coordinate potentials.
9. API layer: `CorticalAreaInfo` DTO + memory params response.
10. BV: `CorticalPropertyMemoryParameters.gd`, `AdvancedCorticalProperties.gd/.tscn`, `PartSpawnCorticalAreaMemory.gd`.
11. Tests per Section 11.

---

## 13. Revision History

| Date | Notes |
|------|-------|
| 2026-05-26 | Initial design. Mode toggle + EMA averaging (alpha=0.5) for MP-aware episodic memory encoding and replay. |
