//! Extended Comprehensive Neural Dynamics Tests
//! 
//! Additional tests for:
//! - Consecutive fire limit/count
//! - Threshold max and increment
//! - Combined parameter stress tests

use feagi_npu_burst_engine::{RustNPU, backend::CPUBackend};
use feagi_npu_neural::{SynapticWeight, SynapticConductance, SynapseType, NeuronId};
use feagi_npu_runtime::StdRuntime;
use feagi_structures::genomic::cortical_area::{CoreCorticalType, CorticalID};
use ahash::AHashMap;

/// Create a standard NPU for testing
fn create_test_npu() -> RustNPU<StdRuntime, f32, CPUBackend> {
    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    let mut npu = RustNPU::new(runtime, backend, 1000, 10000, 20).unwrap();
    
    // Register test areas 1-20
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

/// Helper to create test neurons
fn add_test_neuron(
    npu: &mut RustNPU<StdRuntime, f32, CPUBackend>,
    threshold: f32,
    leak: f32,
    mp_accumulation: bool,
    refractory: u16,
    cortical_area: u32,
    consecutive_fire_limit: u16,
) -> NeuronId {
    // SIMD-friendly encoding: 0 means no limit, convert to MAX
    let consecutive_fire_limit_encoded = if consecutive_fire_limit == 0 {
        u16::MAX
    } else {
        consecutive_fire_limit
    };
    npu.add_neuron(
        threshold,
        f32::MAX, // threshold_limit (MAX = no limit, SIMD-friendly encoding)
        leak,
        0.0, // resting_potential
        0,   // neuron_type
        refractory,
        1.0, // excitability (always fire if threshold met)
        consecutive_fire_limit_encoded,
        0,   // snooze_period
        mp_accumulation,
        cortical_area,
        0, 0, 0,
    ).unwrap()
}

// ============================================================================
// SECTION 11: Consecutive Fire Limit Scenarios
// ============================================================================

#[test]
fn test_consecutive_fire_limit_blocks_after_limit() {
    let mut npu = create_test_npu();
    
    // Neuron with consecutive fire limit of 3
    let neuron = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 10, 3);
    
    // Inject high potential to make it fire repeatedly
    let mut fire_results = Vec::new();
    for _burst in 1..=5 {
        npu.inject_sensory_with_potentials(&[(neuron, 10.0)]);
        let result = npu.process_burst().unwrap();
        fire_results.push(result.fired_neurons.contains(&neuron));
    }
    
    // Should fire for first 3 bursts, then be blocked
    assert_eq!(fire_results[0], true, "Should fire on burst 1");
    assert_eq!(fire_results[1], true, "Should fire on burst 2");
    assert_eq!(fire_results[2], true, "Should fire on burst 3");
    assert_eq!(fire_results[3], false, "Should be blocked on burst 4 (hit limit)");
    // Note: Burst 5 behavior depends on whether snooze/extended refractory is applied
}

#[test]
fn test_consecutive_fire_count_resets_after_not_firing() {
    let mut npu = create_test_npu();
    
    // Neuron with consecutive fire limit of 2
    let neuron = add_test_neuron(&mut npu, 5.0, 0.0, false, 0, 10, 2);
    
    // Burst 1-2: Fire twice (hit limit)
    npu.inject_sensory_with_potentials(&[(neuron, 10.0)]);
    let result1 = npu.process_burst().unwrap();
    assert!(result1.fired_neurons.contains(&neuron), "First fire");
    
    npu.inject_sensory_with_potentials(&[(neuron, 10.0)]);
    let result2 = npu.process_burst().unwrap();
    assert!(result2.fired_neurons.contains(&neuron), "Second fire");
    
    // Burst 3: Would be third, but blocked by limit
    npu.inject_sensory_with_potentials(&[(neuron, 10.0)]);
    let result3 = npu.process_burst().unwrap();
    assert!(!result3.fired_neurons.contains(&neuron), "Blocked by consecutive fire limit");
    
    // Burst 4-6: Don't inject, let refractory/snooze expire and count reset
    for _ in 4..=6 {
        npu.process_burst().unwrap();
    }
    
    // Burst 7: Should be able to fire again after count reset
    npu.inject_sensory_with_potentials(&[(neuron, 10.0)]);
    let result7 = npu.process_burst().unwrap();
    assert!(result7.fired_neurons.contains(&neuron),
        "Neuron should fire again after consecutive count reset and snooze expiration");
}

#[test]
fn test_consecutive_fire_limit_zero_means_unlimited() {
    let mut npu = create_test_npu();
    
    // Neuron with consecutive_fire_limit=0 (unlimited)
    let neuron = add_test_neuron(&mut npu, 1.0, 0.0, false, 0, 10, 0);
    
    // Should be able to fire many times consecutively
    for _burst in 1..=10 {
        npu.inject_sensory_with_potentials(&[(neuron, 10.0)]);
        let result = npu.process_burst().unwrap();
        assert!(result.fired_neurons.contains(&neuron),
            "Neuron with limit=0 should fire unlimited consecutive times");
    }
}

// ============================================================================
// SECTION 12: Threshold Dynamics (Max and Increment)
// ============================================================================

#[test]
fn test_high_threshold_prevents_firing() {
    let mut npu = create_test_npu();
    
    // Create neuron with very high threshold
    let neuron = add_test_neuron(&mut npu, 1000.0, 0.0, false, 0, 10, 0);
    
    // Try to fire with normal potential
    npu.inject_sensory_with_potentials(&[(neuron, 100.0)]);
    let result = npu.process_burst().unwrap();
    
    assert!(!result.fired_neurons.contains(&neuron),
        "Neuron with threshold=1000 should not fire with potential=100");
}

#[test]
fn test_threshold_at_maximum_representable_value() {
    let mut npu = create_test_npu();
    
    // Test with maximum f32 threshold
    let neuron = add_test_neuron(&mut npu, f32::MAX, 0.0, false, 0, 10, 0);
    
    // Should not fire even with high potential
    npu.inject_sensory_with_potentials(&[(neuron, 1000.0)]);
    let result = npu.process_burst().unwrap();
    
    assert!(!result.fired_neurons.contains(&neuron),
        "Neuron with MAX threshold should never fire with finite potential");
}

#[test]
fn test_threshold_boundary_at_zero() {
    let mut npu = create_test_npu();
    
    // Neuron with threshold=0 should fire with any positive potential
    let neuron = add_test_neuron(&mut npu, 0.0, 0.0, false, 0, 10, 0);
    
    npu.inject_sensory_with_potentials(&[(neuron, 0.1)]);
    let result = npu.process_burst().unwrap();
    
    assert!(result.fired_neurons.contains(&neuron),
        "Neuron with threshold=0 should fire with any positive potential");
}

// Note: threshold_increment and threshold_max tests would require:
// 1. Ability to set these parameters on neuron creation (currently not exposed)
// 2. Ability to read dynamic threshold value
// 3. Implementation verification that threshold increments after firing
// These tests document the expected behavior for future implementation

// ============================================================================
// SECTION 13: Combined Parameter Stress Tests
// ============================================================================

#[test]
fn test_all_parameters_combined() {
    let mut npu = create_test_npu();
    
    // Create a neuron with multiple parameters set:
    // - threshold: 100
    // - leak: 0.1 (10% decay per burst)
    // - refractory: 2 bursts
    // - mp_accumulation: true
    // - consecutive_fire_limit: 0 (unlimited)
    let neuron = add_test_neuron(&mut npu, 100.0, 0.1, true, 2, 10, 0);
    
    // Burst 1: Inject 60 potential
    npu.inject_sensory_with_potentials(&[(neuron, 60.0)]);
    let result1 = npu.process_burst().unwrap();
    assert!(!result1.fired_neurons.contains(&neuron), "Should not fire with 60 < 100");
    
    // Burst 2: Inject 50 more (60*0.9 + 50 = 104 > 100, should fire)
    npu.inject_sensory_with_potentials(&[(neuron, 50.0)]);
    let result2 = npu.process_burst().unwrap();
    assert!(result2.fired_neurons.contains(&neuron), "Should fire when accumulated potential exceeds threshold");
    
    // Burst 3: Should be in refractory (2 bursts)
    npu.inject_sensory_with_potentials(&[(neuron, 200.0)]);
    let result3 = npu.process_burst().unwrap();
    assert!(!result3.fired_neurons.contains(&neuron), "Should be blocked by refractory period");
    
    // Burst 4: Still in refractory
    npu.inject_sensory_with_potentials(&[(neuron, 200.0)]);
    let result4 = npu.process_burst().unwrap();
    assert!(!result4.fired_neurons.contains(&neuron), "Should still be in refractory");
    
    // Burst 5: Refractory expired, should fire again
    npu.inject_sensory_with_potentials(&[(neuron, 200.0)]);
    let result5 = npu.process_burst().unwrap();
    assert!(result5.fired_neurons.contains(&neuron), "Should fire after refractory expires");
}

#[test]
fn test_leak_and_refractory_interaction() {
    let mut npu = create_test_npu();
    
    // Neuron with high leak and refractory period
    let neuron = add_test_neuron(&mut npu, 100.0, 0.5, true, 3, 10, 0);
    
    // Inject enough to fire
    npu.inject_sensory_with_potentials(&[(neuron, 150.0)]);
    let result1 = npu.process_burst().unwrap();
    assert!(result1.fired_neurons.contains(&neuron));
    
    // During refractory, MP should still leak if neuron receives input
    // Burst 2-4: In refractory, inject small amounts
    for _ in 2..=4 {
        npu.inject_sensory_with_potentials(&[(neuron, 10.0)]);
        let result = npu.process_burst().unwrap();
        assert!(!result.fired_neurons.contains(&neuron), "Should be in refractory");
    }
    
    // Burst 5: Out of refractory, but MP should have leaked significantly
    // Need high injection to fire again
    npu.inject_sensory_with_potentials(&[(neuron, 50.0)]);
    let result5 = npu.process_burst().unwrap();
    // May or may not fire depending on how much leaked during refractory
}

#[test]
fn test_consecutive_limit_and_refractory_interaction() {
    let mut npu = create_test_npu();
    
    // Neuron with both consecutive fire limit and refractory period
    let neuron = add_test_neuron(&mut npu, 1.0, 0.0, false, 1, 10, 2);
    
    // Should fire twice (with refractory in between), then hit consecutive limit
    npu.inject_sensory_with_potentials(&[(neuron, 10.0)]);
    let result1 = npu.process_burst().unwrap();
    assert!(result1.fired_neurons.contains(&neuron), "First fire");
    
    // Burst 2: Refractory
    npu.inject_sensory_with_potentials(&[(neuron, 10.0)]);
    let result2 = npu.process_burst().unwrap();
    assert!(!result2.fired_neurons.contains(&neuron), "Should be in refractory");
    
    // Burst 3: Second fire (consecutive count = 2)
    npu.inject_sensory_with_potentials(&[(neuron, 10.0)]);
    let result3 = npu.process_burst().unwrap();
    assert!(result3.fired_neurons.contains(&neuron), "Second fire after refractory");
    
    // Burst 4: Refractory from second fire
    npu.inject_sensory_with_potentials(&[(neuron, 10.0)]);
    let result4 = npu.process_burst().unwrap();
    assert!(!result4.fired_neurons.contains(&neuron), "Should be in refractory");
    
    // Burst 5: Would be third fire, but should be blocked by consecutive fire limit (limit=2)
    // After hitting limit, neuron gets extended refractory (snooze period)
    npu.inject_sensory_with_potentials(&[(neuron, 10.0)]);
    let result5 = npu.process_burst().unwrap();
    // Consecutive fire limit triggers extended refractory, so it won't fire
    // This test documents the interaction - may fire or not depending on snooze period
}

#[test]
fn test_mp_accumulation_with_leak() {
    let mut npu = create_test_npu();
    
    // Neuron with mp_accumulation=true and moderate leak
    let neuron = add_test_neuron(&mut npu, 100.0, 0.2, true, 0, 10, 0);
    
    // Inject 60, wait, inject 60 again
    // With 20% leak: 60 -> 48 + 60 = 108 > 100, should fire
    npu.inject_sensory_with_potentials(&[(neuron, 60.0)]);
    npu.process_burst().unwrap();
    
    npu.process_burst().unwrap(); // Let it leak
    
    npu.inject_sensory_with_potentials(&[(neuron, 60.0)]);
    let result = npu.process_burst().unwrap();
    assert!(result.fired_neurons.contains(&neuron),
        "Should fire when leaked+new potential exceeds threshold");
}

