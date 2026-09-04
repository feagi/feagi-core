//! Defines core structs needed for composable and non-composable burst engines

pub mod errors;

pub mod non_composable; // manages npu sealing inside

#[cfg(feature = "alloc")]
pub mod composable;  // manages npu sealing inside

pub mod burst_phases;
pub mod wrapped_values;

pub use burst_phase_output::BurstPhaseOutput;

mod burst_phase_output;
