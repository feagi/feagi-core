// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Core types and error handling for FEAGI Evolution.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};

/// Result type for evolution operations
pub type EvoResult<T> = Result<T, EvoError>;

macro_rules! define_evo_error_key {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(FeagiErrorKey)]
        pub struct $name {
            context: &'static str,
            pub message: String,
        }
    };
}

define_evo_error_key!(FeagiEvoInvalidGenomeErrKey);
define_evo_error_key!(FeagiEvoValidationFailedErrKey);
define_evo_error_key!(FeagiEvoJsonErrKey);
define_evo_error_key!(FeagiEvoIoErrKey);
define_evo_error_key!(FeagiEvoInternalErrKey);
define_evo_error_key!(FeagiEvoInvalidAreaErrKey);
define_evo_error_key!(FeagiEvoInvalidRegionErrKey);

generate_feagi_error! {
    /// Error types for evolution operations
    EvoError,
    keys: {
        InvalidGenome: FeagiEvoInvalidGenomeErrKey,
        ValidationFailed: FeagiEvoValidationFailedErrKey,
        JsonError: FeagiEvoJsonErrKey,
        IoError: FeagiEvoIoErrKey,
        Internal: FeagiEvoInternalErrKey,
        InvalidArea: FeagiEvoInvalidAreaErrKey,
        InvalidRegion: FeagiEvoInvalidRegionErrKey,
    },
    sub_errors: {

    },
}

impl EvoError {
    pub fn invalid_genome(message: impl Into<String>) -> Self {
        FeagiEvoInvalidGenomeErrKey::new("Invalid genome", message.into()).into()
    }

    pub fn validation_failed(message: impl Into<String>) -> Self {
        FeagiEvoValidationFailedErrKey::new("Genome validation failed", message.into()).into()
    }

    pub fn json_error(message: impl Into<String>) -> Self {
        FeagiEvoJsonErrKey::new("JSON parsing error", message.into()).into()
    }

    pub fn io_error(message: impl Into<String>) -> Self {
        FeagiEvoIoErrKey::new("I/O error", message.into()).into()
    }

    pub fn internal(message: impl Into<String>) -> Self {
        FeagiEvoInternalErrKey::new("Internal error", message.into()).into()
    }

    pub fn invalid_area(message: impl Into<String>) -> Self {
        FeagiEvoInvalidAreaErrKey::new("Invalid cortical area", message.into()).into()
    }

    pub fn invalid_region(message: impl Into<String>) -> Self {
        FeagiEvoInvalidRegionErrKey::new("Invalid brain region", message.into()).into()
    }

    pub fn message(&self) -> &str {
        match self {
            EvoError::InvalidGenome(key) => &key.message,
            EvoError::ValidationFailed(key) => &key.message,
            EvoError::JsonError(key) => &key.message,
            EvoError::IoError(key) => &key.message,
            EvoError::Internal(key) => &key.message,
            EvoError::InvalidArea(key) => &key.message,
            EvoError::InvalidRegion(key) => &key.message,
        }
    }
}

// Convert from serde_json::Error
impl From<serde_json::Error> for EvoError {
    fn from(err: serde_json::Error) -> Self {
        EvoError::json_error(err.to_string())
    }
}

// Convert from std::io::Error
impl From<std::io::Error> for EvoError {
    fn from(err: std::io::Error) -> Self {
        EvoError::io_error(err.to_string())
    }
}

impl From<()> for EvoError {
    fn from(_: ()) -> Self {
        EvoError::internal("operation failed")
    }
}

