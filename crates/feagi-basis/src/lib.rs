//! This crate holds subcrates of common data types and directly holds some common generic
//! structs used around FEAGI

pub mod prelude {
    pub use super::feagi_quantization::prelude::*;
    pub use super::feagi_collections::prelude::*;
    pub use super::feagi_basis_error::FeagiBasisError;
    pub use super::burst_index::BurstIndex;
}

pub extern crate feagi_basis_quantization as feagi_quantization;
pub extern crate feagi_basis_collections as feagi_collections;
pub extern crate feagi_basis_threading as feagi_threading;

pub mod generic_collections;

pub use feagi_basis_error::FeagiBasisError;

pub use burst_index::{BurstIndex, BurstIndexEnum};
mod feagi_basis_error;
mod burst_index;
pub mod misc;

// TODO future UI work, may need its own crate? or maybe not
//pub mod ui_parameters;

