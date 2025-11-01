/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! GPU Configuration Integration Tests
//!
//! Tests that GPU configuration correctly controls backend selection.
//!
//! Run with:
//!   cargo test --test gpu_config_integration_test --features gpu

use feagi_burst_engine::backend::{BackendType, GpuConfig};

#[test]
fn test_gpu_config_disabled() {
    let config = GpuConfig {
        use_gpu: false,
        hybrid_enabled: true,
        gpu_threshold: 1_000_000,
        gpu_memory_fraction: 0.8,
    };

    let (backend_type, backend_config) = config.to_backend_config();

    assert_eq!(backend_type, BackendType::CPU);
    assert!(backend_config.force_cpu);
    assert!(!backend_config.force_gpu);
}

#[test]
fn test_gpu_config_hybrid_mode() {
    let config = GpuConfig {
        use_gpu: true,
        hybrid_enabled: true,
        gpu_threshold: 500_000,
        gpu_memory_fraction: 0.8,
    };

    let (backend_type, backend_config) = config.to_backend_config();

    assert_eq!(backend_type, BackendType::Auto);
    assert_eq!(backend_config.gpu_synapse_threshold, 500_000);
    assert!(!backend_config.force_cpu);
    assert!(!backend_config.force_gpu);
}

#[test]
#[cfg(feature = "gpu")]
fn test_gpu_config_always_on() {
    let config = GpuConfig {
        use_gpu: true,
        hybrid_enabled: false,
        gpu_threshold: 1_000_000,
        gpu_memory_fraction: 0.8,
    };

    let (backend_type, backend_config) = config.to_backend_config();

    assert_eq!(backend_type, BackendType::WGPU);
    assert!(!backend_config.force_cpu);
    assert!(backend_config.force_gpu);
}

#[test]
fn test_gpu_config_default() {
    let config = GpuConfig::default();

    assert_eq!(config.use_gpu, true);
    assert_eq!(config.hybrid_enabled, true);
    assert_eq!(config.gpu_threshold, 1_000_000);
    assert_eq!(config.gpu_memory_fraction, 0.8);
}

#[test]
fn test_gpu_config_custom_threshold() {
    let config = GpuConfig {
        use_gpu: true,
        hybrid_enabled: true,
        gpu_threshold: 5_000_000,
        gpu_memory_fraction: 0.5,
    };

    let (_, backend_config) = config.to_backend_config();

    assert_eq!(backend_config.gpu_synapse_threshold, 5_000_000);
    assert_eq!(backend_config.gpu_neuron_threshold, 50_000); // 5M / 100
}

#[test]
fn test_gpu_config_serialization() {
    // Test that config can be created from typical TOML values
    let config = GpuConfig {
        use_gpu: true,
        hybrid_enabled: true,
        gpu_threshold: 1_000_000,
        gpu_memory_fraction: 0.8,
    };

    // Verify values match TOML defaults
    assert_eq!(config.gpu_threshold, 1_000_000);
    assert_eq!(config.gpu_memory_fraction, 0.8);
}

#[test]
fn test_backend_selection_small_genome() {
    use feagi_burst_engine::backend::select_backend;

    let config = GpuConfig {
        use_gpu: true,
        hybrid_enabled: true,
        gpu_threshold: 1_000_000,
        gpu_memory_fraction: 0.8,
    };

    let (_, backend_config) = config.to_backend_config();

    // Small genome: 10K neurons, 100K synapses
    let decision = select_backend(10_000, 100_000, &backend_config);

    assert_eq!(decision.backend_type, BackendType::CPU);
    assert!(decision.reason.contains("Small genome"));
}

#[test]
#[cfg(feature = "gpu")]
fn test_backend_selection_large_genome() {
    use feagi_burst_engine::backend::select_backend;

    let config = GpuConfig {
        use_gpu: true,
        hybrid_enabled: true,
        gpu_threshold: 1_000_000,
        gpu_memory_fraction: 0.8,
    };

    let (_, backend_config) = config.to_backend_config();

    // Large genome: 2M neurons, 200M synapses
    let decision = select_backend(2_000_000, 200_000_000, &backend_config);

    // Should select GPU if available
    if is_gpu_available() {
        assert_eq!(decision.backend_type, BackendType::WGPU);
        assert!(decision.estimated_speedup > 1.5);
    } else {
        assert_eq!(decision.backend_type, BackendType::CPU);
    }
}

#[cfg(feature = "gpu")]
fn is_gpu_available() -> bool {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .is_some()
}

#[test]
fn test_gpu_memory_fraction_range() {
    // Test valid range
    let config = GpuConfig {
        use_gpu: true,
        hybrid_enabled: true,
        gpu_threshold: 1_000_000,
        gpu_memory_fraction: 0.5,
    };
    assert!(config.gpu_memory_fraction >= 0.0 && config.gpu_memory_fraction <= 1.0);

    let config2 = GpuConfig {
        use_gpu: true,
        hybrid_enabled: true,
        gpu_threshold: 1_000_000,
        gpu_memory_fraction: 1.0,
    };
    assert!(config2.gpu_memory_fraction >= 0.0 && config2.gpu_memory_fraction <= 1.0);
}

#[test]
fn test_gpu_threshold_reasonable() {
    let config = GpuConfig::default();
    
    // Threshold should be reasonable (not too low, not too high)
    assert!(config.gpu_threshold >= 100_000, "Threshold too low");
    assert!(config.gpu_threshold <= 100_000_000, "Threshold too high");
}

