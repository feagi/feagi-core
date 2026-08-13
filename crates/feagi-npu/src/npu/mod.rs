//! This module will be lifted to become this crate directly. Treat exposures here as public
//! NPU API interfacing (used primarily by BDU)

pub mod composable_npu;
pub mod neuron_processor_unit_composable;
pub mod burst_engine_executor;
pub mod burst_engine;
pub mod burst_engine_worker;