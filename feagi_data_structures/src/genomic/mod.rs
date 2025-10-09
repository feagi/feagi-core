//! Genomic types and identifiers for FEAGI.
//!
//! Provides core types for identifying and categorizing entities within the genome,
//! including custom, memory, core, sensory, and motor cortical regions.
#![doc = include_str!("../../docs/genomic.md")]

mod cortical_id;
mod cortical_type;
pub mod descriptors;

pub use cortical_id::CorticalID;
pub use cortical_type::{CorticalType, CoreCorticalType, SensorCorticalType, MotorCorticalType};