//! This library contains the core shared types / traits used by burst engines, and the various
//! burst engines themselves (feature gated)

pub use feagi_npu_burst_core::errors; // Should be exposed outside the NPU

pub use feagi_npu_burst_core::wrapped_values; // Should NOT be exposed outside NPU
pub mod data_interface_set; // Should NOT be exposed outside NPU




pub mod enclosed_non_composable_burst_engine;

#[cfg(feature = "alloc")]
pub mod enclosed_composable_burst_engine;

mod burst_engine_package;

//region Specific Burst Engine Sets

#[cfg(feature = "engines-esp32")]
pub extern crate feagi_npu_burst_esp32;

#[cfg(feature = "engines-rayon")]
pub extern crate feagi_npu_burst_rayon;

//endregion







