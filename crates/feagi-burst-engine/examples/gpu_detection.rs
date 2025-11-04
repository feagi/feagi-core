/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! GPU Detection Example
//!
//! Tests if WGPU can detect and initialize GPU on this system.
//! 
//! Usage:
//!   cargo run --example gpu_detection --features gpu
//!
//! Expected output on GPU-enabled system:
//!   ✓ GPU detected: Apple M4 Pro (Metal)
//!   ✓ Estimated speedup for 1M neurons: 7.2x
//!
//! Expected output on CPU-only system:
//!   ✗ No GPU detected
//!   → CPU backend will be used

#[cfg(feature = "gpu")]
fn main() {
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║           FEAGI GPU Detection Test                          ║");
    println!("╔═══════════════════════════════════════════════════════════════╝");
    println!();

    // Test 1: Check WGPU instance creation
    println!("Test 1: Creating WGPU instance...");
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    println!("  ✓ WGPU instance created");
    println!();

    // Test 2: Request GPU adapter
    println!("Test 2: Requesting GPU adapter...");
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }));

    match adapter {
        Some(adapter) => {
            let info = adapter.get_info();
            println!("  ✓ GPU DETECTED!");
            println!();
            println!("GPU Information:");
            println!("  Name:        {}", info.name);
            println!("  Backend:     {:?}", info.backend);
            println!("  Device Type: {:?}", info.device_type);
            println!("  Driver:      {} ({})", info.driver, info.driver_info);
            println!();

            // Test 3: Request device and queue
            println!("Test 3: Requesting GPU device and queue...");
            let device_result = pollster::block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("FEAGI GPU Detection Test"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            ));

            match device_result {
                Ok((device, queue)) => {
                    println!("  ✓ GPU device and queue created successfully");
                    println!();

                    // Test 4: Get limits
                    println!("Test 4: GPU Device Limits:");
                    let limits = device.limits();
                    println!("  Max buffer size:              {} MB", limits.max_buffer_size / 1_000_000);
                    println!("  Max storage buffer binding:   {} MB", limits.max_storage_buffer_binding_size / 1_000_000);
                    println!("  Max compute workgroup size:   {:?}", (
                        limits.max_compute_workgroup_size_x,
                        limits.max_compute_workgroup_size_y,
                        limits.max_compute_workgroup_size_z
                    ));
                    println!("  Max workgroups per dimension: {:?}", (
                        limits.max_compute_workgroups_per_dimension,
                        limits.max_compute_workgroups_per_dimension,
                        limits.max_compute_workgroups_per_dimension
                    ));
                    println!();

                    // Test 5: Estimate FEAGI performance
                    println!("Test 5: Estimated FEAGI Performance:");
                    println!();
                    estimate_performance(&info);

                    // Test 6: Create simple compute shader
                    println!("Test 6: Testing compute shader compilation...");
                    let shader_source = r#"
                        @group(0) @binding(0) var<storage, read> input: array<f32>;
                        @group(0) @binding(1) var<storage, read_write> output: array<f32>;
                        
                        @compute @workgroup_size(256)
                        fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
                            let idx = global_id.x;
                            if (idx < arrayLength(&input)) {
                                output[idx] = input[idx] * 2.0;
                            }
                        }
                    "#;

                    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("Test Shader"),
                        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
                    });

                    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                        label: Some("Test Pipeline"),
                        layout: None,
                        module: &shader,
                        entry_point: "main",
                    });

                    println!("  ✓ Compute shader compiled successfully");
                    println!("  ✓ Compute pipeline created successfully");
                    println!();

                    // Cleanup
                    drop(pipeline);
                    drop(shader);
                    drop(queue);
                    drop(device);

                    println!("╔═══════════════════════════════════════════════════════════════╗");
                    println!("║                    ✓ GPU FULLY FUNCTIONAL                    ║");
                    println!("║                                                               ║");
                    println!("║  FEAGI can use GPU acceleration on this system!              ║");
                    println!("╚═══════════════════════════════════════════════════════════════╝");
                    println!();
                }
                Err(e) => {
                    println!("  ✗ Failed to create GPU device: {}", e);
                    println!();
                    println!("╔═══════════════════════════════════════════════════════════════╗");
                    println!("║                   ✗ GPU NOT AVAILABLE                        ║");
                    println!("║                                                               ║");
                    println!("║  GPU detected but device creation failed.                    ║");
                    println!("║  FEAGI will use CPU backend.                                 ║");
                    println!("╚═══════════════════════════════════════════════════════════════╝");
                    println!();
                }
            }
        }
        None => {
            println!("  ✗ No GPU adapter found");
            println!();
            println!("╔═══════════════════════════════════════════════════════════════╗");
            println!("║                   ✗ GPU NOT DETECTED                         ║");
            println!("║                                                               ║");
            println!("║  No compatible GPU found on this system.                     ║");
            println!("║  FEAGI will use CPU backend.                                 ║");
            println!("║                                                               ║");
            println!("║  To enable GPU acceleration:                                 ║");
            println!("║  - macOS: Ensure Metal is supported                          ║");
            println!("║  - Linux: Install Vulkan drivers                             ║");
            println!("║  - Windows: Ensure DirectX 12 is available                   ║");
            println!("╚═══════════════════════════════════════════════════════════════╝");
            println!();
        }
    }
}

#[cfg(feature = "gpu")]
fn estimate_performance(info: &wgpu::AdapterInfo) {
    // Estimate performance based on backend
    let (tflops, bandwidth_gbs) = match info.backend {
        wgpu::Backend::Metal => {
            // Apple Silicon estimates
            if info.name.contains("M4") {
                (10.0, 120.0)  // M4 Pro: ~10 TFLOPS, ~120 GB/s
            } else if info.name.contains("M3") {
                (8.0, 100.0)   // M3 Pro: ~8 TFLOPS, ~100 GB/s
            } else if info.name.contains("M2") {
                (5.0, 80.0)    // M2 Pro: ~5 TFLOPS, ~80 GB/s
            } else if info.name.contains("M1") {
                (4.0, 60.0)    // M1 Pro: ~4 TFLOPS, ~60 GB/s
            } else {
                (3.0, 40.0)    // Intel/AMD GPU on Mac
            }
        }
        wgpu::Backend::Vulkan => {
            // NVIDIA/AMD estimates (Linux/Windows)
            if info.name.contains("RTX 4090") {
                (82.0, 1000.0)  // RTX 4090: 82 TFLOPS, 1 TB/s
            } else if info.name.contains("RTX 4080") {
                (48.0, 717.0)   // RTX 4080: 48 TFLOPS, 717 GB/s
            } else if info.name.contains("RTX 4070") {
                (29.0, 504.0)   // RTX 4070: 29 TFLOPS, 504 GB/s
            } else if info.name.contains("RTX 3090") {
                (35.0, 936.0)   // RTX 3090: 35 TFLOPS, 936 GB/s
            } else if info.name.contains("RX 7900") {
                (61.0, 960.0)   // AMD RX 7900 XTX: 61 TFLOPS, 960 GB/s
            } else if info.name.contains("Arc A770") {
                (17.2, 560.0)   // Intel Arc A770: 17.2 TFLOPS, 560 GB/s
            } else {
                (10.0, 200.0)   // Generic GPU
            }
        }
        wgpu::Backend::Dx12 => {
            // Windows DirectX 12 (similar to Vulkan estimates)
            (10.0, 200.0)       // Generic GPU
        }
        _ => (5.0, 100.0),      // Fallback
    };

    println!("  Estimated GPU Performance:");
    println!("    Compute:   ~{:.1} TFLOPS (FP32)", tflops);
    println!("    Bandwidth: ~{:.0} GB/s", bandwidth_gbs);
    println!();

    // Estimate FEAGI speedup for different genome sizes
    println!("  Estimated FEAGI Speedup:");
    println!("  ┌──────────────┬────────────┬──────────────┬─────────┐");
    println!("  │ Neurons      │ Synapses   │ CPU Time     │ Speedup │");
    println!("  ├──────────────┼────────────┼──────────────┼─────────┤");
    
    let test_cases = vec![
        (100_000, 10_000_000),
        (500_000, 50_000_000),
        (1_000_000, 100_000_000),
        (5_000_000, 500_000_000),
    ];
    
    for (neurons, synapses) in test_cases {
        let cpu_time_us = estimate_cpu_time(neurons, synapses);
        let gpu_time_us = estimate_gpu_time(neurons, synapses, tflops, bandwidth_gbs);
        let speedup = cpu_time_us / gpu_time_us;
        
        println!("  │ {:<12} │ {:<10} │ {:>8.0} μs │ {:>6.1}x │",
            format_number(neurons),
            format_number(synapses),
            cpu_time_us,
            speedup.max(0.1).min(100.0)
        );
    }
    
    println!("  └──────────────┴────────────┴──────────────┴─────────┘");
    println!();
}

#[cfg(feature = "gpu")]
fn estimate_cpu_time(neurons: usize, synapses: usize) -> f64 {
    // CPU: 100 GFLOPS effective
    let cpu_flops = 100_000_000_000.0;
    let synaptic_ops = synapses as f64 * 10.0;
    let neural_ops = neurons as f64 * 20.0;
    (synaptic_ops + neural_ops) / (cpu_flops / 1_000_000.0)
}

#[cfg(feature = "gpu")]
fn estimate_gpu_time(neurons: usize, synapses: usize, tflops: f64, bandwidth_gbs: f64) -> f64 {
    // GPU compute time
    let gpu_flops = tflops * 1_000_000_000_000.0;
    let synaptic_ops = synapses as f64 * 10.0;
    let neural_ops = neurons as f64 * 20.0;
    let compute_us = (synaptic_ops + neural_ops) / (gpu_flops / 1_000_000.0);
    
    // Transfer time (FCL optimization: only 1% of neurons)
    let firing_rate = 0.01;
    let transfer_bytes = (neurons as f64 * 4.0 * 2.0)  // Membrane potentials
                       + (neurons as f64 * 0.125)       // Fired mask
                       + (neurons as f64 * firing_rate * 4.0);  // Fired IDs
    let transfer_us = (transfer_bytes / (bandwidth_gbs * 1_000_000_000.0)) * 1_000_000.0 + 200.0;
    
    compute_us + transfer_us
}

#[cfg(feature = "gpu")]
fn format_number(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.0}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}

#[cfg(not(feature = "gpu"))]
fn main() {
    eprintln!("╔═══════════════════════════════════════════════════════════════╗");
    eprintln!("║                   GPU FEATURE NOT ENABLED                    ║");
    eprintln!("║                                                               ║");
    eprintln!("║  This example requires the 'gpu' feature flag.               ║");
    eprintln!("║                                                               ║");
    eprintln!("║  To enable GPU support:                                      ║");
    eprintln!("║    cargo run --example gpu_detection --features gpu          ║");
    eprintln!("║                                                               ║");
    eprintln!("║  Or build FEAGI with default features (GPU enabled):         ║");
    eprintln!("║    cargo build --release                                     ║");
    eprintln!("╚═══════════════════════════════════════════════════════════════╝");
}


