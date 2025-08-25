//! # FEAGI Data Processing Library
//!
//! This crate provides comprehensive data structures and processing utilities for the FEAGI
//! (Framework for Evolutionary Artificial General Intelligence) system. It handles the core
//! data processing requirements for neural simulation, including neuron data management,
//! cortical area identification, serialization/deserialization, and brain input/output processing.
//!
//! ## Module Overview
//!
//! - [`byte_structures`]: Serialization and deserialization framework for FEAGI data formats
//! - [`brain_input`]: Processing modules for sensory input data (primarily vision systems)
//! - [`brain_output`]: Processing modules for motor output data
//! - [`error`]: Centralized error handling for all data processing operations
//! - [`neuron_data`]: Core data structures for managing neuron information and properties
//! - [`cortical_data`]: Cortical area identification and management utilities


/// Centralized error handling for data processing operations.
pub mod error;
pub mod io_processing;


pub mod io_data;

pub mod neuron_data;
pub mod genomic_structures;
pub mod templates;
mod feagi_json;

#[cfg(test)]
mod tests {
    // Tests of each module are in the mod file of each module, and are run from there
}

