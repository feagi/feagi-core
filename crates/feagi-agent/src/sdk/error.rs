// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! SDK error types

use thiserror::Error;

/// SDK-specific errors
#[derive(Error, Debug)]
pub enum SdkError {
    /// Core agent error
    #[error("Core agent error: {0}")]
    Core(#[from] crate::core::SdkError),

    /// Topology fetching failed
    #[error("Failed to fetch topology: {0}")]
    TopologyFetch(#[from] reqwest::Error),

    /// Encoding failed
    #[error("Encoding failed: {0}")]
    EncodingFailed(String),

    /// Decoding failed
    #[error("Decoding failed: {0}")]
    DecodingFailed(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Topology not found
    #[error("Topology not found for cortical ID: {0}")]
    TopologyNotFound(String),

    /// FEAGI data structure error
    #[error("FEAGI data structure error: {0}")]
    FeagiData(#[from] feagi_structures::FeagiDataError),

    /// Device registration export/sync failed
    #[error("Device registration sync failed: {0}")]
    DeviceRegistrationSyncFailed(String),

    /// Generic error
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

/// SDK result type
pub type Result<T> = std::result::Result<T, SdkError>;

