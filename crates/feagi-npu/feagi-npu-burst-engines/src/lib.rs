
pub extern crate feagi_npu_burst_core;

mod composable_burst_engine_enum;
mod burst_engine_package;

pub mod implementations;
pub mod errors;

pub use burst_engine_package::BurstEnginePackage;
pub use composable_burst_engine_enum::BurstEngineEnum;

