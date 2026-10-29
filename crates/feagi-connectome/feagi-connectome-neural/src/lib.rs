
pub extern crate neural_basis;

#[cfg(feature = "burst-engine-cpu")]
/// Running the FEAGI neural model on the CPU. This module contains various ways to do that for
/// various CPUS (single vs multithreaded, various SIMD instructions)
pub mod burst_engine_cpu {
    pub use neural_burst_cpu::parallel_burst_engines::cortical_cpu_parallel_dynamics::CorticalCPUParallelDynamics;
    // TODO engine
}

