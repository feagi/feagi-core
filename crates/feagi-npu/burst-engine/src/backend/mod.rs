// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! # Compute Backend Abstraction
//!
//! Provides a unified interface for different compute backends (CPU, GPU).
//! This allows the burst engine to use optimal hardware acceleration without
//! changing the high-level burst processing logic.

mod cpu;
#[cfg(feature = "cuda")]
mod cuda_backend;
#[cfg(feature = "gpu")]
mod wgpu_backend;

pub use cpu::CPUBackend;
#[cfg(feature = "cuda")]
pub use cuda_backend::{enumerate_cuda_devices, is_cuda_available, CUDABackend};
#[cfg(feature = "gpu")]
pub use wgpu_backend::WGPUBackend;

use feagi_npu_neural::types::*;
use feagi_npu_runtime::{NeuronStorage, SynapseStorage};

/// Result of processing a burst on any backend
#[derive(Debug, Clone)]
pub struct BackendBurstResult {
    /// Neurons that fired this burst
    pub fired_neurons: Vec<u32>,

    /// Performance metrics
    pub neurons_processed: usize,
    pub neurons_fired: usize,
    pub neurons_in_refractory: usize,

    /// Timing information (microseconds)
    pub timing: BurstTiming,
}

/// Detailed timing breakdown for burst processing
#[derive(Debug, Clone, Default)]
pub struct BurstTiming {
    /// Time spent on synaptic propagation (μs)
    pub synaptic_propagation_us: f64,

    /// Time spent on neural dynamics (μs)
    pub neural_dynamics_us: f64,

    /// Time spent on data transfer (GPU only, μs)
    pub transfer_us: f64,

    /// Total burst time (μs)
    pub total_us: f64,
}

/// Compute backend trait - abstracts CPU vs GPU execution
///
/// **FCL-Aware Design**: Backends process only Fire Candidate List neurons,
/// not the entire neuron array. This enables efficient sparse processing on GPU.
/// Compute backend trait (CPU, GPU, NPU, Hailo)
///
/// Generic over:
/// - `T: NeuralValue` - Numeric type for membrane potentials
/// - `N: NeuronStorage` - Neuron storage implementation
/// - `S: SynapseStorage` - Synapse storage implementation
pub trait ComputeBackend<T: NeuralValue, N: NeuronStorage<Value = T>, S: SynapseStorage>:
    Send + Sync
{
    /// Get backend type name for logging/debugging
    fn backend_name(&self) -> &str;

    /// Process synaptic propagation: fired neurons → membrane potential updates
    ///
    /// **FCL Integration**: Results are accumulated into FCL, not written to storage.
    ///
    /// # Arguments
    /// * `fired_neurons` - Neurons that fired in previous burst (sparse list)
    /// * `synapse_storage` - All synapses (trait-based storage)
    /// * `fcl` - Fire Candidate List (accumulates synaptic contributions)
    ///
    /// # Returns
    /// Number of synapses processed
    fn process_synaptic_propagation(
        &mut self,
        fired_neurons: &[u32],
        synapse_storage: &S,
        fcl: &mut FireCandidateList,
    ) -> Result<usize>;

    /// Process neural dynamics: FCL candidates → firing decisions
    ///
    /// **FCL-Aware**: Only processes neurons in FCL (~1-10% of total neurons).
    /// GPU backends upload FCL as sparse array, CPU backends iterate FCL directly.
    ///
    /// # Arguments
    /// * `fcl` - Fire Candidate List (which neurons to process)
    /// * `neuron_storage` - Full neuron storage (trait-based)
    /// * `burst_count` - Current burst number (for excitability randomness)
    ///
    /// # Returns
    /// List of neurons that fired + statistics
    fn process_neural_dynamics(
        &mut self,
        fcl: &FireCandidateList,
        neuron_storage: &mut N,
        burst_count: u64,
    ) -> Result<(Vec<u32>, usize, usize)>;

    /// Process full burst cycle (synaptic + neural)
    ///
    /// **FCL-Aware**: Takes FCL as input/output for both phases.
    ///
    /// This is the primary entry point. Backends can override this for
    /// optimizations (e.g., keep data on GPU between stages).
    ///
    /// Default implementation calls the two methods separately.
    fn process_burst(
        &mut self,
        fired_neurons: &[u32],
        synapse_storage: &S,
        fcl: &mut FireCandidateList,
        neuron_storage: &mut N,
        burst_count: u64,
    ) -> Result<BackendBurstResult> {
        let start = std::time::Instant::now();

        // Phase 1: Synaptic propagation → FCL
        let synaptic_start = std::time::Instant::now();
        let _synapses_processed =
            self.process_synaptic_propagation(fired_neurons, synapse_storage, fcl)?;
        let synaptic_us = synaptic_start.elapsed().as_micros() as f64;

        // Phase 2: Neural dynamics (FCL → fired neurons)
        let neural_start = std::time::Instant::now();
        let (new_fired, processed, in_refractory) =
            self.process_neural_dynamics(fcl, neuron_storage, burst_count)?;
        let neural_us = neural_start.elapsed().as_micros() as f64;

        let total_us = start.elapsed().as_micros() as f64;

        Ok(BackendBurstResult {
            fired_neurons: new_fired,
            neurons_processed: processed,
            neurons_fired: 0, // Will be set by caller
            neurons_in_refractory: in_refractory,
            timing: BurstTiming {
                synaptic_propagation_us: synaptic_us,
                neural_dynamics_us: neural_us,
                transfer_us: 0.0,
                total_us,
            },
        })
    }

    /// Initialize/upload persistent data to backend
    ///
    /// For GPU backends, this uploads static data that doesn't change
    /// between bursts (thresholds, leak coefficients, etc.)
    ///
    /// For CPU backends, this is a no-op.
    fn initialize_persistent_data(
        &mut self,
        _neuron_storage: &N,
        _synapse_storage: &S,
    ) -> Result<()> {
        Ok(())
    }

    /// Notify backend that genome has changed (invalidate caches)
    fn on_genome_change(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Backend type enum for construction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    /// CPU with SIMD optimization (current implementation)
    CPU,

    /// GPU via WGPU (Metal/Vulkan/DirectX - cross-platform)
    #[cfg(feature = "gpu")]
    WGPU,

    /// GPU via CUDA (NVIDIA only - highest performance)
    #[cfg(feature = "cuda")]
    CUDA,

    /// Auto-select based on genome size and hardware availability
    Auto,
}

impl Default for BackendType {
    fn default() -> Self {
        Self::Auto
    }
}

impl std::fmt::Display for BackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendType::CPU => write!(f, "CPU"),
            #[cfg(feature = "gpu")]
            BackendType::WGPU => write!(f, "WGPU"),
            #[cfg(feature = "cuda")]
            BackendType::CUDA => write!(f, "CUDA"),
            BackendType::Auto => write!(f, "Auto"),
        }
    }
}

impl std::str::FromStr for BackendType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "cpu" => Ok(BackendType::CPU),
            #[cfg(feature = "gpu")]
            "wgpu" | "gpu" => Ok(BackendType::WGPU),
            #[cfg(feature = "cuda")]
            "cuda" => Ok(BackendType::CUDA),
            "auto" => Ok(BackendType::Auto),
            _ => Err(Error::InvalidBackend(s.to_string())),
        }
    }
}

/// Configuration for backend auto-selection
#[derive(Debug, Clone)]
pub struct BackendConfig {
    /// Minimum neurons to consider WGPU GPU (default: 500,000)
    pub gpu_neuron_threshold: usize,

    /// Minimum synapses to consider WGPU GPU (default: 50,000,000)
    pub gpu_synapse_threshold: usize,

    /// Minimum neurons to consider CUDA GPU (default: 100,000)
    /// CUDA has lower overhead than WGPU, so benefits from smaller genomes
    pub cuda_neuron_threshold: usize,

    /// Minimum synapses to consider CUDA GPU (default: 10,000,000)
    pub cuda_synapse_threshold: usize,

    /// Minimum expected firing rate to benefit from GPU (default: 0.005 = 0.5%)
    pub gpu_min_firing_rate: f32,

    /// Force CPU even if GPU would be beneficial
    pub force_cpu: bool,

    /// Force WGPU GPU even if CPU would be better (for testing)
    pub force_gpu: bool,

    /// Force CUDA GPU even if CPU/WGPU would be better (for testing)
    #[cfg(feature = "cuda")]
    pub force_cuda: bool,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            // WGPU: Based on benchmarks, >500K neurons = 2-3x speedup
            gpu_neuron_threshold: 500_000,

            // 500K neurons × 100 synapses/neuron = 50M synapses
            gpu_synapse_threshold: 50_000_000,

            // CUDA: Lower overhead, benefits from smaller genomes
            // Based on A100 validation, ~100K neurons is the sweet spot
            cuda_neuron_threshold: 100_000,
            cuda_synapse_threshold: 10_000_000,

            // Need at least 0.5% firing for GPU parallelism to be worth it
            gpu_min_firing_rate: 0.005,

            force_cpu: false,
            force_gpu: false,
            #[cfg(feature = "cuda")]
            force_cuda: false,
        }
    }
}

/// GPU configuration from application config (TOML)
///
/// This struct provides a simplified interface for GPU configuration
/// that can be passed from the application layer (feagi/feagi-inference-engine)
/// to the burst engine without creating tight coupling to feagi-config.
#[derive(Debug, Clone)]
pub struct GpuConfig {
    /// Enable GPU processing globally
    pub use_gpu: bool,

    /// Enable hybrid CPU/GPU auto-selection based on genome size
    pub hybrid_enabled: bool,

    /// Threshold in synapses to consider GPU in hybrid mode
    pub gpu_threshold: usize,

    /// Fraction of GPU memory to use (0.0-1.0)
    pub gpu_memory_fraction: f64,
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self {
            use_gpu: true,
            hybrid_enabled: true,
            gpu_threshold: 1_000_000,
            gpu_memory_fraction: 0.8,
        }
    }
}

impl GpuConfig {
    /// Convert to BackendType and BackendConfig for backend selection
    ///
    /// This method translates high-level GPU configuration into the low-level
    /// backend selection parameters.
    pub fn to_backend_selection(&self) -> (BackendType, BackendConfig) {
        let backend_type = if !self.use_gpu {
            // GPU explicitly disabled
            BackendType::CPU
        } else if self.hybrid_enabled {
            // Hybrid mode: auto-select based on genome size
            BackendType::Auto
        } else {
            // GPU always on (if available)
            #[cfg(feature = "gpu")]
            {
                BackendType::WGPU
            }
            #[cfg(not(feature = "gpu"))]
            {
                tracing::warn!("GPU requested but 'gpu' feature not enabled at compile time, falling back to CPU");
                BackendType::CPU
            }
        };

        let backend_config = BackendConfig {
            // Estimate neuron threshold from synapse threshold
            // Assume ~100 synapses per neuron (typical for FEAGI genomes)
            gpu_neuron_threshold: self.gpu_threshold / 100,
            gpu_synapse_threshold: self.gpu_threshold,
            force_cpu: !self.use_gpu,
            force_gpu: self.use_gpu && !self.hybrid_enabled,
            ..Default::default()
        };

        (backend_type, backend_config)
    }
}

/// Backend selection decision with rationale
#[derive(Debug, Clone)]
pub struct BackendDecision {
    pub backend_type: BackendType,
    pub reason: String,
    pub estimated_speedup: f32,
}

/// Auto-select optimal backend based on genome size and hardware
///
/// Selection priority:
/// 1. Honor force flags (force_cpu, force_cuda, force_gpu)
/// 2. Try CUDA (if available and genome large enough) - highest performance
/// 3. Try WGPU (if available and genome large enough) - cross-platform
/// 4. Fall back to CPU - always available
pub fn select_backend(
    neuron_count: usize,
    synapse_count: usize,
    config: &BackendConfig,
) -> BackendDecision {
    // Force overrides
    if config.force_cpu {
        return BackendDecision {
            backend_type: BackendType::CPU,
            reason: "Forced CPU via configuration".to_string(),
            estimated_speedup: 1.0,
        };
    }

    #[cfg(feature = "cuda")]
    if config.force_cuda {
        if is_cuda_available() {
            return BackendDecision {
                backend_type: BackendType::CUDA,
                reason: "Forced CUDA via configuration".to_string(),
                estimated_speedup: estimate_cuda_speedup(neuron_count, synapse_count),
            };
        } else {
            info!("⚠️  CUDA forced but not available, falling back to CPU");
            return BackendDecision {
                backend_type: BackendType::CPU,
                reason: "CUDA forced but not available, falling back to CPU".to_string(),
                estimated_speedup: 1.0,
            };
        }
    }

    #[cfg(feature = "gpu")]
    if config.force_gpu {
        if is_gpu_available() {
            return BackendDecision {
                backend_type: BackendType::WGPU,
                reason: "Forced WGPU via configuration".to_string(),
                estimated_speedup: estimate_gpu_speedup(neuron_count, synapse_count),
            };
        } else {
            info!("⚠️  WGPU forced but not available, falling back to CPU");
            return BackendDecision {
                backend_type: BackendType::CPU,
                reason: "WGPU forced but not available, falling back to CPU".to_string(),
                estimated_speedup: 1.0,
            };
        }
    }

    // Auto-selection: Try CUDA first (best performance), then WGPU (cross-platform), then CPU

    // Check CUDA threshold
    let _meets_cuda_neuron_threshold = neuron_count >= config.cuda_neuron_threshold;
    let _meets_cuda_synapse_threshold = synapse_count >= config.cuda_synapse_threshold;

    #[cfg(feature = "cuda")]
    {
        if _meets_cuda_neuron_threshold || _meets_cuda_synapse_threshold {
            if is_cuda_available() {
                let speedup = estimate_cuda_speedup(neuron_count, synapse_count);

                // Use CUDA if speedup is meaningful (>1.5x)
                if speedup > 1.5 {
                    return BackendDecision {
                        backend_type: BackendType::CUDA,
                        reason: format!(
                            "CUDA selected: {} neurons, {} synapses (optimal for NVIDIA GPUs)",
                            neuron_count, synapse_count
                        ),
                        estimated_speedup: speedup,
                    };
                }
            }
        }
    }

    // Check WGPU threshold
    let _meets_wgpu_neuron_threshold = neuron_count >= config.gpu_neuron_threshold;
    let _meets_wgpu_synapse_threshold = synapse_count >= config.gpu_synapse_threshold;

    #[cfg(feature = "gpu")]
    {
        if _meets_wgpu_neuron_threshold || _meets_wgpu_synapse_threshold {
            if is_gpu_available() {
                let speedup = estimate_gpu_speedup(neuron_count, synapse_count);

                // Use WGPU if speedup is meaningful (>1.5x)
                if speedup > 1.5 {
                    return BackendDecision {
                        backend_type: BackendType::WGPU,
                        reason: format!(
                            "WGPU selected: {} neurons, {} synapses (cross-platform GPU)",
                            neuron_count, synapse_count
                        ),
                        estimated_speedup: speedup,
                    };
                }
            }
        }
    }

    // Default to CPU
    BackendDecision {
        backend_type: BackendType::CPU,
        reason: format!(
            "CPU selected: {} neurons, {} synapses (below GPU thresholds or GPU not available)",
            neuron_count, synapse_count
        ),
        estimated_speedup: 1.0,
    }
}

/// Check if GPU is available
#[cfg(feature = "gpu")]
fn is_gpu_available() -> bool {
    use wgpu::Backends;

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: Backends::all(),
        ..Default::default()
    });

    pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .is_some()
}

/// Estimate GPU speedup based on genome size
#[cfg(feature = "gpu")]
fn estimate_gpu_speedup(neuron_count: usize, synapse_count: usize) -> f32 {
    // Empirical model based on realistic hardware:
    // - PCIe 4.0: ~25 GB/s bidirectional
    // - M4 Pro GPU: ~10 TFLOPS FP32
    // - CPU (16-core): ~100 GFLOPS effective
    // - Full GPU pipeline (synaptic + neural) assumed

    let neurons = neuron_count as f32;
    let synapses = synapse_count as f32;

    // Transfer time (microseconds) - realistic PCIe 4.0 speeds
    // NOTE: Synapses are PERSISTENT on GPU - no transfer cost per burst!
    // Per-burst transfers:
    // - Membrane potentials (4 bytes/neuron, both ways)
    // - Fired neuron mask (1 bit/neuron = 0.125 bytes/neuron, output only)
    // - Fired neuron IDs (assumed ~1% fire rate, 4 bytes each)
    let firing_rate = 0.01; // Assume 1% neurons fire per burst
    let transfer_bytes = (neurons * 4.0 * 2.0)  // Membrane potentials bidirectional
                       + (neurons * 0.125)     // Fired mask (bitpacked)
                       + (neurons * firing_rate * 4.0); // Fired neuron IDs
    let transfer_bandwidth_gbs = 25.0; // GB/s for PCIe 4.0
                                       // Convert: bytes / (GB/s * 1e9 bytes/GB) = seconds, then * 1e6 = microseconds
    let transfer_us =
        (transfer_bytes / (transfer_bandwidth_gbs * 1_000_000_000.0)) * 1_000_000.0 + 200.0; // +200μs fixed overhead

    // CPU compute time (microseconds)
    // Synaptic: ~10 ops per synapse (hash lookup, weight calc, accumulation)
    // Neural: ~20 ops per neuron (leak, threshold check, refractory, RNG)
    let cpu_flops = 100_000_000_000.0; // 100 GFLOPS effective (cache locality, branching)
    let cpu_synaptic_us = (synapses * 10.0) / (cpu_flops / 1_000_000.0);
    let cpu_neural_us = (neurons * 20.0) / (cpu_flops / 1_000_000.0);
    let cpu_total_us = cpu_synaptic_us + cpu_neural_us;

    // GPU compute time (microseconds)
    // GPU benefits from massive parallelism: 100-200x speedup for compute
    let gpu_flops = 10_000_000_000_000.0; // 10 TFLOPS (M4 Pro/RTX 4090)
    let gpu_synaptic_us = (synapses * 10.0) / (gpu_flops / 1_000_000.0);
    let gpu_neural_us = (neurons * 20.0) / (gpu_flops / 1_000_000.0);
    let gpu_compute_us = gpu_synaptic_us + gpu_neural_us;

    let gpu_total_us = transfer_us + gpu_compute_us;

    // Speedup = CPU time / GPU time
    let speedup = cpu_total_us / gpu_total_us;

    // Cap at reasonable maximum (100x)
    speedup.min(100.0).max(0.1)
}

/// Estimate CUDA GPU speedup vs CPU
///
/// CUDA characteristics (based on A100/H100):
/// - Lower launch overhead (~50-100μs vs 200μs for WGPU)
/// - Higher FLOPS (19.5 TFLOPS for A100, 67 TFLOPS for H100)
/// - Better memory bandwidth (1.5TB/s for A100, 3TB/s for H100)
/// - Native GPU access (no abstraction layer)
#[cfg(feature = "cuda")]
fn estimate_cuda_speedup(neuron_count: usize, synapse_count: usize) -> f32 {
    let neurons = neuron_count as f32;
    let synapses = synapse_count as f32;

    // Transfer time (microseconds) - NVLink/PCIe 5.0 speeds
    // CUDA has lower overhead than WGPU (~100μs vs ~200μs)
    let firing_rate = 0.01; // Assume 1% neurons fire per burst
    let transfer_bytes = (neurons * 4.0 * 2.0)  // Membrane potentials bidirectional
                       + (neurons * 0.125)      // Fired mask (bitpacked)
                       + (neurons * firing_rate * 4.0); // Fired neuron IDs
    let transfer_bandwidth_gbs = 32.0; // GB/s for PCIe 5.0
    let transfer_us =
        (transfer_bytes / (transfer_bandwidth_gbs * 1_000_000_000.0)) * 1_000_000.0 + 100.0; // +100μs fixed overhead (lower than WGPU)

    // CPU compute time (same as WGPU estimation)
    let cpu_flops = 100_000_000_000.0; // 100 GFLOPS effective
    let cpu_synaptic_us = (synapses * 10.0) / (cpu_flops / 1_000_000.0);
    let cpu_neural_us = (neurons * 20.0) / (cpu_flops / 1_000_000.0);
    let cpu_total_us = cpu_synaptic_us + cpu_neural_us;

    // CUDA compute time (microseconds)
    // A100: 19.5 TFLOPS, H100: 67 TFLOPS
    // Use conservative estimate (A100 level)
    let cuda_flops = 19_500_000_000_000.0; // 19.5 TFLOPS (A100)
    let cuda_synaptic_us = (synapses * 10.0) / (cuda_flops / 1_000_000.0);
    let cuda_neural_us = (neurons * 20.0) / (cuda_flops / 1_000_000.0);
    let cuda_compute_us = cuda_synaptic_us + cuda_neural_us;

    let cuda_total_us = transfer_us + cuda_compute_us;

    // Speedup = CPU time / CUDA time
    let speedup = cpu_total_us / cuda_total_us;

    // Cap at reasonable maximum (100x)
    speedup.min(100.0).max(0.1)
}

// TODO: create_backend removed - incompatible with new generic backend design
// RustNPU now takes backend directly in constructor
// Users should create backend explicitly:
//   let backend = CPUBackend::new();
//   let npu = RustNPU::new(runtime, backend, ...)?;
/*
pub fn create_backend<T: NeuralValue>(
    backend_type: BackendType,
    neuron_capacity: usize,
    synapse_capacity: usize,
    config: &BackendConfig,
) -> Result<Box<dyn ComputeBackend<T, N, S>>> {
    let actual_type = if backend_type == BackendType::Auto {
        // Count will be updated later, use capacity as estimate
        let decision = select_backend(neuron_capacity, synapse_capacity, config);
        info!(
            "🎯 Backend auto-selection: {} ({})",
            decision.backend_type, decision.reason
        );
        if decision.estimated_speedup > 1.0 {
            info!("   Estimated speedup: {:.1}x", decision.estimated_speedup);
        }
        decision.backend_type
    } else {
        backend_type
    };

    match actual_type {
        BackendType::CPU => {
            info!("🖥️  Using CPU backend (SIMD optimized)");
            Ok(Box::new(CPUBackend::new()))
        }
        #[cfg(feature = "gpu")]
        BackendType::WGPU => {
            // GPU backend currently only supports f32
            // This is because shaders are compiled for f32 arithmetic
            if std::any::TypeId::of::<T>() != std::any::TypeId::of::<f32>() {
                let type_name = std::any::type_name::<T>();
                info!("⚠️  WGPU backend requested but {} quantization is not supported on GPU", type_name);
                info!("   GPU shaders are currently f32-only. Falling back to CPU backend.");
                info!("   Future: f16 GPU support planned for mixed-precision training");
                return Ok(Box::new(CPUBackend::new()));
            }

            info!("🎮 Using WGPU backend (cross-platform GPU)");
            // SAFETY: We've verified T == f32 above, so this is safe
            // We use unsafe transmute because we can't directly cast Box<WGPUBackend> to Box<dyn ComputeBackend<T>>
            // when T is a generic parameter, even though we know T == f32 at runtime
            let backend = WGPUBackend::new(neuron_capacity, synapse_capacity)?;
            let boxed: Box<dyn ComputeBackend<f32>> = Box::new(backend);
            Ok(unsafe { std::mem::transmute(boxed) })
        }
        #[cfg(feature = "cuda")]
        BackendType::CUDA => {
            // CUDA backend currently only supports f32
            if std::any::TypeId::of::<T>() != std::any::TypeId::of::<f32>() {
                let type_name = std::any::type_name::<T>();
                info!("⚠️  CUDA backend requested but {} quantization is not supported", type_name);
                info!("   CUDA kernels are currently f32-only. Falling back to CPU backend.");
                info!("   Future: f16/int8 CUDA support planned for mixed-precision training");
                return Ok(Box::new(CPUBackend::new()));
            }

            info!("🚀 Using CUDA backend (NVIDIA GPU - high performance)");
            // SAFETY: We've verified T == f32 above, so this is safe
            let backend = CUDABackend::new(neuron_capacity, synapse_capacity)?;
            let boxed: Box<dyn ComputeBackend<f32>> = Box::new(backend);
            Ok(unsafe { std::mem::transmute(boxed) })
        }
        BackendType::Auto => {
            // Should never reach here - Auto should be resolved in from_config()
            unreachable!("BackendType::Auto should be resolved before create_backend() is called")
        }
    }
}
*/
