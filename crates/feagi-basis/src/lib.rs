//! This crate holds subcrates of common data types and directly holds some common generic
//! structs used around FEAGI

pub extern crate feagi_basis_error_logging as feagi_error;

pub extern crate feagi_basis_quantization as feagi_quantization;

pub extern crate feagi_basis_genome as feagi_genome;

pub extern crate feagi_basis_neuron as feagi_neuron;

/// Generic traits and implementations for sending data between threads
pub mod thread_messaging;

pub mod generic_collections;

pub use feagi_basis_error::{FeagiBasisError, FeagiFailDataEtc};

mod feagi_basis_error; 

// TODO future UI work, may need its own crate? or maybe not
//pub mod ui_parameters;

