//! This crate holds subcrates of common data types and directly holds some common generic
//! structs used around FEAGI

pub extern crate feagi_data_quantization;

pub extern crate feagi_data_neuron;

/// Generic traits and implementations for sending data between threads
pub mod data_messaging;

pub mod feagi_data_error; // TODO This error is very generic, we should break it apart

pub mod generic_collections;

// TODO future UI work, may need its own crate? or maybe not
//pub mod ui_parameters;

