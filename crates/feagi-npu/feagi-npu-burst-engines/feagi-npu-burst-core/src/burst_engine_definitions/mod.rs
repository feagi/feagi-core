//! Various structures / traits for defining types of burst engines

pub mod burst_phases;
pub mod burst_engine;
pub mod burst_phase_output;

#[cfg(feature = "composable")]
pub mod composable;