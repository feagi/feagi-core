# Synaptic delay (burst-quantized) — architecture notes

## Purpose

Model **integer burst delays** per synapse: if a presynaptic neuron fires at burst `T`, the postsynaptic PSP contribution (same formula as today: weight, PSP dynamics, uniformity, `mp_driven_psp`, etc.) is applied at the **start** of burst `T + d`, where `d` is the per-synapse `delay_bursts` (minimum **1**). Delay **0** is invalid.

This is **orthogonal** to STDP/plasticity: plasticity uses **Fire Queue** and **Fire Ledger** (actual spike history). The delay machinery only affects **when** PSP is injected into the **Fire Candidate List (FCL)** for membrane dynamics.

## Genome (cortical mapping)

- Delay is specified **per cortical mapping rule** (same granularity as `postSynapticCurrent_multiplier`).
- Field name: **`synaptic_delay_bursts`** (integer `>= 1`). **Omitted** means **`1`** (legacy behavior: next burst).
- Flat array rules extend with a **9th** element (index `8`) for the same value; shorter arrays default index `8` to `1` during flat→object conversion.
- During synaptogenesis, the resolved value is **stored per synapse** in synapse SoA (`delay_bursts`), so the runtime and future GPU paths read a single column.

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
- **Invalid `0`**: rejected at synapse creation / genome parse; propagation may still clamp with `max(1, d)` only as a defensive invariant.

## Related code (anchors)

- Synapse storage trait: `feagi-npu/runtime` (`SynapseStorage`).
- Propagation: `burst-engine/src/synaptic_propagation.rs` (`propagate_delayed`).
- Phase 1 and end-of-burst hook: `burst-engine/src/npu.rs`.
- Mapping resolution: `feagi-brain-development` / `connectome_manager.rs` (`resolve_synapse_params_for_rule`, morphologies calling `add_synapse` with delay).
- Flat genome conversion: `feagi-evolutionary/.../converter_flat_full.rs` (`process_dstmap`).
