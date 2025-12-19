// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Service layer error types.

Transport-agnostic errors that can be mapped to HTTP status codes,
ZMQ error codes, or embedded error codes by adapters.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use thiserror::Error;

/// Service layer errors (transport-agnostic)
#[derive(Error, Debug, Clone)]
pub enum ServiceError {
    /// Resource not found (404 in HTTP, NOT_FOUND in ZMQ)
    #[error("Not found: {resource} with id '{id}'")]
    NotFound { resource: String, id: String },

    /// Invalid input parameters (400 in HTTP, BAD_REQUEST in ZMQ)
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Resource already exists (409 in HTTP, CONFLICT in ZMQ)
    #[error("Already exists: {resource} with id '{id}'")]
    AlreadyExists { resource: String, id: String },

    /// Operation not permitted (403 in HTTP, FORBIDDEN in ZMQ)
    #[error("Operation not permitted: {0}")]
    Forbidden(String),

    /// Internal service error (500 in HTTP, INTERNAL_ERROR in ZMQ)
    #[error("Internal error: {0}")]
    Internal(String),

    /// Backend error (BDU, NPU, Evo)
    #[error("Backend error: {0}")]
    Backend(String),

    /// State inconsistency
    #[error("State error: {0}")]
    StateError(String),

    /// Invalid state for operation (e.g., trying to pause when not running)
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Not yet implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

/// Result type for service operations
pub type ServiceResult<T> = Result<T, ServiceError>;

// ============================================================================
// ERROR CONVERSIONS FROM BACKEND
// ============================================================================

impl From<feagi_npu_neural::types::FeagiError> for ServiceError {
    fn from(err: feagi_npu_neural::types::FeagiError) -> Self {
        match err {
            feagi_npu_neural::types::FeagiError::CorticalAreaNotFound(_) => {
                ServiceError::NotFound {
                    resource: "CorticalArea".to_string(),
                    id: err.to_string(),
                }
            }
            feagi_npu_neural::types::FeagiError::InvalidArea(msg) => {
                ServiceError::InvalidInput(msg)
            }
            feagi_npu_neural::types::FeagiError::InvalidRegion(msg) => {
                ServiceError::InvalidInput(msg)
            }
            _ => ServiceError::Backend(err.to_string()),
        }
    }
}

impl From<feagi_data_structures::FeagiDataError> for ServiceError {
    fn from(err: feagi_data_structures::FeagiDataError) -> Self {
        ServiceError::InvalidInput(err.to_string())
    }
}

impl From<feagi_bdu::BduError> for ServiceError {
    fn from(err: feagi_bdu::BduError) -> Self {
        match err {
            feagi_bdu::BduError::InvalidArea(msg) => ServiceError::InvalidInput(msg),
            feagi_bdu::BduError::InvalidGenome(msg) => ServiceError::InvalidInput(msg),
            feagi_bdu::BduError::InvalidMorphology(msg) => ServiceError::InvalidInput(msg),
            _ => ServiceError::Backend(err.to_string()),
        }
    }
}

impl From<feagi_evolutionary::EvoError> for ServiceError {
    fn from(err: feagi_evolutionary::EvoError) -> Self {
        match err {
            feagi_evolutionary::EvoError::InvalidGenome(msg) => ServiceError::InvalidInput(msg),
            feagi_evolutionary::EvoError::InvalidArea(msg) => ServiceError::InvalidInput(msg),
            _ => ServiceError::Backend(err.to_string()),
        }
    }
}
