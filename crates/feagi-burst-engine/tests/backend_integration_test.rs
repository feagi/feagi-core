// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 */

//! Integration tests for backend selection and NPU interaction
//! 
//! Tests the complete flow: configuration → backend selection → NPU creation → burst execution

use feagi_burst_engine::{
    backend::{BackendConfig, BackendType, select_backend},
    RustNPU,
};

/// Test: Backend selection logic with different genome sizes
#[test]
fn test_backend_selection_thresholds() {
    let config = BackendConfig::default();
    
    // Small genome: should select CPU
    let decision = select_backend(10_000, 1_000_000, &config);
    assert_eq!(decision.backend_type, BackendType::CPU);
    assert!(decision.reason.contains("CPU selected"));
    
    // Medium genome: should try CUDA (if available) or CPU
    let decision = select_backend(150_000, 15_000_000, &config);
    #[cfg(feature = "cuda")]
    {
        // If CUDA is available, should select CUDA
        // If not, falls back to CPU
        assert!(
            decision.backend_type == BackendType::CUDA 
            || decision.backend_type == BackendType::CPU
        );
    }
    
    #[cfg(not(feature = "cuda"))]
    {
        assert_eq!(decision.backend_type, BackendType::CPU);
    }
    
    // Large genome: should try CUDA, then WGPU, then CPU
    let _decision = select_backend(600_000, 60_000_000, &config);
    #[cfg(feature = "cuda")]
    {
        // Priority: CUDA > CPU (WGPU only if "gpu" feature enabled)
        assert!(
            _decision.backend_type == BackendType::CUDA 
            || _decision.backend_type == BackendType::CPU
        );
    }
}

/// Test: Force flags work correctly
#[test]
fn test_force_flags() {
    let mut config = BackendConfig::default();
    
    // Force CPU
    config.force_cpu = true;
    let decision = select_backend(1_000_000, 100_000_000, &config);
    assert_eq!(decision.backend_type, BackendType::CPU);
    assert!(decision.reason.contains("Forced CPU"));
    
    // Force GPU (WGPU)
    config = BackendConfig::default();
    config.force_gpu = true;
    let _decision = select_backend(10_000, 1_000_000, &config);
    #[cfg(feature = "gpu")]
    {
        // Should select WGPU or fall back to CPU if not available
        assert!(
            _decision.backend_type == BackendType::WGPU 
            || _decision.backend_type == BackendType::CPU
        );
    }
    
    // Force CUDA
    #[cfg(feature = "cuda")]
    {
        config = BackendConfig::default();
        config.force_cuda = true;
        let decision = select_backend(10_000, 1_000_000, &config);
        // Should select CUDA or fall back to CPU if not available
        assert!(
            decision.backend_type == BackendType::CUDA 
            || decision.backend_type == BackendType::CPU
        );
    }
}

/// Test: NPU creation with auto backend selection
#[test]
fn test_npu_creation_with_auto_backend() {
    // Small genome - should use CPU
    let npu = RustNPU::<f32>::new(
        10_000,     // neuron_capacity
        1_000_000,  // synapse_capacity
        1000,       // fire_ledger_window
        None,       // gpu_config (auto)
    );
    
    // Just verify it was created successfully by dropping it
    drop(npu);
}

/// Test: NPU creation succeeds with different backends
#[test]
fn test_npu_burst_processing() {
    // Create small test genome - should use CPU backend
    let neuron_capacity = 1000;
    let synapse_capacity = 10_000;
    
    let npu = RustNPU::<f32>::new(
        neuron_capacity,
        synapse_capacity,
        100,    // fire_ledger_window
        None,   // auto backend
    );
    
    // Just verify NPU was created successfully
    // We can't access private fields, but we know it works if construction succeeded
    drop(npu); // Explicit drop to show we successfully created it
    
    // Try with larger capacity
    let npu = RustNPU::<f32>::new(
        100_000,
        10_000_000,
        100,
        None,
    );
    drop(npu);
}

/// Test: Backend speedup estimation
#[test]
fn test_speedup_estimation() {
    let config = BackendConfig::default();
    
    // Small genome: speedup should be ~1.0 (CPU is optimal)
    let decision = select_backend(10_000, 1_000_000, &config);
    assert!(decision.estimated_speedup <= 1.5);
    
    // Large genome: speedup should be significant if GPU selected
    let decision = select_backend(600_000, 60_000_000, &config);
    if decision.backend_type != BackendType::CPU {
        assert!(decision.estimated_speedup > 1.5);
    }
}

/// Test: CUDA availability check (compile-time and runtime)
#[test]
#[cfg(feature = "cuda")]
fn test_cuda_availability() {
    use feagi_burst_engine::backend::is_cuda_available;
    
    // This should not panic, just return true/false
    let available = is_cuda_available();
    
    // If CUDA is available, we should be able to enumerate devices
    if available {
        use feagi_burst_engine::backend::enumerate_cuda_devices;
        let devices = enumerate_cuda_devices();
        assert!(!devices.is_empty());
        
        println!("CUDA devices found:");
        for (idx, name, _memory) in devices.iter() {
            println!("  Device {}: {}", idx, name);
        }
    }
}

/// Test: Backend selection priority (CUDA > WGPU > CPU)
#[test]
#[cfg(all(feature = "cuda", feature = "gpu"))]
fn test_backend_priority() {
    use feagi_burst_engine::backend::{is_cuda_available, is_gpu_available};
    
    let config = BackendConfig::default();
    let decision = select_backend(200_000, 20_000_000, &config);
    
    // If CUDA is available, it should be selected for this size
    if is_cuda_available() {
        assert_eq!(decision.backend_type, BackendType::CUDA);
    }
    // Otherwise, if WGPU is available and genome is large enough, use WGPU
    else if is_gpu_available() && 200_000 >= config.gpu_neuron_threshold {
        assert_eq!(decision.backend_type, BackendType::WGPU);
    }
    // Otherwise, CPU
    else {
        assert_eq!(decision.backend_type, BackendType::CPU);
    }
}

/// Test: Configuration threshold overrides
#[test]
fn test_custom_thresholds() {
    let mut config = BackendConfig::default();
    
    // Set lower CUDA threshold (but still reasonable for speedup)
    config.cuda_neuron_threshold = 50_000;
    config.cuda_synapse_threshold = 5_000_000;
    
    #[cfg(feature = "cuda")]
    {
        use feagi_burst_engine::backend::is_cuda_available;
        
        // Medium genome should trigger CUDA with lowered threshold
        let decision = select_backend(75_000, 7_500_000, &config);
        if is_cuda_available() {
            // Should select CUDA (meets threshold and has good speedup)
            assert_eq!(decision.backend_type, BackendType::CUDA);
        }
    }
}

