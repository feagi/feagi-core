// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Tests for spatial gradient implementation of firing thresholds
//! 
//! These tests verify that firing thresholds can be spatially varied across
//! a cortical area using the increment_x, increment_y, and increment_z parameters.

use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::RustNPU;
use feagi_npu_runtime::StdRuntime;
use feagi_structures::genomic::cortical_area::CoreCorticalType;

#[test]
fn test_firing_threshold_spatial_gradient_3d() {
    // Create NPU with capacity for test neurons
    let mut npu = RustNPU::<StdRuntime, f32, CPUBackend>::new(
        StdRuntime,
        CPUBackend::new(),
        1000,
        10,
        1,
    )
    .unwrap();
    
    // Create a 3x3x2 cortical area (18 voxels, 1 neuron per voxel = 18 neurons)
    // Base threshold = 10.0
    // X increment = 1.0 (threshold increases by 1.0 for each X position)
    // Y increment = 2.0 (threshold increases by 2.0 for each Y position)
    // Z increment = 5.0 (threshold increases by 5.0 for each Z position)
    let cortical_idx = 1;
    let width = 3;
    let height = 3;
    let depth = 2;
    let neurons_per_voxel = 1;
    let base_threshold = 10.0;
    let increment_x = 1.0;
    let increment_y = 2.0;
    let increment_z = 5.0;

    // Required for neuron→area mapping during creation
    npu.register_cortical_area(
        cortical_idx,
        CoreCorticalType::Death.to_cortical_id().as_base_64(),
    );
    
    let neuron_count = npu.create_cortical_area_neurons(
        cortical_idx,
        width,
        height,
        depth,
        neurons_per_voxel,
        base_threshold,
        increment_x,
        increment_y,
        increment_z,
        0.0, // threshold_limit (no upper bound)
        0.0, // leak_coefficient
        0.0, // resting_potential
        0,   // neuron_type
        0,   // refractory_period
        1.0, // excitability
        0,   // consecutive_fire_limit
        0,   // snooze_period
        false, // mp_charge_accumulation
    ).expect("Neuron creation failed");
    
    assert_eq!(neuron_count, 18, "Should create 18 neurons (3x3x2)");
    
    // Expected threshold calculation:
    // threshold(x, y, z) = base + x*inc_x + y*inc_y + z*inc_z
    //                    = 10.0 + x*1.0 + y*2.0 + z*5.0
    
    // Neuron ordering: for x in 0..3, for y in 0..3, for z in 0..2
    let mut neuron_idx = 0;
    for x in 0..width {
        for y in 0..height {
            for z in 0..depth {
                let expected_threshold = base_threshold
                    + (x as f32 * increment_x)
                    + (y as f32 * increment_y)
                    + (z as f32 * increment_z);
                
                let actual_threshold = npu
                    .get_neuron_property_by_index(neuron_idx, "threshold")
                    .expect("Threshold should exist");
                
                assert!(
                    (actual_threshold - expected_threshold).abs() < 0.001,
                    "Neuron at ({}, {}, {}) has incorrect threshold: expected {:.1}, got {:.1}",
                    x, y, z, expected_threshold, actual_threshold
                );
                
                neuron_idx += 1;
            }
        }
    }
    
    // Verify some specific neurons:
    // Neuron at (0,0,0) index 0: 10.0 + 0 + 0 + 0 = 10.0
    let threshold_0 = npu
        .get_neuron_property_by_index(0, "threshold")
        .expect("Threshold should exist");
    assert!((threshold_0 - 10.0).abs() < 0.001);
    // Neuron at (2,2,1) index 17: 10.0 + 2 + 4 + 5 = 21.0
    let threshold_17 = npu
        .get_neuron_property_by_index(17, "threshold")
        .expect("Threshold should exist");
    assert!((threshold_17 - 21.0).abs() < 0.001);
}

#[test]
fn test_firing_threshold_no_gradient() {
    // Test that when all increments are 0.0, all neurons get the base threshold
    let mut npu = RustNPU::<StdRuntime, f32, CPUBackend>::new(StdRuntime, CPUBackend::new(), 100, 10, 1).unwrap();
    npu.register_cortical_area(1, CoreCorticalType::Death.to_cortical_id().as_base_64());
    
    let neuron_count = npu.create_cortical_area_neurons(
        1,      // cortical_idx
        2, 2, 2, // 2x2x2 = 8 neurons
        1,      // neurons_per_voxel
        50.0,   // base_threshold
        0.0, 0.0, 0.0, // no spatial gradient
        f32::MAX, // threshold_limit (MAX = no limit, SIMD-friendly encoding)
        0.0, 0.0, 0, 0, 1.0, 0, 0, false,
    ).expect("Neuron creation failed");
    
    assert_eq!(neuron_count, 8);
    
    // All neurons should have threshold = 50.0
    for idx in 0..neuron_count as usize {
        let threshold = npu
            .get_neuron_property_by_index(idx, "threshold")
            .expect("Threshold should exist");
        assert!(
            (threshold - 50.0).abs() < 0.001,
            "Neuron {} should have threshold 50.0, got {:.1}",
            idx, threshold
        );
    }
}

#[test]
fn test_firing_threshold_single_axis_gradient() {
    // Test gradient on X axis only
    let mut npu = RustNPU::<StdRuntime, f32, CPUBackend>::new(StdRuntime, CPUBackend::new(), 100, 10, 1).unwrap();
    npu.register_cortical_area(1, CoreCorticalType::Death.to_cortical_id().as_base_64());
    
    let neuron_count = npu.create_cortical_area_neurons(
        1,
        5, 1, 1, // 5x1x1 = 5 neurons in a line along X
        1,
        100.0,   // base
        10.0,    // X increment
        0.0, 0.0, // Y and Z increments = 0
        0.0,
        0.0, 0.0, 0, 0, 1.0, 0, 0, false,
    ).expect("Neuron creation failed");
    
    assert_eq!(neuron_count, 5);
    
    // Expected: 100, 110, 120, 130, 140
    for x in 0..5 {
        let expected = 100.0 + (x as f32 * 10.0);
        let actual = npu
            .get_neuron_property_by_index(x, "threshold")
            .expect("Threshold should exist");
        assert!(
            (actual - expected).abs() < 0.001,
            "Neuron at x={} should have threshold {:.1}, got {:.1}",
            x, expected, actual
        );
    }
}

#[test]
fn test_firing_threshold_multiple_neurons_per_voxel() {
    // Test that all neurons in the same voxel get the same threshold
    let mut npu = RustNPU::<StdRuntime, f32, CPUBackend>::new(StdRuntime, CPUBackend::new(), 100, 10, 1).unwrap();
    npu.register_cortical_area(1, CoreCorticalType::Death.to_cortical_id().as_base_64());
    
    let neuron_count = npu.create_cortical_area_neurons(
        1,
        2, 1, 1, // 2x1x1 = 2 voxels
        3,       // 3 neurons per voxel = 6 total
        20.0,    // base
        5.0,     // X increment
        0.0, 0.0,
        0.0,
        0.0, 0.0, 0, 0, 1.0, 0, 0, false,
    ).expect("Neuron creation failed");
    
    assert_eq!(neuron_count, 6);
    
    // First 3 neurons at x=0 should have threshold 20.0
    for idx in 0..3 {
        let threshold = npu
            .get_neuron_property_by_index(idx, "threshold")
            .expect("Threshold should exist");
        assert!((threshold - 20.0).abs() < 0.001);
    }
    
    // Next 3 neurons at x=1 should have threshold 25.0
    for idx in 3..6 {
        let threshold = npu
            .get_neuron_property_by_index(idx, "threshold")
            .expect("Threshold should exist");
        assert!((threshold - 25.0).abs() < 0.001);
    }
}

#[test]
fn test_firing_threshold_negative_gradient() {
    // Test that negative increments work (threshold decreases with position)
    let mut npu = RustNPU::<StdRuntime, f32, CPUBackend>::new(StdRuntime, CPUBackend::new(), 100, 10, 1).unwrap();
    npu.register_cortical_area(1, CoreCorticalType::Death.to_cortical_id().as_base_64());
    
    let neuron_count = npu.create_cortical_area_neurons(
        1,
        3, 1, 1, // 3 neurons along X
        1,
        100.0,   // base
        -10.0,   // negative X increment (decreasing)
        0.0, 0.0,
        0.0,
        0.0, 0.0, 0, 0, 1.0, 0, 0, false,
    ).expect("Neuron creation failed");
    
    assert_eq!(neuron_count, 3);
    
    // Expected: 100, 90, 80
    let threshold_0 = npu
        .get_neuron_property_by_index(0, "threshold")
        .expect("Threshold should exist");
    let threshold_1 = npu
        .get_neuron_property_by_index(1, "threshold")
        .expect("Threshold should exist");
    let threshold_2 = npu
        .get_neuron_property_by_index(2, "threshold")
        .expect("Threshold should exist");
    assert!((threshold_0 - 100.0).abs() < 0.001);
    assert!((threshold_1 - 90.0).abs() < 0.001);
    assert!((threshold_2 - 80.0).abs() < 0.001);
}

