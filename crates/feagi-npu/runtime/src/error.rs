// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Error types for runtime operations

use core::fmt;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::string::String;

/// Runtime errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    /// Capacity exceeded
    CapacityExceeded {
        /// Requested capacity
        requested: usize,
        /// Available capacity
        available: usize,
    },
    
    /// Out of memory
    OutOfMemory {
        /// Requested bytes
        requested_bytes: usize,
    },
    
    /// Invalid parameters provided
    #[cfg(feature = "std")]
    InvalidParameters(String),
    
    /// Invalid operation (only available with std feature)
    #[cfg(feature = "std")]
    InvalidOperation(String),
    
    /// Platform not supported (only available with std feature)
    #[cfg(feature = "std")]
    PlatformNotSupported(String),
    
    /// Storage error (only available with std feature)
    #[cfg(feature = "std")]
    StorageError(String),
    
    /// Generic error (for no_std environments)
    #[cfg(not(feature = "std"))]
    GenericError,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::CapacityExceeded { requested, available } => {
                write!(
                    f,
                    "Capacity exceeded: requested {}, available {}",
                    requested, available
                )
            }
            RuntimeError::OutOfMemory { requested_bytes } => {
                write!(f, "Out of memory: requested {} bytes", requested_bytes)
            }
            #[cfg(feature = "std")]
            RuntimeError::InvalidParameters(msg) => {
                write!(f, "Invalid parameters: {}", msg)
            }
            #[cfg(feature = "std")]
            RuntimeError::InvalidOperation(msg) => {
                write!(f, "Invalid operation: {}", msg)
            }
            #[cfg(feature = "std")]
            RuntimeError::PlatformNotSupported(platform) => {
                write!(f, "Platform not supported: {}", platform)
            }
            #[cfg(feature = "std")]
            RuntimeError::StorageError(msg) => {
                write!(f, "Storage error: {}", msg)
            }
            #[cfg(not(feature = "std"))]
            RuntimeError::GenericError => {
                write!(f, "Runtime error")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RuntimeError {}

/// Result type for runtime operations
pub type Result<T> = core::result::Result<T, RuntimeError>;

