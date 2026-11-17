//! Burst engines, which run a connectome on a given kind of hardware. Each engine is a sub crate
//! toggled by a feature of the same name

pub mod npu_workers;

#[cfg(feature = "burst-cpu")]
/// Running the FEAGI neural model on the CPU. This module contains various engines to do that for
/// various CPUS (single vs multithreaded, various SIMD instructions)
pub use burst_cpu;

#[cfg(feature = "burst-esp32")]
/// Running the FEAGI neural model on the ESP32 family of SOCs
pub use burst_esp32;
