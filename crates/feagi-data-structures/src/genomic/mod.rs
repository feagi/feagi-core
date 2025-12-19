//! Genomic types and identifiers for FEAGI.
//!
//! Provides core types for identifying and categorizing entities within the genome,
//! including custom, memory, core, sensory, and motor cortical regions.
#![doc = include_str!("../../docs/genomic.md")]

pub mod brain_regions; // Made public for external access
pub mod cortical_area;
pub mod descriptors;
mod motor_cortical_unit;
mod sensory_cortical_unit;

pub use brain_regions::{BrainRegion, RegionType};
pub use motor_cortical_unit::MotorCorticalUnit;
pub use sensory_cortical_unit::SensoryCorticalUnit;
