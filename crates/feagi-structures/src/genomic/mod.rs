//! Genomic types and identifiers for FEAGI.
//!
//! Provides core types for identifying and categorizing entities within the genome,
//! including custom, memory, core, sensory, and motor cortical regions.
#![doc = include_str!("../../docs/genomic.md")]


//mod descriptors; // DO NOT EXPOSE DIRECTLY! Macros here generate types we do not want used!

pub mod brain_regions; // Made public for external access
pub mod cortical_area;
mod motor_cortical_unit;
mod sensory_cortical_unit;
mod feagi_genome_error;

pub use brain_regions::{BrainRegion, RegionType};
pub use motor_cortical_unit::MotorCorticalUnit;
pub use sensory_cortical_unit::SensoryCorticalUnit;

// SPECIFICALLY ONLY EXPOSE I32 VARIANTS!
pub use descriptors::{GenomeCoordinate2DI32, GenomeCoordinate3DI32};
