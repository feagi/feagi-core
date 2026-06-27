# Synaptic delay (burst-quantized) — architecture notes

## Purpose

Model **integer burst delays** per synapse: if a presynaptic neuron fires at burst `T`, the postsynaptic PSP contribution (same formula as today: weight, PSP dynamics, uniformity, `mp_driven_psp`, etc.) is applied at the **start** of burst `T + d`, where `d` is the per-synapse `delay_bursts` (minimum **1**). Delay **0** is invalid.

This is **orthogonal** to STDP/plasticity: plasticity uses **Fire Queue** and **Fire Ledger** (actual spike history). The delay machinery only affects **when** PSP is injected into the **Fire Candidate List (FCL)** for membrane dynamics.

## Genome (cortical mapping)

- Delay is specified **per cortical mapping rule** (same granularity as `postSynapticCurrent_multiplier`).
- Object-rule field name: **`synaptic_delay_bursts`** (integer `>= 1`). **Omitted** means **`1`** (legacy behavior: next burst).
- **Flat array dstmap rule** (same order as `feagi-evolutionary` `process_dstmap` and `feagi-api` `post_mapping_properties`):

| Index | Field |
|------:|--------|
| 0 | `morphology_id` (string) |
| 1 | `morphology_scalar` |
| 2 | `postSynapticCurrent_multiplier` |
| 3 | `plasticity_flag` |
| 4 | `plasticity_constant` |
| 5 | `ltp_multiplier` |
| 6 | `ltd_multiplier` |
| 7 | `plasticity_window` |
| 8 | **`synaptic_delay_bursts`** |

- Arrays with **fewer than 9** elements: index `8` is treated as **missing** and defaults to **`1`** during flat→object conversion (evolutionary importer) and when the API normalizes an 8-element array for the UI.
- During synaptogenesis, the resolved value is **stored per synapse** in synapse SoA (`delay_bursts`), so the runtime and future GPU paths read a single column.

### HTTP API (UI and tools)

- **`POST /v1/cortical_mapping/mapping_properties`** reads `cortical_mapping_dst` from the connectome and returns a **normalized JSON object per rule**, always including **`synaptic_delay_bursts`**. Array rules with only 8 elements get delay **`1`**; index `8` is parsed when present and **clamped to `>= 1`** (values below 1 become 1 for the response). Object rules use optional **`synaptic_delay_bursts`**, default **`1`**, same clamp.
- Raw responses (e.g. **`GET /v1/cortical_mapping/mapping`**, area blueprint JSON) pass through stored rules; clients that need a stable shape for editors should prefer **`POST .../mapping_properties`** or ensure they read **`synaptic_delay_bursts`** on object rules.

### Genome persistence (connectome service)

- Mapping writes (e.g. **`PUT /v1/cortical_mapping/mapping_properties`**, **`PUT /v1/cortical_mapping/mapping`**) go through **`feagi-services`** `update_cortical_mapping`.
- Full updates that include **`synaptic_delay_bursts`** on each rule are stored as-is in the genome.
- For **partial** mapping updates that merge into an existing rule, the service may **carry forward** `synaptic_delay_bursts` from the previous rule when the incoming object omits the key (same idea as plasticity-field merge), so delay is not accidentally dropped from the saved genome.

## Runtime scheduling (CPU path; GPU-compatible data layout)

### Problem

Previously, PSP from fires at burst `N − 1` was injected into the FCL at burst `N` only (implicit `d = 1`). With arbitrary `d`, contributions from burst `k` must arrive at burst `k + d`.

### Approach

1. **End of burst `k`** (after Phase 2, using the finalized fire queue for burst `k`): run synaptic propagation from all neurons that fired at `k`, using the same `propagate` math as today, but **tagging each contribution with that synapse’s `delay_bursts`**.
2. **Bucket** contributions by arrival burst `k + d` into a **sparse schedule**:
   - `arrival_burst → FireCandidateList` (same merge semantics as FCL: sum potentials per neuron).
   - Associative-memory PSP side-inputs (memory neuron ids) are scheduled the same way in a parallel map: `arrival_burst → (memory_neuron_id → accumulated PSP)`.
3. **Start of burst `M`** (Phase 1, before dynamics): **drain** the schedule for exactly burst `M` into the live `FireCandidateList` and into `memory_associative_fcl_input`, then apply power, sensory, fatigue, and existing sparse MP-reset logic.

No STDP or Fire Ledger changes: they still see spikes at the real fire bursts.

### Memory and performance

- **Sparse by arrival burst**: at most `O(d_max)` non-empty future buckets if every offset is used; typically far fewer.
- **GPU / embedded alignment**: synapse SoA adds a **`delay_bursts` column** (e.g. `u8`). Scheduling can later use a **fixed ring** of `d_max + 1` slots (same sparse FCL payload per slot) without changing the semantic model.
- **Max delay** is expected to stay within global simulation/burst-counter policy (fatigue/sleep/reset are separate lifecycle concerns).

## Default and validation

- **Default delay = 1** when not specified in genome or when loading old snapshots (missing delay column filled with `1`).
- **Invalid `0` in genome / BDU resolution**: `feagi-brain-development` rejects **`synaptic_delay_bursts < 1`** when resolving a rule (strict). The **mapping_properties** POST path clamps sub-1 values to **`1`** for normalized output only; persist **`>= 1`** in saved genomes to avoid ambiguity.
- Propagation may still clamp with `max(1, d)` only as a defensive invariant on the hot path.

## Related code (anchors)

- Synapse storage trait: `feagi-npu/runtime` (`SynapseStorage`).
- Propagation: `burst-engine/src/synaptic_propagation.rs` (`propagate_delayed`).
- Phase 1 and end-of-burst hook: `burst-engine/src/npu.rs`.
- Mapping resolution: `feagi-brain-development` / `connectome_manager.rs` (`resolve_synapse_params_for_rule`, morphologies calling `add_synapse` with delay).
- Flat genome conversion: `feagi-evolutionary/.../converter_flat_full.rs` (`process_dstmap`).
- Normalized mapping for clients: `feagi-api` / `endpoints/cortical_mapping.rs` (`post_mapping_properties`).
- Connectome / genome merge on update: `feagi-services` / `connectome_service_impl.rs` (cortical mapping update path).
