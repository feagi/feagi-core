//! Common data types and generic structs used around FEAGI.

pub extern crate feagi_basis_proc_common;

pub mod prelude {
    pub use super::burst_index::BurstIndex;
    pub use super::collections::prelude::*;
    pub use super::feagi_basis_error::FeagiBasisError;
    pub use super::quantization::prelude::*;
}

pub mod collections;
pub mod parallelism;
pub mod quantization;

pub use feagi_basis_error::FeagiBasisError;

pub use burst_index::{BurstIndex, BurstIndexEnum};
mod burst_index;
pub mod capabilities;
mod feagi_basis_error;
pub mod feagi_message;
pub mod identifiers;
pub mod misc;
pub mod neurons;
pub mod voxels;
// TODO future UI work, may need its own crate? or maybe not
//pub mod ui_parameters;
