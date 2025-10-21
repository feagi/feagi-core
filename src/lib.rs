//! # FEAGI Data Processing Library
//!
//! This crate provides comprehensive data structures and processing utilities for the FEAGI
//! (Framework for Evolutionary Artificial General Intelligence) system. It handles the core
//! data processing requirements for neural simulation, including neuron data management,
//! cortical area identification, serialization/deserialization, and brain input/output processing.


/// Centralized error handling for data processing operations.
pub mod error;
pub mod io_processing;


pub mod io_data;

pub mod neuron_data;
pub mod genomic_structures;
pub mod templates;

#[cfg(test)]
mod tests {
    // Tests of each module are in the mod file of each module, and are run from there
}

