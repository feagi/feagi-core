// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Sparse LIF state for **memory neuron ids** (associative STDP path only).
//!
//! See `plasticity/docs/memory-episodic-associative-design.md` §10: state is allocated only for
//! neurons that have received **associative-tagged** synaptic input. No leak/threshold-increment/
//! mp-charge-accumulation per design; refractory / excitability / consecutive limits match dense
//! naming.

use ahash::AHashMap;
use feagi_npu_neural::excitability_random;

use crate::fire_structures::{FiringNeuron, FIRE_KIND_EPISODIC_MEMORY, FIRE_KIND_STDP_ELIGIBLE};

/// Configurable LIF-style parameters for associative memory neurons (per memory cortical_area area).
#[derive(Debug, Clone)]
pub struct MemoryAssociativeLifParams {
    pub threshold: f32,
    pub threshold_limit: f32,
    pub excitability: f32,
    pub refractory_period: u16,
    pub consecutive_fire_limit: u16,
    pub snooze_period: u16,
}

impl Default for MemoryAssociativeLifParams {
    fn default() -> Self {
        Self {
            threshold: 1.0,
            threshold_limit: f32::MAX,
            excitability: 1.0,
            refractory_period: 0,
            consecutive_fire_limit: u16::MAX,
            snooze_period: 0,
        }
    }
}

/// Mutable state for one memory neuron on the associative path.
#[derive(Debug, Clone)]
pub struct MemoryAssociativeLifState {
    pub membrane_potential: f32,
    pub refractory_countdown: u16,
    pub consecutive_fire_count: u16,
}

impl MemoryAssociativeLifState {
    fn new() -> Self {
        Self {
            membrane_potential: 0.0,
            refractory_countdown: 0,
            consecutive_fire_count: 0,
        }
    }
}

/// Per-cortical_area-area defaults keyed by `cortical_idx` (memory areas).
pub type MemoryAssociativeLifParamsByArea = AHashMap<u32, MemoryAssociativeLifParams>;

/// Sparse map: global memory neuron id → associative LIF state.
pub type SparseMemoryAssociativeLifStates = AHashMap<u32, MemoryAssociativeLifState>;

/// Integrate one burst for a memory neuron using associative sparse LIF.
///
/// `assoc_psp` is summed from synapses with [`feagi_npu_neural::synapse::SYNAPSE_EDGE_ASSOCIATIVE_MEMORY`];
/// `non_assoc_psp` is the remainder of the FCL candidate (`total_fcl - assoc_psp`).
///
/// Returns [`FIRE_KIND_STDP_ELIGIBLE`] on fire; [`None`] if the neuron does not emit a spike this burst.
#[allow(clippy::too_many_arguments)]
pub fn step_sparse_memory_associative_lif(
    neuron_id: u32,
    cortical_idx: u32,
    assoc_psp: f32,
    non_assoc_psp: f32,
    burst_count: u64,
    state: &mut MemoryAssociativeLifState,
    p: &MemoryAssociativeLifParams,
) -> Option<FiringNeuron> {
    // Refractory: block burst and decrement (same semantics as dense LIF).
    if state.refractory_countdown > 0 {
        state.refractory_countdown -= 1;
        let consecutive_fire_limit_raw = p.consecutive_fire_limit;
        let consecutive_fire_limit = if consecutive_fire_limit_raw == 0 {
            u16::MAX
        } else {
            consecutive_fire_limit_raw
        };
        if state.refractory_countdown == 0
            && consecutive_fire_limit_raw != 0
            && consecutive_fire_limit != u16::MAX
            && state.consecutive_fire_count >= consecutive_fire_limit
        {
            state.consecutive_fire_count = 0;
        }
        return None;
    }

    // §10: no leak — integrate FCL drive only.
    let integrate = assoc_psp + non_assoc_psp;
    let current = state.membrane_potential + integrate;

    let above_min = current >= p.threshold;
    let below_max = current <= p.threshold_limit;

    if !(above_min && below_max) {
        state.membrane_potential = current;
        let consecutive_fire_limit_raw = p.consecutive_fire_limit;
        if consecutive_fire_limit_raw != 0 && consecutive_fire_limit_raw != u16::MAX {
            state.consecutive_fire_count = 0;
        }
        return None;
    }

    let consecutive_fire_limit_raw = p.consecutive_fire_limit;
    let consecutive_fire_limit = if consecutive_fire_limit_raw == 0 {
        u16::MAX
    } else {
        consecutive_fire_limit_raw
    };

    if state.consecutive_fire_count >= consecutive_fire_limit {
        state.consecutive_fire_count = 0;
        state.membrane_potential = current;
        return None;
    }

    let excitability = p.excitability;
    if excitability < 0.999 {
        if excitability <= 0.0 {
            state.membrane_potential = current;
            return None;
        }
        let random_val = excitability_random(neuron_id, burst_count);
        if random_val >= excitability {
            state.membrane_potential = current;
            return None;
        }
    }

    // Fire
    let mp_at_fire = current;
    state.membrane_potential = 0.0;
    let old_count = state.consecutive_fire_count;
    state.consecutive_fire_count = old_count.saturating_add(1);

    let refractory_period = p.refractory_period;
    let consecutive_fire_limit_raw = p.consecutive_fire_limit;
    let consecutive_fire_limit = if consecutive_fire_limit_raw == 0 {
        u16::MAX
    } else {
        consecutive_fire_limit_raw
    };

    if consecutive_fire_limit_raw != 0
        && consecutive_fire_limit != u16::MAX
        && state.consecutive_fire_count >= consecutive_fire_limit
    {
        let snooze = p.snooze_period;
        state.refractory_countdown = refractory_period.saturating_add(snooze);
    } else {
        state.refractory_countdown = refractory_period;
    }

    Some(FiringNeuron {
        neuron_id: feagi_npu_neural::types::NeuronId(neuron_id),
        membrane_potential: mp_at_fire,
        cortical_idx,
        x: 0,
        y: 0,
        z: 0,
        fire_kind: FIRE_KIND_STDP_ELIGIBLE,
    })
}

/// Ensure sparse state exists for `neuron_id` when `assoc_psp != 0` (first associative input).
pub fn ensure_sparse_state_for_associative_input(
    states: &mut SparseMemoryAssociativeLifStates,
    neuron_id: u32,
    assoc_psp: f32,
) {
    if assoc_psp != 0.0 && !states.contains_key(&neuron_id) {
        states.insert(neuron_id, MemoryAssociativeLifState::new());
    }
}

/// Resolve parameters for a memory cortical_area area.
pub fn resolve_memory_lif_params(
    cortical_idx: u32,
    by_area: &MemoryAssociativeLifParamsByArea,
) -> MemoryAssociativeLifParams {
    by_area.get(&cortical_idx).cloned().unwrap_or_default()
}

/// Episodic injection output: §7 precedence — does not mutate sparse associative state.
pub fn memory_neuron_episodic_fire(
    neuron_id: u32,
    cortical_idx: u32,
    membrane_potential: f32,
) -> FiringNeuron {
    FiringNeuron {
        neuron_id: feagi_npu_neural::types::NeuronId(neuron_id),
        membrane_potential,
        cortical_idx,
        x: 0,
        y: 0,
        z: 0,
        fire_kind: FIRE_KIND_EPISODIC_MEMORY,
    }
}

/// Legacy instantaneous fire for memory neurons with only non-associative FCL drive (no sparse state).
pub fn memory_neuron_legacy_force_fire(
    neuron_id: u32,
    cortical_idx: u32,
    membrane_potential: f32,
) -> FiringNeuron {
    FiringNeuron {
        neuron_id: feagi_npu_neural::types::NeuronId(neuron_id),
        membrane_potential,
        cortical_idx,
        x: 0,
        y: 0,
        z: 0,
        fire_kind: FIRE_KIND_STDP_ELIGIBLE,
    }
}

/// Resolve output for one memory-neuron FCL candidate: episodic precedence (§7), sparse associative LIF (§10), or legacy force-fire.
///
/// Pass an empty `sparse_lif` and `None` for `associative_psp` / `lif_params_by_cortical` to force the
/// legacy instantaneous path (e.g. CPU backend).
#[allow(clippy::too_many_arguments)]
pub fn resolve_memory_neuron_output(
    neuron_id: u32,
    cortical_idx: u32,
    total_fcl_candidate: f32,
    staged_fire_kind_from_injection: Option<u8>,
    burst_count: u64,
    associative_psp: Option<&AHashMap<u32, f32>>,
    sparse_lif: &mut SparseMemoryAssociativeLifStates,
    lif_params_by_cortical: Option<&MemoryAssociativeLifParamsByArea>,
) -> Option<FiringNeuron> {
    if staged_fire_kind_from_injection == Some(FIRE_KIND_EPISODIC_MEMORY) {
        return Some(memory_neuron_episodic_fire(
            neuron_id,
            cortical_idx,
            total_fcl_candidate,
        ));
    }

    let (Some(assoc_map), Some(params_by_area)) = (associative_psp, lif_params_by_cortical) else {
        return Some(memory_neuron_legacy_force_fire(
            neuron_id,
            cortical_idx,
            total_fcl_candidate,
        ));
    };

    let assoc = assoc_map.get(&neuron_id).copied().unwrap_or(0.0);
    let non = total_fcl_candidate - assoc;
    let p = resolve_memory_lif_params(cortical_idx, params_by_area);
    ensure_sparse_state_for_associative_input(sparse_lif, neuron_id, assoc);

    if sparse_lif.contains_key(&neuron_id) {
        let state = sparse_lif
            .get_mut(&neuron_id)
            .expect("sparse LIF state must exist");
        return step_sparse_memory_associative_lif(
            neuron_id,
            cortical_idx,
            assoc,
            non,
            burst_count,
            state,
            &p,
        );
    }

    Some(memory_neuron_legacy_force_fire(
        neuron_id,
        cortical_idx,
        total_fcl_candidate,
    ))
}
