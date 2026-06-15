pub mod engines;
pub mod common_traits;
pub mod descriptor_flags;
mod burst_engine_just_completed_phase;

pub use burst_engine_just_completed_phase::{NPUWrappedBurstEngineMicroSecondsElapsed, BurstEngineJustCompletedPhase};
