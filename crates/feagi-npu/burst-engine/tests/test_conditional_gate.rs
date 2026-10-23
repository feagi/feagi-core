// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0
//! `synaptic.gate` modulator integration tests.
//!
//! A mapping whose modulation group lists a `synaptic.gate` instance passes nothing
//! until the instance's driver area fires. The driver signal is committed at the end of
//! burst t and applies from burst t+1, like every other synaptic modulator.
//!
//! Network topology:
//!   area 10: source (1 LIF neuron, threshold=1)
//!   area 11: destination (1 LIF neuron, threshold=100)
//!   area 12: gate driver (1 LIF neuron, threshold=1)
//!
//! Synapse from src -> dst has weight=10, PSP=50, yielding contribution=500
//! which exceeds dst threshold=100 when the gate is open.

use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::modulator_engine::ModulatorBinding;
use feagi_npu_burst_engine::RustNPU;
use feagi_npu_neural::types::{NeuronId, SynapticPsp, SynapticWeight};
use feagi_npu_neural::SynapseType;
use feagi_npu_runtime::StdRuntime;
use feagi_structures::genomic::cortical_area::CoreCorticalType;
use feagi_structures::genomic::ModulatorKind;

type GateTestNetwork = (
    RustNPU<StdRuntime, f32, CPUBackend>,
    NeuronId, // src
    NeuronId, // dst
    NeuronId, // gate
);

/// Build a minimal network with src(10), dst(11), gate driver(12) areas.
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

/// Bind one gate instance driven by area 12 and stamp its group onto the src -> dst synapse.
fn gate_mapping(npu: &mut RustNPU<StdRuntime, f32, CPUBackend>, graded: bool) {
    npu.set_modulator_bindings(vec![ModulatorBinding {
        instance_id: "gate".to_string(),
        kind: ModulatorKind::Gate,
        magnitude_percent: 100.0,
        graded,
        full_scale_potential: 1.0,
        driver_cortical_idx: 12,
    }]);
    let group = npu.allocate_modulation_group(&["gate".to_string()]);
    assert!(group > 0);
    npu.stamp_synapse_modulation_group(0, npu.synapse_count(), group);
}

/// A silent driver keeps the gate closed: dst must NOT fire, from the very first burst.
#[test]
fn test_gate_closed_blocks_propagation() {
    let (mut npu, src, dst, _gate) = create_gate_network();
    gate_mapping(&mut npu, false);

    npu.inject_sensory_with_potentials(&[(src, 128.0)]);
    let result1 = npu.process_burst().unwrap();
    assert!(result1.fired_neurons.contains(&src), "Source must fire");

    let result2 = npu.process_burst().unwrap();
    assert!(
        !result2.fired_neurons.contains(&dst),
        "Gate closed: dst must NOT fire, but it did"
    );
}

/// The driver fires in burst 1 (signal committed), the source fires in burst 2 and
/// propagates through the open gate; dst fires in burst 3.
#[test]
fn test_gate_open_after_driver_fires() {
    let (mut npu, src, dst, gate) = create_gate_network();
    gate_mapping(&mut npu, false);

    npu.inject_sensory_with_potentials(&[(gate, 128.0)]);
    let result1 = npu.process_burst().unwrap();
    assert!(result1.fired_neurons.contains(&gate));

    npu.inject_sensory_with_potentials(&[(src, 128.0)]);
    let result2 = npu.process_burst().unwrap();
    assert!(result2.fired_neurons.contains(&src));

    let result3 = npu.process_burst().unwrap();
    assert!(
        result3.fired_neurons.contains(&dst),
        "Gate open: dst must fire from propagated PSP"
    );
}

/// The gate follows the driver one burst behind: a driver that fired two bursts ago
/// no longer holds the gate open.
#[test]
fn test_gate_closes_again_when_driver_goes_silent() {
    let (mut npu, src, dst, gate) = create_gate_network();
    gate_mapping(&mut npu, false);

    npu.inject_sensory_with_potentials(&[(gate, 128.0)]);
    npu.process_burst().unwrap();
    // Driver silent in burst 2: its closed signal applies from burst 3.
    npu.process_burst().unwrap();

    npu.inject_sensory_with_potentials(&[(src, 128.0)]);
    npu.process_burst().unwrap();
    let result4 = npu.process_burst().unwrap();
    assert!(
        !result4.fired_neurons.contains(&dst),
        "Driver silent a burst earlier: gate must be closed"
    );
}

/// Without a gate instance in the group, synapses propagate unconditionally.
#[test]
fn test_no_gate_propagates_unconditionally() {
    let (mut npu, src, dst, _gate) = create_gate_network();

    npu.inject_sensory_with_potentials(&[(src, 128.0)]);
    let result1 = npu.process_burst().unwrap();
    assert!(result1.fired_neurons.contains(&src));

    let result2 = npu.process_burst().unwrap();
    assert!(
        result2.fired_neurons.contains(&dst),
        "No gate: dst must fire from unconditional propagation"
    );
}
