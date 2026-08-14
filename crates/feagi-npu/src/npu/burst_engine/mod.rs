
pub mod requests;
pub mod composable_implementations;

mod common_cpu_structs;
mod burst_engine;
mod composed_burst_engine;
mod composed_burst_engine_enum;

pub use burst_engine::BurstEngine;
pub use composed_burst_engine::ComposableBurstEngine;
// pub use burst_engine_enum::BurstEngineEnum;
pub use composed_burst_engine_enum::ComposableBurstEngineEnum;
