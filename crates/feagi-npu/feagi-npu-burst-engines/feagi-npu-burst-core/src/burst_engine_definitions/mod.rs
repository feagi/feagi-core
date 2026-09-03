//! Various structures / traits for defining types of burst engines

pub mod burst_phases;
pub mod burst_phase_output;

pub mod bust_engine_spawning;
pub mod burst_engine;

#[cfg(feature = "alloc")]
pub mod composable_burst_engine_allocator;
#[cfg(feature = "alloc")]
pub mod connectome_change_messaging;