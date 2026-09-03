//! This library contains the core shared types / traits used by burst engines, and the various
//! burst engines themselves (feature gated)



pub mod errors;
pub mod data_interface_set;
pub use burst_engine_package::BurstEnginePackage;
mod burst_engine_package;
mod enclosed_burst_engine;







