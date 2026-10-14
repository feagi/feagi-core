//! Common data types and generic structs used around FEAGI.

pub extern crate feagi_basis_proc_common;

pub mod prelude {
    pub use super::burst_index::BurstIndex;
    pub use super::collections::prelude::*;
    pub use super::feagi_basis_error::FeagiBasisError;
    pub use super::quantization::prelude::*;
}

pub mod quantization;
pub mod collections;
pub mod parallelism;

pub use feagi_basis_error::FeagiBasisError;

pub use burst_index::{BurstIndex, BurstIndexEnum};
mod feagi_basis_error;
mod burst_index;
pub mod misc;
pub mod neurons;
pub mod voxels;
pub mod feagi_message;
pub mod capabilities;
pub mod identifiers;
// TODO future UI work, may need its own crate? or maybe not
//pub mod ui_parameters;

