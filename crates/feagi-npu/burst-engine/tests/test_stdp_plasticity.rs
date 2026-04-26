// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0
//! STDP integration tests for synaptic plasticity behavior.

use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::npu::{PlasticityMode, StdpMappingParams};
use feagi_npu_burst_engine::RustNPU;
use feagi_npu_burst_engine::FIRE_KIND_STDP_ELIGIBLE;
use feagi_npu_neural::types::{NeuronId, SynapticPsp, SynapticWeight};
use feagi_npu_neural::SynapseType;
use feagi_npu_runtime::StdRuntime;
use feagi_structures::genomic::cortical_area::CoreCorticalType;
use std::sync::Arc;

/// Create a minimal STDP test network with two cortical areas.
fn create_stdp_network() -> (
    RustNPU<StdRuntime, f32, CPUBackend>,
    Vec<NeuronId>,
    Vec<NeuronId>,
) {
    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    let mut npu = RustNPU::new(runtime, backend, 100, 1000, 10).unwrap();
    // Plasticity supplies assoc (active) + LTM predicates; tests treat injected globals as both.
    npu.set_memory_neuron_assoc_predicate(Some(Arc::new(|_| true)));
    npu.set_memory_neuron_longterm_predicate(Some(Arc::new(|_| true)));

    // Register core areas for deterministic neuron IDs.
    npu.register_cortical_area(0, CoreCorticalType::Death.to_cortical_id().as_base_64());
    npu.register_cortical_area(1, CoreCorticalType::Power.to_cortical_id().as_base_64());

    // Register source/destination cortical areas.
    npu.register_cortical_area(10, CoreCorticalType::Death.to_cortical_id().as_base_64());
    npu.register_cortical_area(11, CoreCorticalType::Death.to_cortical_id().as_base_64());

    let mut src_neurons = Vec::new();
    let mut dst_neurons = Vec::new();

    for i in 0..3 {
        let neuron = npu
            .add_neuron(1.0, f32::MAX, 0.1, 0.0, 0, 0, 1.0, 0, 0, true, 10, i, 0, 0)
            .unwrap();
        src_neurons.push(neuron);
    }

    for i in 0..3 {
        let neuron = npu
            .add_neuron(1.0, f32::MAX, 0.1, 0.0, 0, 0, 1.0, 0, 0, true, 11, i, 0, 0)
            .unwrap();
        dst_neurons.push(neuron);
    }

    (npu, src_neurons, dst_neurons)
}

/// Build STDP parameters for a mapping.
fn stdp_params(
    plasticity_window: usize,
    plasticity_constant: i64,
    ltp_multiplier: i8,
    ltd_multiplier: i8,
    bidirectional_stdp: bool,
    synapse_psp: f32,
    synapse_type: SynapseType,
) -> StdpMappingParams {
    StdpMappingParams {
        plasticity_window,
        plasticity_constant,
        ltp_multiplier,
        ltd_multiplier,
        bidirectional_stdp,
        synapse_psp,
        synapse_type,
        plasticity_mode: PlasticityMode::Stdp,
        eligibility_decay_bursts: 0,
        reward_source_area: None,
        punishment_source_area: None,
        max_weight: f32::INFINITY,
    }
}

/// Inject sensory activity and process a burst, returning the burst count.
fn process_burst_with_injection(
    npu: &mut RustNPU<StdRuntime, f32, CPUBackend>,
    neurons: &[(NeuronId, f32)],
) -> u64 {
    npu.inject_sensory_with_potentials(neurons);
    let result = npu.process_burst().unwrap();
    result.burst
}

/// STDP requires the same neuron(s) to fire across the full window.
#[test]
fn test_bidirectional_stdp_requires_consistent_neurons_across_window() {
    let (mut npu, src_neurons, dst_neurons) = create_stdp_network();

    npu.configure_fire_ledger_window(10, 3).unwrap();
    npu.configure_fire_ledger_window(11, 3).unwrap();

    let params = stdp_params(3, 1, 5, 0, true, 10.0, SynapseType::Excitatory);
    npu.register_stdp_mapping(10, 11, params).unwrap();

    // Burst 1: fire src0/dst0
    process_burst_with_injection(
        &mut npu,
        &[(src_neurons[0], 128.0), (dst_neurons[0], 128.0)],
    );
    // Burst 2: fire src1/dst1
    process_burst_with_injection(
        &mut npu,
        &[(src_neurons[1], 128.0), (dst_neurons[1], 128.0)],
    );
    // Burst 3: fire src2/dst2
    process_burst_with_injection(
        &mut npu,
        &[(src_neurons[2], 128.0), (dst_neurons[2], 128.0)],
    );

    // No synapse should form because no single neuron is present in all bursts.
    for src in &src_neurons {
        assert!(
            npu.get_outgoing_synapses(src.0).is_empty(),
            "Unexpected synapse creation for src neuron {}",
            src.0
        );
    }
}

/// Assert that the given neuron fired in the specified burst for the cortical area.
fn assert_neuron_fired(
    npu: &RustNPU<StdRuntime, f32, CPUBackend>,
    cortical_idx: u32,
    burst: u64,
    neuron: NeuronId,
) {
    let window = npu
        .get_fire_ledger_dense_window_bitmaps(cortical_idx, burst, 1)
        .unwrap();
    assert_eq!(window.len(), 1);
    assert!(
        window[0].1.contains(neuron.0),
        "Expected neuron {} to fire in burst {} for area {}",
        neuron.0,
        burst,
        cortical_idx
    );
}

#[test]
fn test_bidirectional_stdp_creates_synapse_after_full_window() {
    let (mut npu, src_neurons, dst_neurons) = create_stdp_network();

    npu.configure_fire_ledger_window(10, 2).unwrap();
    npu.configure_fire_ledger_window(11, 2).unwrap();

    let params = stdp_params(2, 1, 5, 2, true, 200.0, SynapseType::Excitatory);
    npu.register_stdp_mapping(10, 11, params).unwrap();

    let src = src_neurons[0];
    let dst = dst_neurons[0];

    let burst = process_burst_with_injection(&mut npu, &[(src, 128.0), (dst, 128.0)]);
    assert_neuron_fired(&npu, 10, burst, src);
    assert_neuron_fired(&npu, 11, burst, dst);
    assert!(
        npu.get_outgoing_synapses(src.0).is_empty(),
        "No synapse should form until the full window is observed"
    );

    let burst = process_burst_with_injection(&mut npu, &[(src, 128.0), (dst, 128.0)]);
    assert_neuron_fired(&npu, 10, burst, src);
    assert_neuron_fired(&npu, 11, burst, dst);

    let outgoing = npu.get_outgoing_synapses(src.0);
    assert_eq!(
        outgoing.len(),
        1,
        "Synapse should be created after full window"
    );
    let (target, weight, psp, synapse_type) = outgoing[0];
    assert_eq!(target, dst.0);
    assert_eq!(weight, 5.0);
    assert_eq!(psp, 200.0);
    assert_eq!(synapse_type, SynapseType::Excitatory as u8);
}

#[test]
fn test_bidirectional_stdp_with_memory_neuron_ids() {
    const MEMORY_NEURON_ID_START: u32 = 50_000_000;
    let (mut npu, _src_neurons, _dst_neurons) = create_stdp_network();

    npu.configure_fire_ledger_window(10, 2).unwrap();
    npu.configure_fire_ledger_window(11, 2).unwrap();

    let params = stdp_params(2, 1, 5, 0, true, 200.0, SynapseType::Excitatory);
    npu.register_stdp_mapping(10, 11, params).unwrap();

    let src = NeuronId(MEMORY_NEURON_ID_START);
    let dst = NeuronId(MEMORY_NEURON_ID_START + 1);

    npu.inject_memory_neuron_to_fcl_with_kind(src.0, 10, 2.0, FIRE_KIND_STDP_ELIGIBLE);
    npu.inject_memory_neuron_to_fcl_with_kind(dst.0, 11, 2.0, FIRE_KIND_STDP_ELIGIBLE);
    let burst = npu.process_burst().unwrap().burst;
    assert_neuron_fired(&npu, 10, burst, src);
    assert_neuron_fired(&npu, 11, burst, dst);
    assert!(
        npu.get_outgoing_synapses(src.0).is_empty(),
        "No synapse should form until the full window is observed"
    );

    npu.inject_memory_neuron_to_fcl_with_kind(src.0, 10, 2.0, FIRE_KIND_STDP_ELIGIBLE);
    npu.inject_memory_neuron_to_fcl_with_kind(dst.0, 11, 2.0, FIRE_KIND_STDP_ELIGIBLE);
    let burst = npu.process_burst().unwrap().burst;
    assert_neuron_fired(&npu, 10, burst, src);
    assert_neuron_fired(&npu, 11, burst, dst);

    let outgoing = npu.get_outgoing_synapses(src.0);
    assert_eq!(
        outgoing.len(),
        1,
        "Synapse should be created after full window"
    );
    assert_eq!(outgoing[0].0, dst.0);
}

/// Default `inject_memory_neuron_to_fcl` uses episodic fire kind; those spikes still drive
/// associative STDP on the main fire ledger (synapse creation after full window).
#[test]
fn test_bidirectional_stdp_with_memory_neuron_ids_episodic_default_triggers_associative_stdp() {
    const MEMORY_NEURON_ID_START: u32 = 50_000_000;
    let (mut npu, _src_neurons, _dst_neurons) = create_stdp_network();

    npu.configure_fire_ledger_window(10, 2).unwrap();
    npu.configure_fire_ledger_window(11, 2).unwrap();

    let params = stdp_params(2, 1, 5, 0, true, 200.0, SynapseType::Excitatory);
    npu.register_stdp_mapping(10, 11, params).unwrap();

    let src = NeuronId(MEMORY_NEURON_ID_START);
    let dst = NeuronId(MEMORY_NEURON_ID_START + 1);

    npu.inject_memory_neuron_to_fcl(src.0, 10, 2.0);
    npu.inject_memory_neuron_to_fcl(dst.0, 11, 2.0);
    let burst = npu.process_burst().unwrap().burst;
    assert_neuron_fired(&npu, 10, burst, src);
    assert_neuron_fired(&npu, 11, burst, dst);
    assert!(
        npu.get_outgoing_synapses(src.0).is_empty(),
        "No synapse should form until the full window is observed"
    );

    npu.inject_memory_neuron_to_fcl(src.0, 10, 2.0);
    npu.inject_memory_neuron_to_fcl(dst.0, 11, 2.0);
    let burst = npu.process_burst().unwrap().burst;
    assert_neuron_fired(&npu, 10, burst, src);
    assert_neuron_fired(&npu, 11, burst, dst);

    let outgoing = npu.get_outgoing_synapses(src.0);
    assert_eq!(
        outgoing.len(),
        1,
        "Synapse should be created after full window"
    );
    assert_eq!(outgoing[0].0, dst.0);
}

/// When **both** (10→11) and (11→10) STDP mappings are registered, each direction is created by its
/// own mapping iteration only (no automatic reciprocal synapse from a single mapping).
#[test]
fn test_bidirectional_memory_neuron_both_mappings_no_duplicate_mirror() {
    const MEMORY_NEURON_ID_START: u32 = 50_000_000;
    let (mut npu, _src_neurons, _dst_neurons) = create_stdp_network();

    npu.configure_fire_ledger_window(10, 2).unwrap();
    npu.configure_fire_ledger_window(11, 2).unwrap();

    let params = stdp_params(2, 1, 5, 0, true, 200.0, SynapseType::Excitatory);
    npu.register_stdp_mapping(10, 11, params).unwrap();
    npu.register_stdp_mapping(11, 10, params).unwrap();

    let src = NeuronId(MEMORY_NEURON_ID_START);
    let dst = NeuronId(MEMORY_NEURON_ID_START + 1);

    npu.inject_memory_neuron_to_fcl_with_kind(src.0, 10, 2.0, FIRE_KIND_STDP_ELIGIBLE);
    npu.inject_memory_neuron_to_fcl_with_kind(dst.0, 11, 2.0, FIRE_KIND_STDP_ELIGIBLE);
    let _ = npu.process_burst().unwrap();

    npu.inject_memory_neuron_to_fcl_with_kind(src.0, 10, 2.0, FIRE_KIND_STDP_ELIGIBLE);
    npu.inject_memory_neuron_to_fcl_with_kind(dst.0, 11, 2.0, FIRE_KIND_STDP_ELIGIBLE);
    let _ = npu.process_burst().unwrap();

    let fwd = npu.get_outgoing_synapses(src.0);
    assert_eq!(fwd.len(), 1);
    assert_eq!(fwd[0].0, dst.0);

    let rev = npu.get_outgoing_synapses(dst.0);
    assert_eq!(
        rev.len(),
        1,
        "expected dst→src edge only when reverse STDP mapping is registered"
    );
    assert_eq!(rev[0].0, src.0);
}

/// Single registered associative mapping does **not** create a reverse synapse; register B→A separately.
#[test]
fn test_memory_only_single_mapping_has_no_automatic_reverse_edge() {
    const MEMORY_NEURON_ID_START: u32 = 50_000_000;
    let (mut npu, _src_neurons, _dst_neurons) = create_stdp_network();

    npu.configure_fire_ledger_window(10, 2).unwrap();
    npu.configure_fire_ledger_window(11, 2).unwrap();

    let params = stdp_params(2, 1, 5, 0, true, 200.0, SynapseType::Excitatory);
    npu.register_stdp_mapping(10, 11, params).unwrap();

    let src = NeuronId(MEMORY_NEURON_ID_START);
    let dst = NeuronId(MEMORY_NEURON_ID_START + 1);

    npu.inject_memory_neuron_to_fcl_with_kind(src.0, 10, 2.0, FIRE_KIND_STDP_ELIGIBLE);
    npu.inject_memory_neuron_to_fcl_with_kind(dst.0, 11, 2.0, FIRE_KIND_STDP_ELIGIBLE);
    let _ = npu.process_burst().unwrap();

    npu.inject_memory_neuron_to_fcl_with_kind(src.0, 10, 2.0, FIRE_KIND_STDP_ELIGIBLE);
    npu.inject_memory_neuron_to_fcl_with_kind(dst.0, 11, 2.0, FIRE_KIND_STDP_ELIGIBLE);
    let _ = npu.process_burst().unwrap();

    let fwd = npu.get_outgoing_synapses(src.0);
    assert_eq!(fwd.len(), 1);
    assert_eq!(fwd[0].0, dst.0);

    assert!(
        npu.get_outgoing_synapses(dst.0).is_empty(),
        "reverse edge must not be synthesized without a B→A STDP mapping"
    );
}

#[test]
fn test_memory_memory_associative_stdp_skipped_when_not_assoc_eligible() {
    const MEMORY_NEURON_ID_START: u32 = 50_000_000;
    let (mut npu, _src, _dst) = create_stdp_network();
    npu.set_memory_neuron_assoc_predicate(Some(Arc::new(|_| false)));

    npu.configure_fire_ledger_window(10, 2).unwrap();
    npu.configure_fire_ledger_window(11, 2).unwrap();

    let params = stdp_params(2, 1, 5, 0, true, 200.0, SynapseType::Excitatory);
    npu.register_stdp_mapping(10, 11, params).unwrap();

    let src = NeuronId(MEMORY_NEURON_ID_START);
    let dst = NeuronId(MEMORY_NEURON_ID_START + 1);

    npu.inject_memory_neuron_to_fcl_with_kind(src.0, 10, 2.0, FIRE_KIND_STDP_ELIGIBLE);
    npu.inject_memory_neuron_to_fcl_with_kind(dst.0, 11, 2.0, FIRE_KIND_STDP_ELIGIBLE);
    let _ = npu.process_burst().unwrap();
    npu.inject_memory_neuron_to_fcl_with_kind(src.0, 10, 2.0, FIRE_KIND_STDP_ELIGIBLE);
    npu.inject_memory_neuron_to_fcl_with_kind(dst.0, 11, 2.0, FIRE_KIND_STDP_ELIGIBLE);
    let _ = npu.process_burst().unwrap();

    assert!(npu.get_outgoing_synapses(src.0).is_empty());
    assert!(npu.get_outgoing_synapses(dst.0).is_empty());
}

#[test]
fn test_bidirectional_stdp_ltp_accumulates_on_sync() {
    let (mut npu, src_neurons, dst_neurons) = create_stdp_network();

    npu.configure_fire_ledger_window(10, 1).unwrap();
    npu.configure_fire_ledger_window(11, 1).unwrap();

    let params = stdp_params(1, 2, 3, 1, true, 128.0, SynapseType::Excitatory);
    npu.register_stdp_mapping(10, 11, params).unwrap();

    let src = src_neurons[0];
    let dst = dst_neurons[0];

    let burst = process_burst_with_injection(&mut npu, &[(src, 128.0), (dst, 128.0)]);
    assert_neuron_fired(&npu, 10, burst, src);
    assert_neuron_fired(&npu, 11, burst, dst);

    let outgoing = npu.get_outgoing_synapses(src.0);
    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing[0].0, dst.0);
    assert_eq!(outgoing[0].1, 6.0);

    let burst = process_burst_with_injection(&mut npu, &[(src, 128.0), (dst, 128.0)]);
    assert_neuron_fired(&npu, 10, burst, src);
    assert_neuron_fired(&npu, 11, burst, dst);

    let outgoing = npu.get_outgoing_synapses(src.0);
    assert_eq!(outgoing[0].1, 12.0);
}

#[test]
fn test_classic_plasticity_updates_existing_synapses_only() {
    let (mut npu, src_neurons, dst_neurons) = create_stdp_network();

    npu.configure_fire_ledger_window(10, 1).unwrap();
    npu.configure_fire_ledger_window(11, 1).unwrap();

    let params = stdp_params(1, 3, 2, 1, false, 100.0, SynapseType::Excitatory);
    npu.register_stdp_mapping(10, 11, params).unwrap();

    let src = src_neurons[0];
    let dst = dst_neurons[0];
    let no_pair_src = src_neurons[1];
    let no_pair_dst = dst_neurons[1];

    npu.add_synapse(
        src,
        dst,
        SynapticWeight(9.0),
        SynapticPsp(100.0),
        SynapseType::Excitatory,
        0,
        1,
    )
    .unwrap();
    npu.rebuild_synapse_index();

    let burst = process_burst_with_injection(
        &mut npu,
        &[(src, 128.0), (no_pair_src, 128.0), (no_pair_dst, 128.0)],
    );
    assert_neuron_fired(&npu, 10, burst, src);
    assert_neuron_fired(&npu, 10, burst, no_pair_src);
    assert_neuron_fired(&npu, 11, burst, no_pair_dst);

    let outgoing = npu.get_outgoing_synapses(src.0);
    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing[0].0, dst.0);
    assert_eq!(outgoing[0].1, 6.0);

    let no_pair_outgoing = npu.get_outgoing_synapses(no_pair_src.0);
    assert!(
        no_pair_outgoing.is_empty(),
        "Classic plasticity should not create new synapses"
    );
}

#[test]
fn test_ltd_reduces_to_zero_and_marks_prunable() {
    let (mut npu, src_neurons, dst_neurons) = create_stdp_network();

    npu.configure_fire_ledger_window(10, 1).unwrap();
    npu.configure_fire_ledger_window(11, 1).unwrap();

    let params = stdp_params(1, 2, 1, 2, false, 100.0, SynapseType::Excitatory);
    npu.register_stdp_mapping(10, 11, params).unwrap();

    let src = src_neurons[0];
    let dst = dst_neurons[0];

    npu.add_synapse(
        src,
        dst,
        SynapticWeight(1.0),
        SynapticPsp(100.0),
        SynapseType::Excitatory,
        0,
        1,
    )
    .unwrap();
    npu.rebuild_synapse_index();

    let burst = process_burst_with_injection(&mut npu, &[(src, 128.0)]);
    assert_neuron_fired(&npu, 10, burst, src);

    let outgoing = npu.get_outgoing_synapses(src.0);
    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing[0].0, dst.0);
    assert_eq!(outgoing[0].1, 0.0, "Weight=0 marks synapse as prunable");
}
