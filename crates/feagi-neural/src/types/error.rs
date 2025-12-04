// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Error types for FEAGI operations

use core::fmt;
use super::ids::NeuronId;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::string::String;

/// Error types for FEAGI operations
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(Clone))]
pub enum FeagiError {
    #[cfg(feature = "std")]
    InvalidNeuronId(u32),

    #[cfg(feature = "std")]
    InvalidCorticalAreaId(u32),

    #[cfg(feature = "std")]
    InvalidSynapseId(u32),

    #[cfg(feature = "std")]
    NeuronNotFound(NeuronId),

    #[cfg(feature = "std")]
    CorticalAreaNotFound(u32),

    #[cfg(feature = "std")]
    ArraySizeMismatch { expected: usize, actual: usize },

    #[cfg(feature = "std")]
    ComputationError(String),

    #[cfg(feature = "std")]
    MemoryAllocationError(String),

    #[cfg(feature = "std")]
    InvalidBackend(String),
    
    #[cfg(feature = "std")]
    InvalidArea(String),
    
    #[cfg(feature = "std")]
    OutOfBounds {
        x: i32,
        y: i32,
        z: i32,
        width: usize,
        height: usize,
        depth: usize,
    },
    
    #[cfg(feature = "std")]
    InvalidRegion(String),
    
    #[cfg(feature = "std")]
    RegionNotFound(String),
    
    #[cfg(feature = "std")]
    CircularDependency(String),
    
    #[cfg(feature = "std")]
    RuntimeError(String),
    
    #[cfg(not(feature = "std"))]
    GenericError,
}

impl fmt::Display for FeagiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "std")]
        match self {
            FeagiError::InvalidNeuronId(id) => write!(f, "Invalid neuron ID: {}", id),
            FeagiError::InvalidCorticalAreaId(id) => write!(f, "Invalid cortical area ID: {}", id),
            FeagiError::InvalidSynapseId(id) => write!(f, "Invalid synapse ID: {}", id),
            FeagiError::NeuronNotFound(id) => write!(f, "Neuron not found: {}", id),
            FeagiError::CorticalAreaNotFound(id) => write!(f, "Cortical area not found: {}", id),
            FeagiError::ArraySizeMismatch { expected, actual } => {
                write!(f, "Array size mismatch: expected {}, got {}", expected, actual)
            }
            FeagiError::ComputationError(msg) => write!(f, "Computation error: {}", msg),
            FeagiError::MemoryAllocationError(msg) => write!(f, "Memory allocation error: {}", msg),
            FeagiError::InvalidBackend(msg) => write!(f, "Invalid backend: {}", msg),
            FeagiError::InvalidArea(msg) => write!(f, "Invalid cortical area: {}", msg),
            FeagiError::OutOfBounds { x, y, z, width, height, depth } => {
                write!(
                    f,
                    "Out of bounds: position ({}, {}, {}) exceeds dimensions ({}, {}, {})",
                    x, y, z, width, height, depth
                )
            }
            FeagiError::InvalidRegion(msg) => write!(f, "Invalid brain region: {}", msg),
            FeagiError::RegionNotFound(msg) => write!(f, "Region not found: {}", msg),
            FeagiError::CircularDependency(msg) => write!(f, "Circular dependency detected: {}", msg),
            FeagiError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
        }
        
        #[cfg(not(feature = "std"))]
        match self {
            FeagiError::GenericError => write!(f, "FEAGI error"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for FeagiError {}

pub type Result<T> = core::result::Result<T, FeagiError>;
pub type Error = FeagiError;

