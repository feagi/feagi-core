//! Genomic types and identifiers for FEAGI.
//!
//! Provides core types for identifying and categorizing entities within the genome,
//! including custom, memory, core, sensory, and motor cortical regions.
#![doc = include_str!("../../docs/genomic.md")]

pub mod descriptors;
pub mod cortical_area;
pub mod brain_regions; // Made public for external access
mod sensory_cortical_unit;
mod motor_cortical_unit;

pub use sensory_cortical_unit::SensoryCorticalUnit;
pub use motor_cortical_unit::MotorCorticalUnit;
pub use brain_regions::{BrainRegion, RegionType};