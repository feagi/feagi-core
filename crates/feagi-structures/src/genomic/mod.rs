//! Genomic types and identifiers for FEAGI.
//!
//! Provides core types for identifying and categorizing entities within the genome,
//! including custom, memory, core, sensory, and motor cortical regions.
#![doc = include_str!("../../docs/genomic.md")]

pub mod brain_regions; // Made public for external access
pub mod classifiers;
pub mod cortical_area;
pub mod descriptors;
pub mod modulator;
mod motor_cortical_unit;
mod sensory_cortical_unit;

pub use brain_regions::{BrainRegion, RegionType, ROOT_BRAIN_REGION_NAME};
pub use classifiers::{Classifier, ClassifierMapping};
pub use modulator::{
    modulator_signal, multiplicative_factor, scale_baseline, summed_reward,
    validate_instance_fields, validate_spike_train, ModulatorKind, ModulatorValidationError,
};
pub use motor_cortical_unit::MotorCorticalUnit;
pub use sensory_cortical_unit::SensoryCorticalUnit;
pub use sensory_cortical_unit::UnitTopology;
