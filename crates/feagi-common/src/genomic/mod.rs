//! Genomic types and identifiers for FEAGI.
//!
//! Provides core types for identifying and categorizing entities within the genome,
//! Note that not all structs are exposed in no-std / no alloc situations

#[cfg(feature = "alloc")]
pub mod brain_regions;

pub mod cortical_area;
mod motor_cortical_unit;
mod sensory_cortical_unit;
mod feagi_genome_error;
pub mod descriptors;

pub use feagi_genome_error::FeagiStructuresGenomicError;
pub use motor_cortical_unit::MotorCorticalUnit;
pub use sensory_cortical_unit::SensoryCorticalUnit;

