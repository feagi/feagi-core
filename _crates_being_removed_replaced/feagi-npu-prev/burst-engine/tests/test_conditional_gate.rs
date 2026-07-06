// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0
//! Conditional gate (transistor synapse) integration tests.
//!
//! These tests verify that synapses belonging to a gated mapping produce zero
//! contribution when the gate cortical_area area has no firing activity, and normal
//! contribution when the gate area fires.
//!
//! Network topology:
//!   area 10: source (1 LIF neuron, threshold=1)
//!   area 11: destination (1 LIF neuron, threshold=100)
//!   area 12: gate area (1 LIF neuron, threshold=1)
//!
//! Synapse from src -> dst has weight=10, PSP=50, yielding contribution=500
//! which exceeds dst threshold=100 when the gate is open.
//! Mapping 10 -> 11 is conditionally gated by area 12.

use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::RustNPU;
use feagi_npu_neural::types::{NeuronId, SynapticPsp, SynapticWeight};
use feagi_npu_neural::SynapseType;
use feagi_npu_runtime::StdRuntime;
use feagi_structures::genomic::cortical_area::CoreCorticalType;

type GateTestNetwork = (
    RustNPU<StdRuntime, f32, CPUBackend>,
    NeuronId, // src
    NeuronId, // dst
    NeuronId, // gate
);

/// Build a minimal network with src(10), dst(11), gate(12) areas.
fn create_gate_network() -> GateTestNetwork {
    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    let mut npu = RustNPU::new(runtime, backend, 100, 1000, 10).unwrap();

    npu.register_cortical_area(0, CoreCorticalType::Death.to_cortical_id().as_base_64());
    npu.register_cortical_area(1, CoreCorticalType::Power.to_cortical_id().as_base_64());

    npu.register_cortical_area(10, CoreCorticalType::Death.to_cortical_id().as_base_64());
    npu.register_cortical_area(11, CoreCorticalType::Death.to_cortical_id().as_base_64());
    npu.register_cortical_area(12, CoreCorticalType::Death.to_cortical_id().as_base_64());

    let src = npu
        .add_neuron(1.0, f32::MAX, 0.0, 0.0, 0, 0, 1.0, 0, 0, true, 10, 0, 0, 0)
        .unwrap();
    let dst = npu
        .add_neuron(
            100.0,
            f32::MAX,
            0.0,
            0.0,
            0,
            0,
            1.0,
            0,
            0,
            true,
            11,
            0,
            0,
            0,
        )
        .unwrap();
    let gate = npu
        .add_neuron(1.0, f32::MAX, 0.0, 0.0, 0, 0, 1.0, 0, 0, true, 12, 0, 0, 0)
        .unwrap();

    // Wire src -> dst with contribution = weight * psp = 10 * 50 = 500 (exceeds dst threshold of 100)
    npu.add_synapse(
        src,
        dst,
        SynapticWeight(10.0),
        SynapticPsp(50.0),
        SynapseType::Excitatory,
        0,
        1, // delay_bursts = 1
    )
    .unwrap();
    npu.rebuild_synapse_index();

    (npu, src, dst, gate)
}

/// When the gate area has no firing, the gated synapse must contribute zero PSP
/// to the destination neuron (transistor OFF). The destination must NOT fire.
#[test]
fn test_gate_closed_blocks_propagation() {
    let (mut npu, src, dst, _gate) = create_gate_network();
    npu.register_gate_mapping(10, 11, 12).unwrap();

    // Burst 1: fire source only (gate silent). Synapse fires, PSP is scheduled with delay=1.
    npu.inject_sensory_with_potentials(&[(src, 128.0)]);
    let result1 = npu.process_burst().unwrap();
    assert!(result1.fired_neurons.contains(&src), "Source must fire");

    // Burst 2: delayed contribution would arrive, but gate was closed during burst 1
    // so contribution was zero. Dst must NOT fire.
    let result2 = npu.process_burst().unwrap();
    assert!(
        !result2.fired_neurons.contains(&dst),
        "Gate closed: dst must NOT fire, but it did"
    );
}

/// When the gate area fires in the same burst as the source, the synapse propagates
/// normally (transistor ON). The destination MUST fire.
#[test]
fn test_gate_open_allows_propagation() {
    let (mut npu, src, dst, gate) = create_gate_network();
    npu.register_gate_mapping(10, 11, 12).unwrap();

    // Burst 1: fire source AND gate simultaneously.
    npu.inject_sensory_with_potentials(&[(src, 128.0), (gate, 128.0)]);
    let result1 = npu.process_burst().unwrap();
    assert!(result1.fired_neurons.contains(&src));
    assert!(result1.fired_neurons.contains(&gate));

    // Burst 2: delayed contribution arrives. Gate was open during burst 1 so PSP propagates.
    let result2 = npu.process_burst().unwrap();
    assert!(
        result2.fired_neurons.contains(&dst),
        "Gate open: dst must fire from propagated PSP"
    );
}

/// Without gate registration, synapses propagate unconditionally.
#[test]
fn test_no_gate_propagates_unconditionally() {
    let (mut npu, src, dst, _gate) = create_gate_network();
    // No gate registered.

    // Burst 1: fire source only.
    npu.inject_sensory_with_potentials(&[(src, 128.0)]);
    let result1 = npu.process_burst().unwrap();
    assert!(result1.fired_neurons.contains(&src));

    // Burst 2: delayed contribution arrives normally (no gate to block it).
    let result2 = npu.process_burst().unwrap();
    assert!(
        result2.fired_neurons.contains(&dst),
        "No gate: dst must fire from unconditional propagation"
    );
}

/// Gate state is per-burst: a gate that fired last burst but not the current one
/// still blocks propagation for the current burst's sources.
#[test]
fn test_gate_state_is_per_burst() {
    let (mut npu, src, dst, gate) = create_gate_network();
    npu.register_gate_mapping(10, 11, 12).unwrap();

    // Burst 1: gate fires alone (no source). Opens gate but nothing to propagate.
    npu.inject_sensory_with_potentials(&[(gate, 128.0)]);
    npu.process_burst().unwrap();

    // Burst 2: source fires but gate is silent. Gate must be closed THIS burst.
    npu.inject_sensory_with_potentials(&[(src, 128.0)]);
    npu.process_burst().unwrap();

    // Burst 3: delayed contribution from burst 2 would arrive. Should be zero.
    let result3 = npu.process_burst().unwrap();
    assert!(
        !result3.fired_neurons.contains(&dst),
        "Gate was open in prior burst but closed this burst: must block propagation"
    );
}

/// Unregistering the gate restores normal (unconditional) propagation.
#[test]
fn test_unregister_gate_restores_propagation() {
    let (mut npu, src, dst, _gate) = create_gate_network();

    npu.register_gate_mapping(10, 11, 12).unwrap();
    npu.unregister_gate_mapping(10, 11);

    // Fire source only (gate silent). Since gate is unregistered, should propagate.
    npu.inject_sensory_with_potentials(&[(src, 128.0)]);
    npu.process_burst().unwrap();

    let result2 = npu.process_burst().unwrap();
    assert!(
        result2.fired_neurons.contains(&dst),
        "Unregistered gate must allow unconditional propagation"
    );
}

/// Registering a gate on an invalid cortical_area area must return an error.
#[test]
fn test_register_gate_invalid_area_fails() {
    let (mut npu, _src, _dst, _gate) = create_gate_network();

    let err = npu.register_gate_mapping(10, 11, 999);
    assert!(
        err.is_err(),
        "Gate registration with invalid gate area must fail"
    );
}
