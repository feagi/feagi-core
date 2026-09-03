//! This library contains the core shared types / traits used by burst engines, and the various
//! burst engines themselves (feature gated)

pub use feagi_npu_burst_core::errors; // Should be exposed outside the NPU
pub use feagi_npu_burst_core::burst_engine_definitions::burst_engine_spawning; // Should be exposed outside NPU
pub use feagi_npu_burst_core::wrapped_values; // Should NOT be exposed outside NPU
pub mod data_interface_set; // Should NOT be exposed outside NPU
pub use enclosed_burst_engine::EnclosedBurstEngine; // Should NOT be exposed outside NPU

#[cfg(feature = "engines-esp32")]
pub extern crate feagi_npu_burst_esp32;

// TODO Rayon export

mod burst_engine_package;
mod enclosed_burst_engine;







