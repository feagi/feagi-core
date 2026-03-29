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

---

## 6. Independent temporal windows (memory → memory)

When **both** episodic and associative mechanisms apply between two memory areas:

| Mechanism | Window / bookkeeping |
|-----------|----------------------|
| **Episodic (pattern)** | Uses its **own** dedicated temporal window (e.g. pattern detector / `temporal_depth` semantics). **Must not** be merged into the STDP window. |
| **Associative (STDP)** | Uses the **STDP / plasticity** window (e.g. fire ledger + mapping keyed by STDP rules). |

They remain **strictly independent**.

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

## 10. Planned: memory-neuron LIF parameters (associative path only)

**Scope (agreed direction, implementation may follow in a later change):**

- Parameters use the **same property names** as regular neurons, but apply only to **memory** cortical areas and the **memory neuron** subsystem — **no** mixing with dense `NeuronStorage`.
- **Sparse state**: allocate LIF-related state **only** for memory neuron ids that have **ever received associative** input (efficiency).
- **Off / omitted**: MP charge accumulation (always off); **no** firing threshold increment; **no** leak constant and **no** leak variability (dropped given no MP accumulation carryover design).
- Remaining LIF-related knobs (subject to final property list): e.g. refractory, snooze, consecutive fire limit, firing threshold, firing threshold limit, neuron excitability — as aligned with implementation PRs.

**Routing:** Associative LIF only applies when incoming drive is from **associative-tagged** synapses; episodic injection follows pattern/FCL semantics and **wins** same burst (§7).

---

## 11. Relationship to existing code

- Synapse rows encode **associative STDP** edges via **`edge_flags`** (see §9).
- **Dual FireLedger:** `RustNPU` holds **`fire_ledger`** (STDP / associative-eligible dense + memory fires with `fire_kind != FIRE_KIND_EPISODIC_MEMORY`) and **`episodic_memory_fire_ledger`** (only `FIRE_KIND_EPISODIC_MEMORY` memory-neuron fires from the pattern-injection path). Phase 3 archives `clone_for_stdp_fire_ledger` / `clone_for_episodic_memory_fire_ledger` (see `feagi_npu_burst_engine::fire_structures`). `Memory` `FiringNeuron::fire_kind` comes from staged injections or defaults to STDP-eligible for propagation-sourced memory candidates (`process_neural_dynamics`).
- **`register_memory_area`** configures upstream STDP windows on the main ledger **and** the memory cortical area on the episodic ledger (`configure_episodic_memory_fire_ledger_window` on `DynamicNPU`).
- `docs/INTEGRATION.md` in this crate covers **service wiring** and task history; this document covers **memory semantics** only.

---

## 12. Revision history

| Date | Notes |
|------|--------|
| 2026-03-29 | Initial consolidation from architecture discussion (episodic vs associative, topology, windows, synapse rules). |
| 2026-03-28 | `SynapseStorage::edge_flags`; STDP batch + connectome `associative_memory` (memory↔memory) stamp `SYNAPSE_EDGE_ASSOCIATIVE_MEMORY`; `count_synapses_with_edge_flag_bits` on NPU. |
| 2026-03-28 | Dual `FireLedger` + `FiringNeuron::fire_kind`; episodic vs STDP archive paths; plasticity registers episodic memory area window. |
