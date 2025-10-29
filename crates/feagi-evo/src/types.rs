/*!
Core types and error handling for FEAGI Evolution.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use thiserror::Error;

/// Result type for evolution operations
pub type EvoResult<T> = Result<T, EvoError>;

/// Error types for evolution operations
#[derive(Error, Debug)]
pub enum EvoError {
    #[error("Invalid genome: {0}")]
    InvalidGenome(String),
    
    #[error("Genome validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("JSON parsing error: {0}")]
    JsonError(String),
    
    #[error("I/O error: {0}")]
    IoError(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Invalid cortical area: {0}")]
    InvalidArea(String),
    
    #[error("Invalid brain region: {0}")]
    InvalidRegion(String),
}

// Convert from serde_json::Error
impl From<serde_json::Error> for EvoError {
    fn from(err: serde_json::Error) -> Self {
        EvoError::JsonError(err.to_string())
    }
}

// Convert from std::io::Error
impl From<std::io::Error> for EvoError {
    fn from(err: std::io::Error) -> Self {
        EvoError::IoError(err.to_string())
    }
}

// Convert from feagi_types::FeagiError
impl From<feagi_types::FeagiError> for EvoError {
    fn from(err: feagi_types::FeagiError) -> Self {
        match &err {
            feagi_types::FeagiError::InvalidArea(msg) => EvoError::InvalidArea(msg.clone()),
            feagi_types::FeagiError::InvalidRegion(msg) => EvoError::InvalidRegion(msg.clone()),
            _ => EvoError::Internal(err.to_string()),
        }
    }
}

