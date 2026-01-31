// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0
//! STDP integration tests for synaptic plasticity behavior.

use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::npu::StdpMappingParams;
use feagi_npu_burst_engine::RustNPU;
use feagi_npu_neural::types::{NeuronId, SynapticConductance, SynapticWeight};
use feagi_npu_neural::SynapseType;
use feagi_npu_runtime::StdRuntime;
use feagi_structures::genomic::cortical_area::CoreCorticalType;

/// Create a minimal STDP test network with two cortical areas.
fn create_stdp_network() -> (RustNPU<StdRuntime, f32, CPUBackend>, Vec<NeuronId>, Vec<NeuronId>) {
    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    let mut npu = RustNPU::new(runtime, backend, 100, 1000, 10).unwrap();

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
    ltp_multiplier: i64,
    ltd_multiplier: i64,
    bidirectional_stdp: bool,
    synapse_conductance: u8,
    synapse_type: SynapseType,
) -> StdpMappingParams {
    StdpMappingParams {
        plasticity_window,
        plasticity_constant,
        ltp_multiplier,
        ltd_multiplier,
        bidirectional_stdp,
        synapse_conductance,
        synapse_type,
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

    let params = stdp_params(2, 1, 5, 2, true, 200, SynapseType::Excitatory);
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
    assert_eq!(outgoing.len(), 1, "Synapse should be created after full window");
    let (target, weight, psp, synapse_type) = outgoing[0];
    assert_eq!(target, dst.0);
    assert_eq!(weight, 5);
    assert_eq!(psp, 200);
    assert_eq!(synapse_type, SynapseType::Excitatory as u8);
}

#[test]
fn test_bidirectional_stdp_ltp_accumulates_on_sync() {
    let (mut npu, src_neurons, dst_neurons) = create_stdp_network();

    npu.configure_fire_ledger_window(10, 1).unwrap();
    npu.configure_fire_ledger_window(11, 1).unwrap();

    let params = stdp_params(1, 2, 3, 1, true, 128, SynapseType::Excitatory);
    npu.register_stdp_mapping(10, 11, params).unwrap();

    let src = src_neurons[0];
    let dst = dst_neurons[0];

    let burst = process_burst_with_injection(&mut npu, &[(src, 128.0), (dst, 128.0)]);
    assert_neuron_fired(&npu, 10, burst, src);
    assert_neuron_fired(&npu, 11, burst, dst);

    let outgoing = npu.get_outgoing_synapses(src.0);
    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing[0].0, dst.0);
    assert_eq!(outgoing[0].1, 6);

    let burst = process_burst_with_injection(&mut npu, &[(src, 128.0), (dst, 128.0)]);
    assert_neuron_fired(&npu, 10, burst, src);
    assert_neuron_fired(&npu, 11, burst, dst);

    let outgoing = npu.get_outgoing_synapses(src.0);
    assert_eq!(outgoing[0].1, 12);
}

#[test]
fn test_classic_plasticity_updates_existing_synapses_only() {
    let (mut npu, src_neurons, dst_neurons) = create_stdp_network();

    npu.configure_fire_ledger_window(10, 1).unwrap();
    npu.configure_fire_ledger_window(11, 1).unwrap();

    let params = stdp_params(1, 3, 2, 1, false, 100, SynapseType::Excitatory);
    npu.register_stdp_mapping(10, 11, params).unwrap();

    let src = src_neurons[0];
    let dst = dst_neurons[0];
    let no_pair_src = src_neurons[1];
    let no_pair_dst = dst_neurons[1];

    npu.add_synapse(
        src,
        dst,
        SynapticWeight(9),
        SynapticConductance(100),
        SynapseType::Excitatory,
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
    assert_eq!(outgoing[0].1, 6);

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

    let params = stdp_params(1, 2, 1, 2, false, 100, SynapseType::Excitatory);
    npu.register_stdp_mapping(10, 11, params).unwrap();

    let src = src_neurons[0];
    let dst = dst_neurons[0];

    npu.add_synapse(
        src,
        dst,
        SynapticWeight(1),
        SynapticConductance(100),
        SynapseType::Excitatory,
    )
    .unwrap();
    npu.rebuild_synapse_index();

    let burst = process_burst_with_injection(&mut npu, &[(src, 128.0)]);
    assert_neuron_fired(&npu, 10, burst, src);

    let outgoing = npu.get_outgoing_synapses(src.0);
    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing[0].0, dst.0);
    assert_eq!(outgoing[0].1, 0, "Weight=0 marks synapse as prunable");
}
