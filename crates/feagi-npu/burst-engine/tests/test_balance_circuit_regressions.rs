// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0
//! Regressions found while building a reward-driven balance circuit through the API.
//!
//! Network: area 10 (memory source), area 11 (custom destination), area 12 (reward driver).

use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::fire_structures::FIRE_KIND_STDP_ELIGIBLE;
use feagi_npu_burst_engine::modulator_engine::ModulatorBinding;
use feagi_npu_burst_engine::npu::{PlasticityMode, StdpMappingParams};
use feagi_npu_burst_engine::RustNPU;
use feagi_npu_neural::types::{NeuronId, SynapticPsp, SynapticWeight};
use feagi_npu_neural::SynapseType;
use feagi_npu_runtime::StdRuntime;
use feagi_structures::genomic::cortical_area::{CoreCorticalType, CorticalID};
use feagi_structures::genomic::ModulatorKind;

const MEMORY_NEURON_ID_START: u32 = 50_000_000;

fn npu_with_areas() -> RustNPU<StdRuntime, f32, CPUBackend> {
    let mut npu = RustNPU::new(StdRuntime, CPUBackend::new(), 100, 1000, 10).unwrap();
    npu.register_cortical_area(0, CoreCorticalType::Death.to_cortical_id().as_base_64());
    npu.register_cortical_area(1, CoreCorticalType::Power.to_cortical_id().as_base_64());
    for (idx, id) in [(10u32, b"mmem0001"), (11, b"cdst0001"), (12, b"crew0001")] {
        npu.register_cortical_area(idx, CorticalID::try_from_bytes(id).unwrap().as_base_64());
    }
    npu
}

fn add_lif(npu: &mut RustNPU<StdRuntime, f32, CPUBackend>, area: u32, x: u32) -> NeuronId {
    npu.add_neuron(
        1.0,
        f32::MAX,
        0.0,
        0.0,
        0,
        0,
        1.0,
        0,
        0,
        true,
        area,
        x,
        0,
        0,
    )
    .unwrap()
}

fn stdp_params(mode: PlasticityMode, group: u16) -> StdpMappingParams {
    StdpMappingParams {
        plasticity_window: 1,
        plasticity_constant: 1,
        ltp_multiplier: 1,
        ltd_multiplier: 1,
        bidirectional_stdp: false,
        associative_memory_mapping: false,
        synapse_psp: 10.0,
        synapse_type: SynapseType::Excitatory,
        plasticity_mode: mode,
        eligibility_decay_bursts: 0,
        reward_source_area: None,
        punishment_source_area: None,
        max_weight: 3.0,
        plasticity_eta: 1.0,
        instance_reward: false,
        modulation_group: group,
    }
}

/// Item 1: a memory area mapped to a non-memory area with `associative_memory` in plain
/// STDP mode (no reward modulators, the API default) must grow synapses when the memory
/// neuron and the destination neuron co-fire.
#[test]
fn memory_to_custom_associative_stdp_creates_synapses() {
    let mut npu = npu_with_areas();
    let dst = add_lif(&mut npu, 11, 0);
    npu.configure_fire_ledger_window(10, 1).unwrap();
    npu.configure_fire_ledger_window(11, 1).unwrap();
    let mut params = stdp_params(PlasticityMode::Stdp, 0);
    params.associative_memory_mapping = true;
    npu.register_stdp_mapping(10, 11, params).unwrap();

    let mem = NeuronId(MEMORY_NEURON_ID_START + 1);
    npu.register_dynamic_neuron_mapping(mem.0, CorticalID::try_from_bytes(b"mmem0001").unwrap());
    for _ in 0..3 {
        npu.inject_memory_neuron_to_fcl_with_kind(mem.0, 10, 2.0, FIRE_KIND_STDP_ELIGIBLE);
        npu.inject_sensory_with_potentials(&[(dst, 128.0)]);
        npu.process_burst().unwrap();
    }
    let out = npu.get_outgoing_synapses(mem.0);
    assert_eq!(
        out.len(),
        1,
        "memory -> custom associative mapping must create one synapse"
    );
    assert_eq!(out[0].0, dst.0);
}

/// Item 3: a plastic mapping that subscribes to `synaptic.reward` must not potentiate
/// while the reward driver is silent, whatever `plasticity_mode` says.
#[test]
fn reward_subscribed_stdp_mapping_does_not_potentiate_without_reward() {
    let mut npu = npu_with_areas();
    let src = add_lif(&mut npu, 10, 0);
    let dst = add_lif(&mut npu, 11, 0);
    let _reward_driver = add_lif(&mut npu, 12, 0);
    npu.configure_fire_ledger_window(10, 1).unwrap();
    npu.configure_fire_ledger_window(11, 1).unwrap();
    npu.set_modulator_bindings(vec![ModulatorBinding {
        instance_id: "pleasure".to_string(),
        kind: ModulatorKind::Reward,
        magnitude_percent: 100.0,
        graded: false,
        full_scale_potential: 1.0,
        driver_cortical_idx: 12,
    }]);
    let group = npu.allocate_modulation_group(&["pleasure".to_string()]);
    assert!(group > 0);
    npu.add_synapse(
        src,
        dst,
        SynapticWeight(1.0),
        SynapticPsp(0.0),
        SynapseType::Excitatory,
        0,
        1,
    )
    .unwrap();
    npu.rebuild_synapse_index();
    npu.register_stdp_mapping(10, 11, stdp_params(PlasticityMode::Stdp, group))
        .unwrap();

    for _ in 0..5 {
        npu.inject_sensory_with_potentials(&[(src, 128.0), (dst, 128.0)]);
        npu.process_burst().unwrap();
    }
    let weight = npu.get_outgoing_synapses(src.0)[0].1;
    assert_eq!(
        weight, 1.0,
        "no reward fired, so the weight must stay at 1.0"
    );
}

/// Item 5: `neuro.firing_probability` at -70% on a subscribed area must block roughly
/// 70% of the firings while the driver fires.
#[test]
fn firing_probability_modulator_blocks_firing() {
    let mut npu = npu_with_areas();
    let neurons: Vec<NeuronId> = (0..20).map(|x| add_lif(&mut npu, 11, x)).collect();
    let driver = add_lif(&mut npu, 12, 0);
    npu.set_modulator_bindings(vec![ModulatorBinding {
        instance_id: "damp".to_string(),
        kind: ModulatorKind::FiringProbability,
        magnitude_percent: -70.0,
        graded: false,
        full_scale_potential: 1.0,
        driver_cortical_idx: 12,
    }]);
    let mut subs = ahash::AHashMap::new();
    subs.insert(11u32, vec!["damp".to_string()]);
    npu.set_modulator_area_subscriptions(subs);

    let mut fired_total = 0usize;
    for burst in 0..40 {
        let mut inj: Vec<(NeuronId, f32)> = neurons.iter().map(|n| (*n, 128.0)).collect();
        inj.push((driver, 128.0));
        npu.inject_sensory_with_potentials(&inj);
        let result = npu.process_burst().unwrap();
        if burst >= 2 {
            fired_total += result
                .fired_neurons
                .iter()
                .filter(|n| neurons.contains(n))
                .count();
        }
    }
    let rate = fired_total as f32 / (38.0 * 20.0);
    assert!(
        (0.15..=0.45).contains(&rate),
        "expected ~30% firing under -70% firing_probability, got {rate}"
    );
}

/// Item 7: a `[1,1,1]` area wired to itself with delay 1 and no refractory period must fire
/// on every burst once kicked, not every other burst.
#[test]
fn self_mapping_with_delay_one_fires_every_burst() {
    let mut npu = npu_with_areas();
    let n = npu
        .add_neuron(0.1, f32::MAX, 0.0, 0.0, 0, 0, 1.0, 0, 0, true, 11, 0, 0, 0)
        .unwrap();
    npu.add_synapse(
        n,
        n,
        SynapticWeight(1.0),
        SynapticPsp(30.0),
        SynapseType::Excitatory,
        0,
        1,
    )
    .unwrap();
    npu.rebuild_synapse_index();

    npu.inject_sensory_with_potentials(&[(n, 1.0)]);
    let mut pattern = Vec::new();
    for _ in 0..8 {
        let result = npu.process_burst().unwrap();
        pattern.push(result.fired_neurons.contains(&n));
    }
    assert!(
        pattern.iter().all(|fired| *fired),
        "self-mapped neuron should fire every burst, got {pattern:?}"
    );
}

/// Item 7 variant: same self-loop with `mp_charge_accumulation = false` (the CUSTOM default).
#[test]
fn self_mapping_without_mp_accumulation_fires_every_burst() {
    let mut npu = npu_with_areas();
    let n = npu
        .add_neuron(0.1, f32::MAX, 0.0, 0.0, 0, 0, 1.0, 0, 0, false, 11, 0, 0, 0)
        .unwrap();
    npu.add_synapse(
        n,
        n,
        SynapticWeight(1.0),
        SynapticPsp(30.0),
        SynapseType::Excitatory,
        0,
        1,
    )
    .unwrap();
    npu.rebuild_synapse_index();
    npu.inject_sensory_with_potentials(&[(n, 1.0)]);
    let mut pattern = Vec::new();
    for _ in 0..8 {
        let result = npu.process_burst().unwrap();
        pattern.push(result.fired_neurons.contains(&n));
    }
    assert!(pattern.iter().all(|f| *f), "got {pattern:?}");
}

/// Item 1 variant: memory neurons created by pattern detection fire with the episodic kind.
#[test]
fn memory_to_custom_associative_stdp_with_episodic_fires_creates_synapses() {
    use feagi_npu_burst_engine::fire_structures::FIRE_KIND_EPISODIC_MEMORY;
    let mut npu = npu_with_areas();
    let dst = add_lif(&mut npu, 11, 0);
    npu.configure_fire_ledger_window(10, 1).unwrap();
    npu.configure_fire_ledger_window(11, 1).unwrap();
    let mut params = stdp_params(PlasticityMode::Stdp, 0);
    params.associative_memory_mapping = true;
    npu.register_stdp_mapping(10, 11, params).unwrap();
    let mem = NeuronId(MEMORY_NEURON_ID_START + 1);
    npu.register_dynamic_neuron_mapping(mem.0, CorticalID::try_from_bytes(b"mmem0001").unwrap());
    for _ in 0..3 {
        npu.inject_memory_neuron_to_fcl_with_kind(mem.0, 10, 2.0, FIRE_KIND_EPISODIC_MEMORY);
        npu.inject_sensory_with_potentials(&[(dst, 128.0)]);
        npu.process_burst().unwrap();
    }
    let out = npu.get_outgoing_synapses(mem.0);
    assert_eq!(
        out.len(),
        1,
        "episodic-kind memory fires must still drive associative synthesis"
    );
}
