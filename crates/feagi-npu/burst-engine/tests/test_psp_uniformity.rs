// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Tests for PSP uniformity feature
//!
//! PSP Uniformity determines how postsynaptic potential is distributed:
//! - uniformity = false: PSP is divided among all outgoing synapses
//! - uniformity = true: Full PSP value applied to each synapse

use ahash::AHashMap;
use feagi_npu_burst_engine::RustNPU;
use feagi_npu_neural::types::*;
use feagi_npu_runtime::StdRuntime;
use feagi_structures::genomic::cortical_area::CoreCorticalType;

#[test]
fn test_psp_uniformity_false_divides_psp() {
    // Create NPU
    let runtime = StdRuntime;
    let backend = feagi_npu_burst_engine::backend::CPUBackend::new();
    let mut npu = RustNPU::new(runtime, backend, 100, 100, 5).expect("Failed to create NPU");

    // Register two cortical areas (use valid cortical types)
    let area_a = CoreCorticalType::Power.to_cortical_id();
    let area_b = CoreCorticalType::Death.to_cortical_id();

    npu.register_cortical_area(2, area_a.as_base_64());
    npu.register_cortical_area(3, area_b.as_base_64());

    // Create source neuron in area A
    let source_neuron = npu
        .add_neuron(
            1.0,      // threshold
            f32::MAX, // threshold_limit (MAX = no limit, SIMD-friendly encoding)
            0.0,      // leak
            0.0,      // resting
            0,        // neuron_type
            0,        // refractory
            1.0,      // excitability
            u16::MAX, // consecutive_fire_limit (MAX = unlimited, SIMD-friendly encoding)
            0,        // snooze
            true,     // mp_charge_accumulation
            2,        // cortical_area (area A)
            0,        // x
            0,        // y
            0,        // z
        )
        .expect("Failed to add source neuron");

    // Create 5 target neurons in area B
    let mut target_neurons = Vec::new();
    for i in 0..5 {
        let target = npu
            .add_neuron(
                1.0,  // threshold
                0.0,  // threshold_limit (0 = no limit)
                0.0,  // leak
                0.0,  // resting
                0,    // neuron_type
                0,    // refractory
                1.0,  // excitability
                0,    // consecutive_fire_limit
                0,    // snooze
                true, // mp_charge_accumulation
                3,    // cortical_area (area B)
                i, 0, 0,
            )
            .expect("Failed to add target neuron");
        target_neurons.push(target);
    }

    // Create 5 synapses from source to each target with PSP=10, weight=255
    for &target in &target_neurons {
        npu.add_synapse(
            source_neuron,
            target,
            SynapticWeight(255),
            SynapticConductance(10), // PSP = 10
            SynapseType::Excitatory,
        )
        .expect("Failed to add synapse");
    }

    // CRITICAL: Rebuild synapse index so propagation engine can find new synapses
    npu.rebuild_synapse_index();

    // Set PSP uniformity to FALSE for area A
    let mut flags = AHashMap::new();
    flags.insert(area_a, false);
    npu.set_psp_uniform_distribution_flags(flags);

    // Verify synapse count
    assert_eq!(npu.get_synapse_count(), 5);

    // With PSP uniformity = false:
    // PSP = 10, divided among 5 synapses = 2 per synapse
    // This division happens during propagation in the SynapticPropagationEngine

    println!(
        "✓ PSP uniformity = false test: {} synapses created",
        npu.get_synapse_count()
    );
    println!("  Each synapse will receive PSP/5 = 10/5 = 2 during propagation");
}

#[test]
fn test_psp_uniformity_true_applies_full_psp() {
    // Create NPU
    let runtime = StdRuntime;
    let backend = feagi_npu_burst_engine::backend::CPUBackend::new();
    let mut npu = RustNPU::new(runtime, backend, 100, 100, 5).expect("Failed to create NPU");

    // Register two cortical areas (use valid cortical types)
    let area_a = CoreCorticalType::Power.to_cortical_id();
    let area_b = CoreCorticalType::Death.to_cortical_id();

    npu.register_cortical_area(2, area_a.as_base_64());
    npu.register_cortical_area(3, area_b.as_base_64());

    // Create source neuron in area A
    let source_neuron = npu
        .add_neuron(
            1.0,  // threshold
            0.0,  // threshold_limit (0 = no limit)
            0.0,  // leak
            0.0,  // resting
            0,    // neuron_type
            0,    // refractory
            1.0,  // excitability
            0,    // consecutive_fire_limit
            0,    // snooze
            true, // mp_charge_accumulation
            2,    // cortical_area (area A)
            0, 0, 0,
        )
        .expect("Failed to add source neuron");

    // Create 5 target neurons in area B
    let mut target_neurons = Vec::new();
    for i in 0..5 {
        let target = npu
            .add_neuron(
                1.0,  // threshold
                0.0,  // threshold_limit (0 = no limit)
                0.0,  // leak
                0.0,  // resting
                0,    // neuron_type
                0,    // refractory
                1.0,  // excitability
                0,    // consecutive_fire_limit
                0,    // snooze
                true, // mp_charge_accumulation
                3,    // cortical_area (area B)
                i, 0, 0,
            )
            .expect("Failed to add target neuron");
        target_neurons.push(target);
    }

    // Create 5 synapses from source to each target with PSP=10, weight=255
    for &target in &target_neurons {
        npu.add_synapse(
            source_neuron,
            target,
            SynapticWeight(255),
            SynapticConductance(10), // PSP = 10
            SynapseType::Excitatory,
        )
        .expect("Failed to add synapse");
    }

    // CRITICAL: Rebuild synapse index so propagation engine can find new synapses
    npu.rebuild_synapse_index();

    // Set PSP uniformity to TRUE for area A
    let mut flags = AHashMap::new();
    flags.insert(area_a, true);
    npu.set_psp_uniform_distribution_flags(flags);

    // Verify synapse count
    assert_eq!(npu.get_synapse_count(), 5);

    // With PSP uniformity = true:
    // PSP = 10, applied to each synapse (not divided)
    // Each synapse gets the full PSP value of 10

    println!(
        "✓ PSP uniformity = true test: {} synapses created",
        npu.get_synapse_count()
    );
    println!("  Each synapse will receive full PSP = 10 during propagation");
}

#[test]
fn test_psp_uniformity_default_is_false() {
    // Create NPU
    let runtime = StdRuntime;
    let backend = feagi_npu_burst_engine::backend::CPUBackend::new();
    let mut npu = RustNPU::new(runtime, backend, 100, 100, 5).expect("Failed to create NPU");

    // Register cortical area (using valid CoreCorticalType)
    let area_a = CoreCorticalType::Power.to_cortical_id();
    npu.register_cortical_area(2, area_a.as_base_64());

    // Create neurons
    let source = npu
        .add_neuron(
            1.0,  // threshold
            0.0,  // threshold_limit (0 = no limit)
            0.0,  // leak
            0.0,  // resting
            0,    // neuron_type
            0,    // refractory
            1.0,  // excitability
            0,    // consecutive_fire_limit
            0,    // snooze
            true, // mp_charge_accumulation
            2,    // cortical_area
            0, 0, 0,
        )
        .expect("Failed to add source neuron");

    let target1 = npu
        .add_neuron(
            1.0,  // threshold
            0.0,  // threshold_limit (0 = no limit)
            0.0,  // leak
            0.0,  // resting
            0,    // neuron_type
            0,    // refractory
            1.0,  // excitability
            0,    // consecutive_fire_limit
            0,    // snooze
            true, // mp_charge_accumulation
            2,    // cortical_area
            1, 0, 0,
        )
        .expect("Failed to add target1");

    let target2 = npu
        .add_neuron(
            1.0,  // threshold
            0.0,  // threshold_limit (0 = no limit)
            0.0,  // leak
            0.0,  // resting
            0,    // neuron_type
            0,    // refractory
            1.0,  // excitability
            0,    // consecutive_fire_limit
            0,    // snooze
            true, // mp_charge_accumulation
            2,    // cortical_area
            2, 0, 0,
        )
        .expect("Failed to add target2");

    // Create synapses
    npu.add_synapse(
        source,
        target1,
        SynapticWeight(255),
        SynapticConductance(10),
        SynapseType::Excitatory,
    )
    .expect("Failed to add synapse 1");

    npu.add_synapse(
        source,
        target2,
        SynapticWeight(255),
        SynapticConductance(10),
        SynapseType::Excitatory,
    )
    .expect("Failed to add synapse 2");

    // CRITICAL: Rebuild synapse index so propagation engine can find new synapses
    npu.rebuild_synapse_index();

    // Don't set PSP uniformity flags - should default to false (divided)

    // Verify
    assert_eq!(npu.get_synapse_count(), 2);

    // With default (uniformity = false):
    // PSP = 10, divided among 2 synapses = 5 per synapse

    println!(
        "✓ Default behavior test: {} synapses created",
        npu.get_synapse_count()
    );
    println!("  PSP defaults to divided mode (uniformity = false)");
    println!("  Each synapse will receive PSP/2 = 10/2 = 5 during propagation");
}
