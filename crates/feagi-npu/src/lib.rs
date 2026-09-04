//! System for managing backends that execute neuron and synaptic dynamics. The heart of FEAGI.

pub mod errors {
    pub use super::npu_errors::*;
    pub use feagi_npu_burst_engines::errors::{BurstEngineError};
}

/// Neuron Processing Units
pub mod npu {
    #[cfg(feature = "engines-esp32")]
    pub use super::neural_processing_units::embedded_npu::neural_processing_unit::EmbeddedNPU;
    // TODO standard
}

/// Allows for instantiation of specific burst engines with required parameters
pub mod burst_engine_spawners {
    #[cfg(feature = "engines-esp32")]
    pub use feagi_npu_burst_engines::feagi_npu_burst_esp32::esp_32::ESP32BoardESP32Spawner;
}


mod npu_errors;
mod neural_processing_units;

//mod wnpu;
//pub use standard::npu::npu_target_frequency::NPUTargetFrequency; // this is an exception! Do not expose anything else!
// internal, do not expose!
//mod npu;

//pub mod a;
//pub mod wnpu;
//pub mod npu;

