//! Cortical units are collections of cortical_area areas (1 or more) corresponding to some motor or
//! sensor, allowing interpretation of agent sensor and motor data as neuron voxel activity

mod indexes;
mod motor_cortical_units;
mod sensor_cortical_units;

pub mod motor_cortical_unit;
pub mod sensor_cortical_unit;

pub use indexes::{CorticalUnitIndex, CorticalSubUnitIndex};
