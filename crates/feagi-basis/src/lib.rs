//! Common data types and generic structs used around FEAGI.

pub mod prelude {
    pub use super::quantization::prelude::*;
    pub use super::collections::prelude::*;
    pub use super::feagi_basis_error::FeagiBasisError;
    pub use super::burst_index::BurstIndex;
}

pub mod quantization;
pub mod collections;
pub mod threading;

pub use feagi_basis_error::FeagiBasisError;

pub use burst_index::{BurstIndex, BurstIndexEnum};
mod feagi_basis_error;
mod burst_index;
pub mod misc;
pub mod channel;
pub mod neurons;
pub mod voxels;
// TODO future UI work, may need its own crate? or maybe not
//pub mod ui_parameters;

