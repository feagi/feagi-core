# Memory Area: Membrane Potential-Aware Encoding

Design proposal for adding optional MP (membrane potential) learning to episodic memory areas, enabling pattern reconstruction with intensity fidelity (e.g., autoencoders, image regeneration from learned patterns).

---

## 1. Problem Statement

Current episodic memory captures **which** neurons fired (binary identity) and **where** they are (voxel coordinates), but discards **how strongly** they fired (membrane potential at fire time). This makes the memory sufficient for pattern recognition but incapable of faithful signal reconstruction.

| Current storage | Missing for reconstruction |
|----------------|---------------------------|
| `Vec<(u32, u32, u32)>` coords per frame | Per-coordinate membrane potential at encoding time |
| Pattern-only force-fire of those coords on the twin | Variable per-coordinate replay potential |

---

## 2. Design Decisions

### 2.1 Mode Toggle

Memory areas gain a boolean property `mp_learning_enabled` (default: `false`).

- **Off (default)**: `ReplayFrame` stores coords only. Replay force-fires those twin voxels (authoritative FQ), ignoring per-voxel LIF thresholds. Membrane potentials are not reconstructed.
- **On**: `ReplayFrame` stores coords with associated MP values. Replay injects per-coordinate potentials so intensity also appears on the twin.

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
    /// When Some, replay injects each coordinate at its stored potential.
    /// When None, replay force-fires the stored coordinates (pattern-only).
    pub membrane_potentials: Option<Vec<f32>>,
}
```

### 3.3 `MemoryAreaConfig`

```rust
#[derive(Debug, Clone)]
pub struct MemoryAreaConfig {
    pub temporal_depth: u32,
    pub upstream_areas: Vec<u32>,
    /// PatternOnly | MpLearning | Change(MpChangeEncoding) -- see Section 13.
    pub mp_mode: MemoryMpMode,
}
```

### 3.4 `ReplayInjection` (fire_structures)

```rust
pub struct ReplayInjection {
    pub target_burst: u64,
    pub twin_area_idx: u32,
    pub coords: Vec<(u32, u32, u32)>,
    /// Force-fire stored coords, or inject stored per-coordinate MPs.
    pub potentials: ReplayPotentialMode,
}

pub enum ReplayPotentialMode {
    /// Pattern-only replay: force-fire each stored coord, bypassing LIF/threshold.
    ForceFire,
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
    None => ReplayPotentialMode::ForceFire,
};
```

### 5.2 Twin Area Injection

When processing `ReplayInjection` during the target burst:

- `ReplayPotentialMode::ForceFire`: Merge stored twin coords into the fire queue after Phase 2 (`pending_authoritative_fq`). They fire even when local thresholds exceed any PSP inject.
- `ReplayPotentialMode::PerCoordinate(mps)`: Each coordinate injected at its corresponding `mps[i]` value through LIF so stored intensities appear on the twin.

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
    mp_mode: MemoryMpMode,
) -> bool
```

Callers resolve `mp_mode` from genome properties with `feagi_brain_development::memory_mp_mode()`, which re-validates the MP settings. An invalid configuration is logged and the area is not registered.

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

> The MP Learning checkbox described here is superseded by the single MP Encoding dropdown in Section 13.4.

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
| Twin area creation | **Unchanged** -- twin dimensions and morphology remain the same. Pattern-only replay force-fires stored coords; MP-aware replay injects stored potentials. |
| Episodic precedence (Section 7 of base design) | **Unchanged**. |

---

## 11. Testing Requirements

1. **Unit**: `ReplayFrame` with/without MPs; EMA averaging correctness over multiple reactivations.
2. **Integration**: End-to-end encode-replay cycle verifying per-coordinate potentials arrive at twin area.
3. **Regression**: Pattern-only replay (`mp_learning_enabled = false`) still force-fires every stored twin coord, including voxels whose LIF threshold exceeds 1.0.
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

## 13. Change-Based MP Encoding (`mp_change_mode`)

MP learning (Sections 2-12) stores absolute potentials for replay. Change-based encoding instead makes **pattern identity** depend on how upstream MPs change across the temporal window, regardless of absolute level. Example (differential, depth 2): 2 -> 4 and 7 -> 9 activate the same memory neuron.

### 13.1 Properties

| Property | Flat key suffix | Type | Default | Meaning |
|----------|-----------------|------|---------|---------|
| `mp_change_mode` | `mpchg-t` | string | `none` | `none`, `mp_differential`, `mp_ratio` |
| `mp_delta_quantization` | `mpdlq-f` | float | `1.0` | Differential bucket width (MP units) |
| `mp_ratio_quantization` | `mprtq-f` | float | `20.0` | Ratio bucket width (percent, compounding) |

The keys are additive and optional (no schema bump). `mp_change_mode != none` and `mp_learning_enabled = true` are mutually exclusive; the BV exposes a single dropdown (none / MP learning / MP differential / MP ratio). Quantization must be finite and > 0.

Validation (`validate_memory_mp_properties`) runs in the genome validator, in the genome service update path (rejects with `InvalidInput`), and again at registration.

### 13.2 Encoding

For a window of D frames, only the D-1 step transitions are hashed (the first frame's absolute values are not). Per neuron per step, with `prior`/`current` being fire-time MPs from the FireLedger MP archive:

- **Differential**: `bucket = round((current - prior) / q)`. A neuron missing from a frame counts as MP 0; neurons absent from both frames are excluded.
- **Ratio**: `bucket = round(ln(current / prior) / ln(1 + q/100))`. Only positive -> positive steps count; steps involving 0, a missing neuron, or a negative value are ignored. At q = 20%, doubling = +4 and halving = -4 buckets.

Rounding is nearest. `ln`/`round` use `libm` in f64 for platform-deterministic results. Each step is a sorted list of `(neuron_id, bucket)`; empty steps are hashed as empty (their position matters). If every step is empty, no memory neuron forms.

The hash (xxh64, seed 0) is prefixed with a mode tag byte (1 = differential, 2 = ratio) so differential, ratio, and pattern-only encodings occupy separate hash inputs. Implementation: `feagi-npu/plasticity/src/mp_change_encoder.rs`.

`MemoryNeuronArray` keys memory neurons by `(memory_area_idx, pattern_hash)`. Memory areas with identical upstream wiring compute the same hash for the same input, but each area owns an independent neuron (own ID, lifecycle, synapses, LTM conversion), so an associative mapping from one area never fires on another area's detection.

### 13.3 Runtime behavior

- **Replay disabled**: change-mode memory neurons carry no replay frames (a change cannot be reconstructed into absolute twin activity). Neurons still form, fire, age, and convert to LTM, so they can be composed with other memory areas in larger circuits.
- **No MP averaging**: the EMA path (Section 2.2) applies only to `MpLearning`.
- **Temporal depth < 2**: no step exists; registration logs a warning and the area runs as `PatternOnly`. The genome validator reports a warning for the same case.
- **FireLedger**: MP archival is enabled for upstream areas of change-mode memory areas (`MemoryMpMode::requires_mp_archival`).

### 13.4 Reporting and clients

- **API**: `GET /v1/cortical_area/memory` returns the configured keys under `memory_parameters` and the runtime mode as top-level `effective_mp_mode` (`pattern_only`, `mp_learning`, `mp_differential`, `mp_ratio`; `null` if the area is not registered). The two differ when a change mode was auto-disabled.
- **BV**: one MP Encoding dropdown (None / MP Learning / MP Differential / MP Ratio) replaces the MP Learning checkbox in Advanced Cortical Properties and the create-memory dialog. A selection always writes both `mp_learning_enabled` and `mp_change_mode`. Only the quantization for the selected change mode is shown. Mapping lives in `CorticalPropertyMemoryParameters.gd`.
- **feagi-mcp**: `get_memory_area_runtime_config` returns an `mp_encoding` block (configured vs. effective mode, `auto_disabled`, `replay_enabled`, active quantization).

---

## 14. Revision History

| Date | Notes |
|------|-------|
| 2026-05-26 | Initial design. Mode toggle + EMA averaging (alpha=0.5) for MP-aware episodic memory encoding and replay. |
| 2026-09-29 | Added change-based encoding (`mp_change_mode`: differential and compounding ratio), runtime `MemoryMpMode`, replay disabled in change modes. `effective_mp_mode` in the memory API, BV MP Encoding dropdown, MCP `mp_encoding` report. Memory neurons keyed per memory area (fixes shared-upstream areas sharing one neuron). |
