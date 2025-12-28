// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Tests for spatial gradient implementation of firing thresholds
//! 
//! These tests verify that firing thresholds can be spatially varied across
//! a cortical area using the increment_x, increment_y, and increment_z parameters.

use feagi_npu_burst_engine::RustNPU;
use feagi_npu_runtime::NeuronStorage;

#[test]
fn test_firing_threshold_spatial_gradient_3d() {
    // Create NPU with capacity for test neurons
    let mut npu = RustNPU::<f32>::new(1000);
    
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
    
    // Get neuron storage to verify thresholds
    let storage = npu.neuron_storage();
    let thresholds = storage.thresholds();
    
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
                
                let actual_threshold = thresholds[neuron_idx];
                
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
    assert!((thresholds[0] - 10.0).abs() < 0.001);
    // Neuron at (2,2,1) index 17: 10.0 + 2 + 4 + 5 = 21.0
    assert!((thresholds[17] - 21.0).abs() < 0.001);
}

#[test]
fn test_firing_threshold_no_gradient() {
    // Test that when all increments are 0.0, all neurons get the base threshold
    let mut npu = RustNPU::<f32>::new(100);
    
    let neuron_count = npu.create_cortical_area_neurons(
        1,      // cortical_idx
        2, 2, 2, // 2x2x2 = 8 neurons
        1,      // neurons_per_voxel
        50.0,   // base_threshold
        0.0, 0.0, 0.0, // no spatial gradient
        0.0,    // threshold_limit
        0.0, 0.0, 0, 0, 1.0, 0, 0, false,
    ).expect("Neuron creation failed");
    
    assert_eq!(neuron_count, 8);
    
    let storage = npu.neuron_storage();
    let thresholds = storage.thresholds();
    
    // All neurons should have threshold = 50.0
    for (idx, &threshold) in thresholds.iter().enumerate() {
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
    let mut npu = RustNPU::<f32>::new(100);
    
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
    
    let storage = npu.neuron_storage();
    let thresholds = storage.thresholds();
    
    // Expected: 100, 110, 120, 130, 140
    for x in 0..5 {
        let expected = 100.0 + (x as f32 * 10.0);
        assert!(
            (thresholds[x] - expected).abs() < 0.001,
            "Neuron at x={} should have threshold {:.1}, got {:.1}",
            x, expected, thresholds[x]
        );
    }
}

#[test]
fn test_firing_threshold_multiple_neurons_per_voxel() {
    // Test that all neurons in the same voxel get the same threshold
    let mut npu = RustNPU::<f32>::new(100);
    
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
    
    let storage = npu.neuron_storage();
    let thresholds = storage.thresholds();
    
    // First 3 neurons at x=0 should have threshold 20.0
    for idx in 0..3 {
        assert!((thresholds[idx] - 20.0).abs() < 0.001);
    }
    
    // Next 3 neurons at x=1 should have threshold 25.0
    for idx in 3..6 {
        assert!((thresholds[idx] - 25.0).abs() < 0.001);
    }
}

#[test]
fn test_firing_threshold_negative_gradient() {
    // Test that negative increments work (threshold decreases with position)
    let mut npu = RustNPU::<f32>::new(100);
    
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
    
    let storage = npu.neuron_storage();
    let thresholds = storage.thresholds();
    
    // Expected: 100, 90, 80
    assert!((thresholds[0] - 100.0).abs() < 0.001);
    assert!((thresholds[1] - 90.0).abs() < 0.001);
    assert!((thresholds[2] - 80.0).abs() < 0.001);
}

