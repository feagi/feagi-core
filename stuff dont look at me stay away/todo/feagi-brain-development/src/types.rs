// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Core types for BDU operations.

These types match the Python API for seamless integration.
*/

/// Cortical area identifier (6-character string in Python)
pub type AreaId = String;

/// 3D position (x, y, z)
pub type Position = (u32, u32, u32);

/// Synaptic weight (0-255 in u8, converted from Python float)
pub type Weight = u8;

/// Result type for BDU operations
pub type BduResult<T> = Result<T, BduError>;

/// Errors that can occur during BDU operations
#[derive(Debug, thiserror::Error)]
pub enum BduError {
    #[error("Invalid area: {0}")]
    InvalidArea(String),

    #[error("Invalid morphology: {0}")]
    InvalidMorphology(String),

    #[error("Invalid position: {0:?}")]
    InvalidPosition(Position),

    #[error("Dimension mismatch: expected {expected:?}, got {actual:?}")]
    DimensionMismatch {
        expected: (usize, usize, usize),
        actual: (usize, usize, usize),
    },

    #[error("Out of bounds: position {pos:?} not in dimensions {dims:?}")]
    OutOfBounds {
        pos: Position,
        dims: (usize, usize, usize),
    },

    #[error("Invalid genome: {0}")]
    InvalidGenome(String),

    #[error("Invalid neuron: {0}")]
    InvalidNeuron(String),

    #[error("Invalid synapse: {0}")]
    InvalidSynapse(String),

    #[error("Internal error: {0}")]
    Internal(String),

    /// Capability the WNPU does not expose yet during the NPU transition.
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Region IO designation policy violation (cross-region mapping vs declared interface)
    #[error("Region IO policy violation: {0}")]
    RegionIoPolicyViolation(String),
}

// Convert from feagi_evolutionary::EvoError
impl From<feagi_evolutionary::EvoError> for BduError {
    fn from(err: feagi_evolutionary::EvoError) -> Self {
        match &err {
            feagi_evolutionary::EvoError::InvalidGenome(_) => {
                BduError::InvalidGenome(err.message().to_string())
            }
            feagi_evolutionary::EvoError::InvalidArea(_) => {
                BduError::InvalidArea(err.message().to_string())
            }
            _ => BduError::Internal(err.to_string()),
        }
    }
}

// Note: Dimensions has been moved to feagi-types and is re-exported from feagi-bdu::lib
