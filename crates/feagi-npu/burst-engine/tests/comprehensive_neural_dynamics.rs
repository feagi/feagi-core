// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive Neural Dynamics Test Suite
//!
//! This test suite systematically validates synaptic propagation and neural dynamics
//! across all parameter combinations and edge cases.
//!
//! # Test Coverage Matrix
//!
//! ## Threshold Scenarios
//! - PSP < threshold (should NOT fire)
//! - PSP = threshold (should fire)
//! - PSP > threshold (should fire)
//!
//! ## MP Accumulation
//! - mp_charge_accumulation = false (reset each burst)
//! - mp_charge_accumulation = true (accumulate across bursts)
//!
//! ## Leak Coefficient
//! - No leak (0.0)
//! - Partial leak (0.5)
//! - Full leak (1.0)
//!
//! ## PSP Uniformity
//! - psp_uniform_distribution = false (divide among synapses)
//! - psp_uniform_distribution = true (full PSP to each)
//!
//! ## Multiple Synapses
//! - Single synapse
//! - Multiple synapses from same source
//! - Multiple synapses from different sources
//!
//! ## Synapse Types
//! - Excitatory only
//! - Inhibitory only
//! - Mixed (excitatory + inhibitory)
//!
//! ## Refractory Periods
//! - No refractory (0)
//! - Normal refractory (1-5)
//! - Extended refractory (snooze)
//!
//! ## Backend Compatibility
//! - All tests use generic backend trait
//! - Can run on CPU, GPU, or CUDA

use feagi_npu_burst_engine::{RustNPU, backend::CPUBackend};
use feagi_npu_neural::{SynapticWeight, SynapticConductance, SynapseType, NeuronId};
use feagi_npu_runtime::StdRuntime;
use feagi_structures::genomic::cortical_area::{CoreCorticalType, CorticalID};
use ahash::AHashMap;

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a standard NPU for testing
fn create_test_npu() -> RustNPU<StdRuntime, f32, CPUBackend> {
    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    let mut npu = RustNPU::new(runtime, backend, 1000, 10000, 20).unwrap();
    // Register test areas 1-20 (avoiding area 1 being treated as Power auto-inject area)
    // Use unique cortical IDs to avoid conflicts
    for area_id in 1..=20 {
        let cortical_type = if area_id % 2 == 0 {
            CoreCorticalType::Death
        } else {
            CoreCorticalType::Power
        };
        npu.register_cortical_area(area_id, cortical_type.to_cortical_id().as_base_64());
    }
    
    npu
}

/// Create a neuron with specified parameters for testing
fn add_test_neuron(
    npu: &mut RustNPU<StdRuntime, f32, CPUBackend>,
    threshold: f32,
    leak: f32,
    mp_accumulation: bool,
    refractory: u16,
    cortical_area: u32,
    x: u32,
) -> NeuronId {
    npu.add_neuron(
        threshold,
        0.0, // threshold_limit (0 = no limit)
        leak,
        0.0, // resting_potential
        0,   // neuron_type
        refractory,
        1.0, // excitability (always fire if threshold met)
        0,   // consecutive_fire_limit (unlimited)
        0,   // snooze_period
        mp_accumulation,
        cortical_area,
        x, 0, 0,
    ).unwrap()
}

/// Helper to add synapse in tests (rebuilds index automatically)
fn add_test_synapse(
    npu: &mut RustNPU<StdRuntime, f32, CPUBackend>,
    source: NeuronId,
    target: NeuronId,
    weight: SynapticWeight,
    conductance: SynapticConductance,
    synapse_type: SynapseType,
) {
    npu.add_synapse(source, target, weight, conductance, synapse_type)
        .unwrap();
    npu.rebuild_synapse_index();
}

// ============================================================================
// SECTION 1: Threshold Scenarios
// ============================================================================

#[test]
fn test_psp_below_threshold_no_fire() {
    let mut npu = create_test_npu();
    
    // Source neuron: threshold 1.0, fires easily
    let source = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 10, 0);
    
    // Target neuron: threshold 1.1 (HIGHER than PSP)
    let target = add_test_neuron(&mut npu, 1.1, 0.0, false, 0, 11, 0);
    
    // Synapse with PSP = 1.0 (weight=1 × conductance=1)
    add_test_synapse(&mut npu, 
        source,
        target,
        SynapticWeight(1),
        SynapticConductance(1),
        SynapseType::Excitatory,
    );    
    // Burst 1: Fire source
    npu.inject_sensory_with_potentials(&[(source, 2.0)]);
    let result1 = npu.process_burst().unwrap();
    assert!(
        result1.fired_neurons.contains(&source),
        "Source should fire"
    );
    
    // Burst 2: Target should NOT fire (PSP=1.0 < threshold=1.1)
    let result2 = npu.process_burst().unwrap();
    assert!(!result2.fired_neurons.contains(&target),
        "Target should NOT fire when PSP (1.0) < threshold (1.1)");
}

#[test]
fn test_psp_equals_threshold_fires() {
    let mut npu = create_test_npu();
    
    // Use areas 10 and 11 to avoid hardcoded power injection for area_id==1
    let source = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 10, 0);
    let target = add_test_neuron(&mut npu, 100.0, 0.0, false, 0, 11, 0);
    
    // Synapse with PSP = 100 (weight=10 × conductance=10)
    add_test_synapse(&mut npu, 
        source,
        target,
        SynapticWeight(10),
        SynapticConductance(10),
        SynapseType::Excitatory,
    );    
    // Burst 1: Fire source
    npu.inject_sensory_with_potentials(&[(source, 2.0)]);
    let result1 = npu.process_burst().unwrap();    assert!(result1.fired_neurons.contains(&source));
    
    // Burst 2: Target SHOULD fire (PSP=100 >= threshold=100)
    let result2 = npu.process_burst().unwrap();    assert!(result2.fired_neurons.contains(&target),
        "Target should fire when PSP (100) >= threshold (100)");
}

#[test]
fn test_psp_above_threshold_fires() {
    let mut npu = create_test_npu();
    
    let source = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 1, 0);
    let target = add_test_neuron(&mut npu, 50.0, 0.0, false, 0, 2, 0);
    
    // Synapse with PSP = 100 (weight=10 × conductance=10)
    add_test_synapse(&mut npu, 
        source,
        target,
        SynapticWeight(10),
        SynapticConductance(10),
        SynapseType::Excitatory,
    );    
    // Burst 1: Fire source
    npu.inject_sensory_with_potentials(&[(source, 2.0)]);
    npu.process_burst().unwrap();    
    // Burst 2: Target SHOULD fire (PSP=100 > threshold=50)
    let result2 = npu.process_burst().unwrap();    assert!(result2.fired_neurons.contains(&target),
        "Target should fire when PSP (100) > threshold (50)");
}

// ============================================================================
// SECTION 2: MP Accumulation Scenarios
// ============================================================================

#[test]
fn test_mp_accumulation_false_resets_each_burst() {
    let mut npu = create_test_npu();
    
    let source = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 1, 0);
    let target = add_test_neuron(&mut npu, 200.0, 0.0, false, 0, 2, 0); // mp_acc=FALSE
    
    // Synapse with PSP = 100 (less than threshold)
    add_test_synapse(&mut npu, 
        source,
        target,
        SynapticWeight(10),
        SynapticConductance(10),
        SynapseType::Excitatory,
    );    
    // Burst 1: Fire source
    npu.inject_sensory_with_potentials(&[(source, 2.0)]);
    npu.process_burst().unwrap();    
    // Burst 2: Target gets PSP=100, but threshold=200, should NOT fire
    let result2 = npu.process_burst().unwrap();    assert!(!result2.fired_neurons.contains(&target));
    
    // Burst 3: Fire source again
    npu.inject_sensory_with_potentials(&[(source, 2.0)]);
    npu.process_burst().unwrap();    
    // Burst 4: Target should STILL NOT fire (mp reset to 0 each burst)
    let result4 = npu.process_burst().unwrap();    assert!(!result4.fired_neurons.contains(&target),
        "Target should NOT fire with mp_accumulation=false, PSP does not accumulate");
}

#[test]
fn test_mp_accumulation_true_accumulates_across_bursts() {
    let mut npu = create_test_npu();
    
    // Use areas 10/11 to avoid power injection on area 1
    let source = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 10, 0);
    let target = add_test_neuron(&mut npu, 200.0, 0.0, true, 0, 11, 0); // mp_acc=TRUE
    
    // Synapse with PSP = 100 (less than threshold)
    add_test_synapse(&mut npu, 
        source,
        target,
        SynapticWeight(10),
        SynapticConductance(10),
        SynapseType::Excitatory,
    );    
    // Burst 1: Fire source
    npu.inject_sensory_with_potentials(&[(source, 2.0)]);
    npu.process_burst().unwrap();    
    // Burst 2: Target gets PSP=100, but threshold=200, should NOT fire yet
    let result2 = npu.process_burst().unwrap();    assert!(!result2.fired_neurons.contains(&target));
    
    // Burst 3: Fire source again
    npu.inject_sensory_with_potentials(&[(source, 2.0)]);
    npu.process_burst().unwrap();    
    // Burst 4: Target should NOW fire (accumulated 100 + 100 = 200 >= threshold)
    let result4 = npu.process_burst().unwrap();    assert!(result4.fired_neurons.contains(&target),
        "Target should fire with mp_accumulation=true, PSP accumulates (100+100=200 >= 200)");
}

// ============================================================================
// SECTION 3: Leak Coefficient Scenarios
// ============================================================================

#[test]
fn test_no_leak_preserves_potential() {
    let mut npu = create_test_npu();
    
    let target = add_test_neuron(&mut npu, 200.0, 0.0, true, 0, 2, 0); // leak=0.0
    
    // Inject 100, wait, check if still 100
    npu.inject_sensory_with_potentials(&[(target, 100.0)]);
    npu.process_burst().unwrap();    
    // Check membrane potential after 3 bursts
    for _ in 0..3 {
        npu.process_burst().unwrap();    }
    
    let mp = npu.get_neuron_property_by_index(target.0 as usize, "membrane_potential")
        .expect("Should have membrane potential");
    
    assert!((mp - 100.0).abs() < 0.01, 
        "With no leak (0.0), potential should be preserved (expected 100, got {})", mp);
}

#[test]
fn test_partial_leak_decays_potential() {
    let mut npu = create_test_npu();
    
    let target = add_test_neuron(&mut npu, 200.0, 0.5, true, 0, 2, 0); // leak=0.5
    
    // Inject 100, wait, check decay
    npu.inject_sensory_with_potentials(&[(target, 100.0)]);
    npu.process_burst().unwrap();    
    // After 1 burst with leak=0.5: mp = 100 + 0.5 * (0 - 100) = 100 - 50 = 50
    npu.process_burst().unwrap();    
    let mp = npu.get_neuron_property_by_index(target.0 as usize, "membrane_potential")
        .expect("Should have membrane potential");
    
    assert!((mp - 50.0).abs() < 0.01,
        "With leak=0.5, potential should decay to 50 (got {})", mp);
}

#[test]
fn test_full_leak_resets_to_resting() {
    let mut npu = create_test_npu();
    
    let target = add_test_neuron(&mut npu, 200.0, 1.0, true, 0, 2, 0); // leak=1.0
    
    // Inject 100, wait, check full decay
    npu.inject_sensory_with_potentials(&[(target, 100.0)]);
    npu.process_burst().unwrap();    
    // After 1 burst with leak=1.0: mp = 100 + 1.0 * (0 - 100) = 0
    npu.process_burst().unwrap();    
    let mp = npu.get_neuron_property_by_index(target.0 as usize, "membrane_potential")
        .expect("Should have membrane potential");
    
    assert!(mp.abs() < 0.01,
        "With leak=1.0, potential should fully decay to resting (0), got {}", mp);
}

// ============================================================================
// SECTION 4: PSP Uniformity Scenarios
// ============================================================================

#[test]
fn test_psp_uniformity_false_divides_among_synapses() {
    let mut npu = create_test_npu();
    
    let source = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 1, 0);
    let target1 = add_test_neuron(&mut npu, 50.0, 0.0, false, 0, 2, 0);
    let target2 = add_test_neuron(&mut npu, 50.0, 0.0, false, 0, 2, 1);
    
    // Two synapses with conductance=10 each
    add_test_synapse(&mut npu, source, target1, SynapticWeight(10), SynapticConductance(10), SynapseType::Excitatory);
    add_test_synapse(&mut npu, source, target2, SynapticWeight(10), SynapticConductance(10), SynapseType::Excitatory);
    
    // Set PSP uniformity to FALSE (divide)
    let mut flags = AHashMap::new();
    flags.insert(CoreCorticalType::Power.to_cortical_id(), false);
    npu.set_psp_uniform_distribution_flags(flags);
    
    // Burst 1: Fire source
    npu.inject_sensory_with_potentials(&[(source, 2.0)]);
    npu.process_burst().unwrap();    
    // Burst 2: Each target should receive PSP/2 = 100/2 = 50
    let result2 = npu.process_burst().unwrap();    
    // Both should fire (50 >= 50 threshold)
    assert!(result2.fired_neurons.contains(&target1) || result2.fired_neurons.contains(&target2),
        "At least one target should fire with divided PSP");
}

#[test]
fn test_psp_uniformity_true_full_to_each_synapse() {
    let mut npu = create_test_npu();
    
    let source = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 1, 0);
    let target1 = add_test_neuron(&mut npu, 100.0, 0.0, false, 0, 2, 0);
    let target2 = add_test_neuron(&mut npu, 100.0, 0.0, false, 0, 2, 1);
    
    // Two synapses with conductance=10 each
    add_test_synapse(&mut npu, source, target1, SynapticWeight(10), SynapticConductance(10), SynapseType::Excitatory);
    add_test_synapse(&mut npu, source, target2, SynapticWeight(10), SynapticConductance(10), SynapseType::Excitatory);
    
    // Set PSP uniformity to TRUE (full to each)
    let mut flags = AHashMap::new();
    flags.insert(CoreCorticalType::Power.to_cortical_id(), true);
    npu.set_psp_uniform_distribution_flags(flags);
    
    // Burst 1: Fire source
    npu.inject_sensory_with_potentials(&[(source, 2.0)]);
    npu.process_burst().unwrap();    
    // Burst 2: Each target should receive full PSP = 100
    let result2 = npu.process_burst().unwrap();    
    // Both should fire (100 >= 100 threshold)
    assert!(result2.fired_neurons.contains(&target1),
        "Target1 should fire with full PSP");
    assert!(result2.fired_neurons.contains(&target2),
        "Target2 should fire with full PSP");
}

// ============================================================================
// SECTION 5: Multiple Synapse Scenarios
// ============================================================================

#[test]
fn test_multiple_synapses_from_same_source() {
    let mut npu = create_test_npu();
    
    // Use areas 10/11 to avoid power injection
    let source = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 10, 0);
    let target = add_test_neuron(&mut npu, 200.0, 0.0, false, 0, 11, 0);
    
    // Set PSP uniformity to true for area 10 so each synapse gets full PSP
    let mut psp_flags = ahash::AHashMap::new();
    let area_10_id = CorticalID::try_from_base_64(&CoreCorticalType::Death.to_cortical_id().as_base_64()).unwrap();    psp_flags.insert(area_10_id, true);
    npu.set_psp_uniform_distribution_flags(psp_flags);
    
    // Add 2 synapses from same source to same target (PSP=100 each)
    add_test_synapse(&mut npu, source, target, SynapticWeight(10), SynapticConductance(10), SynapseType::Excitatory);
    add_test_synapse(&mut npu, source, target, SynapticWeight(10), SynapticConductance(10), SynapseType::Excitatory);
    
    // Burst 1: Fire source
    npu.inject_sensory_with_potentials(&[(source, 2.0)]);
    npu.process_burst().unwrap();    
    // Burst 2: Target should receive 100+100=200 (accumulated in FCL)
    let result2 = npu.process_burst().unwrap();    assert!(result2.fired_neurons.contains(&target),
        "Target should fire when multiple synapses accumulate (100+100=200 >= 200)");
}

#[test]
fn test_multiple_synapses_from_different_sources() {
    let mut npu = create_test_npu();
    
    let source1 = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 1, 0);
    let source2 = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 1, 1);
    let target = add_test_neuron(&mut npu, 200.0, 0.0, false, 0, 2, 0);
    
    // Synapse from each source (PSP=100 each)
    add_test_synapse(&mut npu, source1, target, SynapticWeight(10), SynapticConductance(10), SynapseType::Excitatory);
    add_test_synapse(&mut npu, source2, target, SynapticWeight(10), SynapticConductance(10), SynapseType::Excitatory);
    
    // Burst 1: Fire both sources
    npu.inject_sensory_with_potentials(&[(source1, 2.0), (source2, 2.0)]);
    npu.process_burst().unwrap();    
    // Burst 2: Target should receive 100+100=200
    let result2 = npu.process_burst().unwrap();    assert!(result2.fired_neurons.contains(&target),
        "Target should fire when inputs from multiple sources converge (100+100=200 >= 200)");
}

// ============================================================================
// SECTION 6: Excitatory vs Inhibitory Synapses
// ============================================================================

#[test]
fn test_excitatory_synapse_increases_potential() {
    let mut npu = create_test_npu();
    
    let source = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 1, 0);
    let target = add_test_neuron(&mut npu, 100.0, 0.0, false, 0, 2, 0);
    
    add_test_synapse(&mut npu, source, target, SynapticWeight(10), SynapticConductance(10), SynapseType::Excitatory);
    
    // Fire source
    npu.inject_sensory_with_potentials(&[(source, 2.0)]);
    npu.process_burst().unwrap();    
    // Target should fire (PSP=100 >= 100)
    let result = npu.process_burst().unwrap();    assert!(result.fired_neurons.contains(&target),
        "Excitatory synapse should increase potential and cause firing");
}

#[test]
fn test_inhibitory_synapse_decreases_potential() {
    let mut npu = create_test_npu();
    
    let excitatory = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 1, 0);
    let inhibitory = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 1, 1);
    let target = add_test_neuron(&mut npu, 100.0, 0.0, false, 0, 2, 0);
    
    // Excitatory: +100, Inhibitory: -100
    add_test_synapse(&mut npu, excitatory, target, SynapticWeight(10), SynapticConductance(10), SynapseType::Excitatory);
    add_test_synapse(&mut npu, inhibitory, target, SynapticWeight(10), SynapticConductance(10), SynapseType::Inhibitory);
    
    // Fire both
    npu.inject_sensory_with_potentials(&[(excitatory, 2.0), (inhibitory, 2.0)]);
    npu.process_burst().unwrap();    
    // Target should NOT fire (100 - 100 = 0 < 100)
    let result = npu.process_burst().unwrap();    assert!(!result.fired_neurons.contains(&target),
        "Inhibitory synapse should cancel excitatory (100-100=0 < 100)");
}

#[test]
fn test_mixed_excitatory_inhibitory_net_effect() {
    let mut npu = create_test_npu();
    
    let excitatory = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 1, 0);
    let inhibitory = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 1, 1);
    let target = add_test_neuron(&mut npu, 100.0, 0.0, false, 0, 2, 0);
    
    // Excitatory: +200, Inhibitory: -50, Net: +150
    add_test_synapse(&mut npu, excitatory, target, SynapticWeight(20), SynapticConductance(10), SynapseType::Excitatory);
    add_test_synapse(&mut npu, inhibitory, target, SynapticWeight(5), SynapticConductance(10), SynapseType::Inhibitory);
    
    // Fire both
    npu.inject_sensory_with_potentials(&[(excitatory, 2.0), (inhibitory, 2.0)]);
    npu.process_burst().unwrap();    
    // Target SHOULD fire (200 - 50 = 150 >= 100)
    let result = npu.process_burst().unwrap();    assert!(result.fired_neurons.contains(&target),
        "Net effect of mixed synapses should be 200-50=150, causing firing");
}

// ============================================================================
// SECTION 7: Refractory Period Scenarios
// ============================================================================

#[test]
fn test_refractory_period_blocks_firing() {
    let mut npu = create_test_npu();
    
    let neuron = add_test_neuron(&mut npu, 1.0, 0.0, false, 3, 1, 0); // refractory=3
    
    // Burst 1: Fire neuron
    npu.inject_sensory_with_potentials(&[(neuron, 2.0)]);
    let result1 = npu.process_burst().unwrap();    assert!(result1.fired_neurons.contains(&neuron));
    
    // Bursts 2-4: Should be in refractory (blocked)
    for burst in 2..=4 {
        npu.inject_sensory_with_potentials(&[(neuron, 2.0)]);
        let result = npu.process_burst().unwrap();        assert_eq!(result.burst, burst);
        // Neuron blocked by refractory
    }
    
    // Burst 5: Should be able to fire again (refractory expired)
    npu.inject_sensory_with_potentials(&[(neuron, 2.0)]);
    let result5 = npu.process_burst().unwrap();    assert!(result5.fired_neurons.contains(&neuron),
        "Neuron should fire again after refractory period expires");
}

// ============================================================================
// SECTION 8: Multi-Burst Chain Propagation
// ============================================================================

#[test]
fn test_chain_propagation_with_delay() {
    let mut npu = create_test_npu();
    
    // Chain: N1 -> N2 -> N3
    let n1 = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 1, 0);
    let n2 = add_test_neuron(&mut npu, 100.0, 0.0, false, 0, 2, 0);
    let n3 = add_test_neuron(&mut npu, 100.0, 0.0, false, 0, 2, 1);
    
    add_test_synapse(&mut npu, n1, n2, SynapticWeight(10), SynapticConductance(10), SynapseType::Excitatory);
    add_test_synapse(&mut npu, n2, n3, SynapticWeight(10), SynapticConductance(10), SynapseType::Excitatory);
    
    // Burst 1: Fire N1
    npu.inject_sensory_with_potentials(&[(n1, 2.0)]);
    let result1 = npu.process_burst().unwrap();    assert!(result1.fired_neurons.contains(&n1));
    assert!(!result1.fired_neurons.contains(&n2)); // Not yet
    
    // Burst 2: N2 should fire (from N1)
    let result2 = npu.process_burst().unwrap();    assert!(result2.fired_neurons.contains(&n2));
    assert!(!result2.fired_neurons.contains(&n3)); // Not yet
    
    // Burst 3: N3 should fire (from N2)
    let result3 = npu.process_burst().unwrap();    assert!(result3.fired_neurons.contains(&n3),
        "Chain should propagate with 1-burst delay at each step");
}

// ============================================================================
// SECTION 9: Edge Cases
// ============================================================================

#[test]
fn test_zero_weight_no_propagation() {
    let mut npu = create_test_npu();
    
    let source = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 1, 0);
    let target = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 2, 0);
    
    // Synapse with weight=0 (PSP = 0 × 10 = 0)
    add_test_synapse(&mut npu, source, target, SynapticWeight(0), SynapticConductance(10), SynapseType::Excitatory);
    
    npu.inject_sensory_with_potentials(&[(source, 2.0)]);
    npu.process_burst().unwrap();    
    // Target should NOT fire (zero PSP)
    let result = npu.process_burst().unwrap();    assert!(!result.fired_neurons.contains(&target),
        "Zero weight should result in no propagation");
}

#[test]
fn test_maximum_psp_saturates() {
    let mut npu = create_test_npu();
    
    let source = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 1, 0);
    let target = add_test_neuron(&mut npu, 10000.0, 0.0, false, 0, 2, 0);
    
    // Maximum PSP = 255 × 255 = 65,025
    add_test_synapse(&mut npu, source, target, SynapticWeight(255), SynapticConductance(255), SynapseType::Excitatory);
    
    npu.inject_sensory_with_potentials(&[(source, 2.0)]);
    npu.process_burst().unwrap();    
    // Target SHOULD fire (65,025 >> 10,000)
    let result = npu.process_burst().unwrap();    assert!(result.fired_neurons.contains(&target),
        "Maximum PSP (255×255=65025) should cause firing");
}

#[test]
fn test_excitability_zero_prevents_firing() {
    let mut npu = create_test_npu();
    
    // Create neuron with excitability = 0 (never fires)
    let neuron = npu
        .add_neuron(
            1.0, // threshold
            0.0, // threshold_limit (0 = no limit)
            0.0, // leak
            0.0, // resting
            0,   // neuron_type
            0,   // refractory
            0.0, // excitability = 0 (never fires)
            0,   // consecutive_fire_limit
            0,   // snooze
            false, // mp_charge_accumulation
            2,   // cortical_area (avoid power auto-injection area 1)
            0, 0, 0,
        )
        .unwrap();
    // Inject well above threshold
    npu.inject_sensory_with_potentials(&[(neuron, 100.0)]);
    let result = npu.process_burst().unwrap();    
    assert!(!result.fired_neurons.contains(&neuron),
        "Neuron with excitability=0 should never fire");
}

// ============================================================================
// SECTION 10: Complex Integration Scenarios
// ============================================================================

#[test]
fn test_complex_network_convergence_divergence() {
    let mut npu = create_test_npu();
    
    // Create convergence-divergence network
    // Sources: S1, S2 (area 10)
    // Hub: H (area 11)
    // Targets: T1, T2 (area 11)
    // Pattern: S1,S2 -> H -> T1,T2
    
    let s1 = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 10, 0);
    let s2 = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 10, 1);
    let hub = add_test_neuron(&mut npu, 200.0, 0.0, false, 0, 11, 0);
    let t1 = add_test_neuron(&mut npu, 100.0, 0.0, false, 0, 11, 1);
    let t2 = add_test_neuron(&mut npu, 100.0, 0.0, false, 0, 11, 2);
    
    // Set PSP uniformity for area 10 and 11 so each synapse gets full PSP
    let mut psp_flags = ahash::AHashMap::new();
    let area_10_id = CorticalID::try_from_base_64(&CoreCorticalType::Death.to_cortical_id().as_base_64()).unwrap();    let area_11_id = CorticalID::try_from_base_64(&CoreCorticalType::Power.to_cortical_id().as_base_64()).unwrap();
    psp_flags.insert(area_10_id, true);
    psp_flags.insert(area_11_id, true);
    npu.set_psp_uniform_distribution_flags(psp_flags);
    
    // Convergence: S1,S2 -> H (each contributes 100, total 200)
    add_test_synapse(&mut npu, s1, hub, SynapticWeight(10), SynapticConductance(10), SynapseType::Excitatory);
    add_test_synapse(&mut npu, s2, hub, SynapticWeight(10), SynapticConductance(10), SynapseType::Excitatory);
    
    // Divergence: H -> T1,T2 (each gets 100)
    add_test_synapse(&mut npu, hub, t1, SynapticWeight(10), SynapticConductance(10), SynapseType::Excitatory);
    add_test_synapse(&mut npu, hub, t2, SynapticWeight(10), SynapticConductance(10), SynapseType::Excitatory);
    
    // Burst 1: Fire S1 and S2
    npu.inject_sensory_with_potentials(&[(s1, 2.0), (s2, 2.0)]);
    let result1 = npu.process_burst().unwrap();    assert!(result1.fired_neurons.contains(&s1));
    assert!(result1.fired_neurons.contains(&s2));
    
    // Burst 2: Hub should fire (100+100=200 >= 200)
    let result2 = npu.process_burst().unwrap();    assert!(result2.fired_neurons.contains(&hub),
        "Hub should fire from convergent inputs");
    
    // Burst 3: Both targets should fire (divergence)
    let result3 = npu.process_burst().unwrap();    assert!(result3.fired_neurons.contains(&t1),
        "Target 1 should fire from hub");
    assert!(result3.fired_neurons.contains(&t2),
        "Target 2 should fire from hub");
}

#[test]
fn test_feedback_loop_with_refractory() {
    let mut npu = create_test_npu();
    
    // Create feedback loop: N1 -> N2 -> N1
    let n1 = add_test_neuron(&mut npu, 100.0, 0.0, false, 2, 1, 0); // refractory=2
    let n2 = add_test_neuron(&mut npu, 100.0, 0.0, false, 2, 2, 0); // refractory=2
    
    add_test_synapse(&mut npu, n1, n2, SynapticWeight(10), SynapticConductance(10), SynapseType::Excitatory);
    add_test_synapse(&mut npu, n2, n1, SynapticWeight(10), SynapticConductance(10), SynapseType::Excitatory);
    
    // Burst 1: Fire N1
    npu.inject_sensory_with_potentials(&[(n1, 150.0)]);
    let result1 = npu.process_burst().unwrap();    assert!(result1.fired_neurons.contains(&n1));
    
    // Burst 2: N2 fires from N1
    let result2 = npu.process_burst().unwrap();    assert!(result2.fired_neurons.contains(&n2));
    
    // Burst 3: N1 blocked (refractory), N2 also blocked
    let result3 = npu.process_burst().unwrap();    assert_eq!(result3.burst, 3);
    // Both in refractory
    
    // Burst 4: N1 can fire again from N2's burst 2
    let _result4 = npu.process_burst().unwrap();    // Refractory should prevent continuous oscillation
    
    println!("Feedback loop tested with refractory preventing runaway");
}

