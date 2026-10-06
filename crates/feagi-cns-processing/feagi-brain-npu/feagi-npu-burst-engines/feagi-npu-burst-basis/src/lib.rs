//! Defines core structs needed for composable and non-composable burst engines

pub mod errors;
pub mod burst_phases;
pub use non_composable::NonComposableBurstEngineSpawner;
#[cfg(feature = "alloc")]
pub use composable::ComposableBurstEngineSpawner;

/// Structs that should be public of this crate but not leave the NPU crate
pub mod npu_sealed {

    pub use super::burst_phase_output::BurstPhaseOutput;
    
    
    #[cfg(feature = "alloc")]
    pub mod composable {
        pub use super::super::composable::npu_sealed::*;
    }

    pub mod non_composable {
        pub use super::super::non_composable::npu_sealed::*;
    }

    pub mod wrapped_values {
        pub use super::super::wrapped_values::*;
    }
}

mod wrapped_values;
mod burst_phase_output;
mod non_composable; // manages npu sealing inside
#[cfg(feature = "alloc")]
mod composable;  // manages npu sealing inside

