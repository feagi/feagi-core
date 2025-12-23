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
}

// Convert from feagi_npu_neural::types::FeagiError
impl From<feagi_npu_neural::types::FeagiError> for BduError {
    fn from(err: feagi_npu_neural::types::FeagiError) -> Self {
        match &err {
            feagi_npu_neural::types::FeagiError::InvalidArea(msg) => {
                BduError::InvalidArea(msg.clone())
            }
            feagi_npu_neural::types::FeagiError::InvalidRegion(msg) => {
                BduError::InvalidArea(msg.clone())
            }
            _ => BduError::Internal(err.to_string()),
        }
    }
}

// Convert from feagi_structures::FeagiDataError
impl From<feagi_structures::FeagiDataError> for BduError {
    fn from(err: feagi_structures::FeagiDataError) -> Self {
        BduError::Internal(err.to_string())
    }
}

// Convert from feagi_evolutionary::EvoError
impl From<feagi_evolutionary::EvoError> for BduError {
    fn from(err: feagi_evolutionary::EvoError) -> Self {
        match &err {
            feagi_evolutionary::EvoError::InvalidGenome(msg) => {
                BduError::InvalidGenome(msg.clone())
            }
            feagi_evolutionary::EvoError::InvalidArea(msg) => BduError::InvalidArea(msg.clone()),
            _ => BduError::Internal(err.to_string()),
        }
    }
}

// Note: Dimensions has been moved to feagi-types and is re-exported from feagi-bdu::lib
