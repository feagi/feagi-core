//! System for managing backends that execute neuron and synaptic dynamics. The heart of FEAGI.

pub mod errors {
    pub use super::npu_errors::{BurstEngineWorkerPoolError, BurstEngineWorkerError, NPUError};
    pub use feagi_npu_burst_engines::errors::{BurstEngineError};
}

pub mod burst_engines {
    pub use feagi_npu_burst_core::composable::burst_engine_spawning::{EngineStartingConnectomeData, NoStartingConnectomeData, BurstEngineSpawner};
    pub use feagi_npu_burst_engines::data_interface_set; // TODO dont expose this directly, have some sort of enum setting

    #[cfg(feature = "engines-esp32")]
    pub use feagi_npu_burst_engines::feagi_npu_burst_esp32;
}




mod npu_errors;
pub mod neural_processing_units;
// TODO reorganize all of this for feature stuff

//mod wnpu;
//pub use standard::npu::npu_target_frequency::NPUTargetFrequency; // this is an exception! Do not expose anything else!
// internal, do not expose!
//mod npu;

//pub mod a;
//pub mod wnpu;
//pub mod npu;

