//! This module will be lifted to become this crate directly. Treat exposures here as public
//! NPU API interfacing (used primarily by BDU)

pub mod neuron_processor_unit_composable;
pub mod burst_engine;
pub mod burst_engine_worker;
pub mod neuron_processing_unit_commands;
pub mod npu_target_frequency;
pub mod data_interface;
