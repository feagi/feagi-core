//! This library contains the core shared types / traits used by burst engines, and the various
//! burst engines themselves (feature gated)

pub use feagi_npu_burst_core::errors; // Should be exposed outside the NPU
pub use feagi_npu_burst_core::burst_phases; // Should be exposed outside the NPU

pub mod npu_sealed {
    pub use feagi_npu_burst_core::npu_sealed::*;

    pub use super::enclosed_non_composable_burst_engine::{EnclosedNonComposableBurstEngine, NonComposableEngineToEnclosedNonComposable};
    #[cfg(feature = "alloc")]
    pub use super::enclosed_composable_burst_engine::{EnclosedComposableBurstEngine, ComposableEngineToEnclosedComposable};
    pub use super::burst_engine_package::{BurstEnginePackage, EnclosedEngine};
    
    pub mod data_interface_set {
        pub use super::super::data_interface_set::*;
    }
    
}

//region Specific Burst Engine Sets

#[cfg(feature = "engines-esp32")]
pub extern crate feagi_npu_burst_esp32;

#[cfg(feature = "engines-rayon")]
pub extern crate feagi_npu_burst_rayon;

//endregion


mod data_interface_set; // Should NOT be exposed outside NPU
mod enclosed_non_composable_burst_engine;
#[cfg(feature = "alloc")]
mod enclosed_composable_burst_engine;
mod burst_engine_package;
