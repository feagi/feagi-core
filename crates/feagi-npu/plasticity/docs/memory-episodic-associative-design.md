# Memory: Episodic vs Associative — Design Decisions

This document records agreed **architecture and behavior** for episodic (pattern/hash) memory vs associative (STDP-based) memory in FEAGI plasticity. It is the reference for genome rules, morphology options, runtime routing, and future implementation.

---

## 1. Terminology

| Term | Meaning |
|------|--------|
| **Memory cortical area** | Cortical area with memory semantics; uses the plasticity **memory neuron array** (global ids), not dense `NeuronStorage` for those units. |
| **Interconnect** | Non-memory cortical areas used as associative “hubs” (language in product docs; implementation uses cortical types as today). |
| **Episodic (pattern) path** | Upstream pattern detection → **hash / pattern association** with a memory neuron. Does **not** rely on a pattern-link **synapse** in the connectome for that binding. |
| **Associative (STDP) path** | **STDP-driven** plasticity where the **source** of the mapping is a **memory** cortical area. Implemented with **real synapses** on the associative mapping. |

---

## 2. Interconnect ↔ interconnect (plastic)

- Plasticity is expressed as **one directional STDP connection per mapping**.
- **Bidirectional** plasticity between two interconnects is achieved by registering **two** one-directional STDP mappings (A→B and B→A), not a single “default bidirectional” flag.

---

## 3. Upstream → memory (what is allowed)

### 3.1 Non-memory → memory

- **Only episodic** (pattern / hash) wiring into the memory area.
- **No** associative STDP **from** non-memory **into** memory.

### 3.2 Memory → memory

- **Episodic, associative, or both** may apply, depending on genome/mappings.
- **Episodic** and **associative** use **separate mechanisms** (see §6 and §7).

---

## 4. Memory → downstream (memory as source)

- When a **memory** area connects **out** to another area (interconnect, **another memory** area, core, IPU, OPU), the **only** plastic option for that **associative** story is **STDP** (directional mapping from genome/morphology).
- Memory areas are **1×1×1**; long-term memory neurons (per lifecycle rules) can participate in building/strengthening/weakening synapses toward the destination per STDP rules (first co-activation can create the edge; then LTP/LTD as defined).

---

## 5. Definition: “Associative memory” (product / genome)

- **Associative memory** = **STDP-based** connection whose **source** cortical area is a **memory** type.
- It is **not** globally bidirectional by default.
- It may be:
  - memory → memory, or  
  - memory → non-memory (interconnect, core, IPU, OPU).
- **Two** opposing one-directional **memory→memory** associative mappings behave as **effective bidirectional** associative memory (same idea as two one-way interconnect STDP links).
- The NPU **does not** create a reverse synapse when only one directional mapping exists; bidirectional behavior requires **two** registered mappings (A→B and B→A).

---

## 6. Independent temporal windows vs shared spike visibility (memory → memory)

When **both** episodic and associative mechanisms apply between two memory areas:

| Mechanism | Window / bookkeeping |
|-----------|----------------------|
| **Episodic (pattern)** | Pattern / hash association uses its **own** temporal semantics (e.g. pattern detector / `temporal_depth`). That **bookkeeping** is not merged with STDP’s pairing logic. |
| **Associative (STDP)** | Uses the **STDP / plasticity** window (fire ledger + mapping). **Episodic activations** (pattern-driven memory neuron spikes) are **the usual triggers** for associative STDP: those spikes are recorded on the **same** STDP fire ledger so source/destination co-activation can create a synapse if missing and then apply LTP/LTD. |

**Pattern machinery** and **STDP eligibility predicates** (e.g. assoc/LTM) remain separate concerns; **spike events** from the episodic path are visible to STDP for associative memory mappings.

---

## 7. Same-burst precedence

If **both** episodic activation and associative (LIF) integration would affect the **same** memory neuron in the **same** burst:

- **Episodic takes precedence**: the neuron fires per the episodic path for that burst.
- Episodic firing **does not reset** associative LIF state for later bursts (per agreed behavior). With **MP charge accumulation off** for memory neurons, there is **no** cross-burst MP carryover from accumulation semantics.

---

## 8. Synapses: what exists for each path

### 8.1 Episodic (pattern) path

- Does **not** create or update a **synapse** for the “pattern link” itself.
- Establishes / updates **hash ↔ memory neuron** association in the episodic / pattern machinery only.

### 8.2 Associative (STDP) path

- **Only** creates/updates **real synapses** on the **associative STDP mapping** (memory-as-source).

### 8.3 Memory → memory when both are active

- **Episodic**: still **never** creates/updates a synapse for the pattern link — **only** hash / memory-neuron association.
- **STDP**: **only** touches **real synapses** on the **associative** mapping.

---

## 9. Synapse tagging (runtime, efficient)

- **Associative** edges that exist as **physical synapses** carry a per-row **`edge_flags`** byte in `SynapseStorage` (std: `Vec<u8>` parallel to synapses; embedded: fixed array). Bit **`SYNAPSE_EDGE_ASSOCIATIVE_MEMORY`** (`feagi_npu_neural::synapse`) marks edges from **associative STDP** batch adds in the NPU and from **`associative_memory`** projector application when both areas are memory (see `apply_projector_morphology_with_dimensions` / `apply_function_morphology`). Other morphologies pass **`0`** unless extended later.
- **Episodic** binding does **not** require a synapse tag for the pattern link (no synapse for that binding).
- Genome: **morphology** distinguishes **associative memory mapping** (and related options) so edges can be stamped at creation once wired through.

---

## 10. Memory-neuron LIF parameters (associative path only) — implemented

**Implementation (`feagi_npu_burst_engine::sparse_memory_lif`, `process_neural_dynamics`, `NPU::process_burst`):**

- Parameters use the **same property names** as regular neurons, but apply only to **memory** cortical areas and the **memory neuron** subsystem — **no** mixing with dense `NeuronStorage`. Defaults are keyed by `cortical_idx` via `MemoryAssociativeLifParamsByArea`; `RustNPU::set_memory_associative_lif_params` / `DynamicNPU` expose configuration.
- **Sparse state**: `SparseMemoryAssociativeLifStates` allocates LIF-related state **only** for memory neuron ids that have **ever received non-zero associative PSP** (from synapses with `SYNAPSE_EDGE_ASSOCIATIVE_MEMORY`).
- **Off / omitted**: MP charge accumulation (always off); **no** firing threshold increment; **no** leak (integrate FCL drive only per burst).
- **Remaining LIF knobs** in the associative path: refractory, snooze, consecutive fire limit, firing threshold, firing threshold limit, excitability (see `MemoryAssociativeLifParams`).

**Routing:** Phase 1 accumulates associative contributions into `memory_associative_fcl_input` during synaptic propagation. Phase 2 resolves each memory candidate with `resolve_memory_neuron_output`: **episodic** staged `fire_kind` wins same burst (§7); else if both associative PSP map and LIF params are present, **sparse associative LIF**; else **legacy** instantaneous force-fire with STDP-eligible kind. GPU/CPU backends that do not wire associative maps pass empty maps / `None` and stay on the legacy path.

---

## 11. Relationship to existing code

- Synapse rows encode **associative STDP** edges via **`edge_flags`** (see §9).
- **Dual FireLedger:** `RustNPU` holds **`fire_ledger`** (full STDP archive: dense + **all** memory-neuron fires, including `FIRE_KIND_EPISODIC_MEMORY` pattern injections, so associative STDP can match co-activation) and **`episodic_memory_fire_ledger`** (subset: only episodic-tagged memory-neuron fires for consumers that need pattern-path-only history). Phase 3 archives `clone_for_stdp_fire_ledger` (full queue) / `clone_for_episodic_memory_fire_ledger` (see `feagi_npu_burst_engine::fire_structures`). `FiringNeuron::fire_kind` comes from staged injections or defaults to STDP-eligible for propagation-sourced memory candidates (`process_neural_dynamics`).
- **`register_memory_area`** configures upstream STDP windows on the main ledger **and** the memory cortical area on the episodic ledger (`configure_episodic_memory_fire_ledger_window` on `DynamicNPU`).
- `docs/INTEGRATION.md` in this crate covers **service wiring** and task history; this document covers **memory semantics** only.

---

## 12. Episodic scan and in-region classifier

- **`episodic_scan`** is a semantic-only function morphology. It never enters `get_episodic_memory_upstream_cortical_areas` or the holistic ID hash. Old episodic encode/replay is unchanged when no scan mapping exists.
- **Classifier assembly** is a first-class genome object under the top-level `classifiers` key (parallel to `brain_regions`). It is not a `BrainRegion` and is not exportable as a circuit. The record stores name, parent region, coordinates, referenced inputs, and owned internals (`kernel_memory_id`, `class_memory_id`, `scan_twin_id`). Neuroembryogenesis loads this map; area delete and mapping edits update the record and its required mappings.
- Wiring: kernel `--episodic_memory-->` Mem1; class `--episodic_memory-->` Mem2; Mem1 `--associative_memory-->` Mem2; field `--episodic_scan-->` Mem1.
- Scan twin dimensions are **field_x × field_y × C** where `C` is the class-area volume. The twin is a classical, connectable cortical area named `{classifier}_twin`, placed at the classifier X/Z with **Y above the classifier** by the twin's own height, with burst engine on. Classifier create/update also turns burst on for the bound field. It is not a field clone and does not receive `memory_replay`.
- Brain Visualizer stamp is the **memory assembly body** hosted on kernel memory: XY from stored internal memory neuron count (`ceil(sqrt(N))`), Z from kernel `temporal_depth`. Class memory stays hidden. Clicking the stamp selects the classifier. The twin is a standard interconnect with a visual classifier → twin edge; user mappings from the twin stay real connectome mappings.
- LTM stores a spatial occupancy sidecar (not a replacement hash). After encode, Mem1 LTM is bound to Mem2 class channels. Scan stamps class hits at `(x, y, class_z)` via NPU `schedule_replay_injection`.
- Skip the field when `F / field_volume > scan_skip_density`. Skip a window when occupancy `< min_window_activity`. Defaults come from `MemoryAreaProperties` (1 and 1.0). No kernel, class area, associative mapping, or scan source means no scan.

## 13. Revision history

| Date | Notes |
|------|--------|
| 2026-03-29 | Initial consolidation from architecture discussion (episodic vs associative, topology, windows, synapse rules). STDP fire ledger includes episodic-tagged memory spikes (associative STDP triggers); episodic ledger remains a subset view. |
| 2026-03-29 | Associative memory STDP: single registered mapping creates edges only in that direction (no automatic reverse synapse); two mappings for true bidirectional. |
| 2026-03-28 | `SynapseStorage::edge_flags`; STDP batch + connectome `associative_memory` (memory↔memory) stamp `SYNAPSE_EDGE_ASSOCIATIVE_MEMORY`; `count_synapses_with_edge_flag_bits` on NPU. |
| 2026-03-28 | Dual `FireLedger` + `FiringNeuron::fire_kind`; episodic vs STDP archive paths; plasticity registers episodic memory area window. |
| 2026-03-28 | §10: sparse associative LIF for memory neurons (`sparse_memory_lif`, `memory_associative_fcl_input` from propagation, `process_neural_dynamics` wiring); §10 marked implemented. |
| 2026-09-20 | §12: `episodic_scan`, spatial LTM sidecar, class-map twin, in-region classifier assembly. |
| 2026-09-20 | §12: first-class genome `classifiers` key (parallel to `brain_regions`), not exportable as a circuit. |
| 2026-09-20 | §12: stamp is kernel-memory assembly (XY from neuron count, Z from temporal depth); twin is a separate field_x×field_y×C cortical area. |
| 2026-09-21 | §12: twin Y is above the classifier by the twin height; X/Z stay on the classifier. |
| 2026-09-21 | §12: classifier create/update keeps twin and field `burst_engine_active`. |
