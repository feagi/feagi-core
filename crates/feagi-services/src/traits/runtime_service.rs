/*!
Runtime control service trait.

Defines the stable interface for controlling the FEAGI burst engine runtime.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::types::*;
use async_trait::async_trait;

/// Runtime control service (transport-agnostic)
#[async_trait]
pub trait RuntimeService: Send + Sync {
    /// Start the burst engine
    ///
    /// Begins executing neural bursts at the configured frequency.
    ///
    /// # Errors
    /// * `ServiceError::InvalidState` - Already running
    /// * `ServiceError::Backend` - Failed to start burst engine
    ///
    async fn start(&self) -> ServiceResult<()>;

    /// Stop the burst engine
    ///
    /// Gracefully stops burst execution.
    ///
    /// # Errors
    /// * `ServiceError::Backend` - Failed to stop burst engine
    ///
    async fn stop(&self) -> ServiceResult<()>;

    /// Pause the burst engine
    ///
    /// Temporarily pauses burst execution without stopping the thread.
    ///
    /// # Errors
    /// * `ServiceError::InvalidState` - Not running
    /// * `ServiceError::Backend` - Failed to pause
    ///
    async fn pause(&self) -> ServiceResult<()>;

    /// Resume the burst engine
    ///
    /// Resumes burst execution after pause.
    ///
    /// # Errors
    /// * `ServiceError::InvalidState` - Not paused
    /// * `ServiceError::Backend` - Failed to resume
    ///
    async fn resume(&self) -> ServiceResult<()>;

    /// Execute a single burst step
    ///
    /// Executes one burst cycle and then pauses.
    /// Useful for debugging and step-by-step execution.
    ///
    /// # Errors
    /// * `ServiceError::InvalidState` - Already running in continuous mode
    /// * `ServiceError::Backend` - Failed to execute step
    ///
    async fn step(&self) -> ServiceResult<()>;

    /// Get runtime status
    ///
    /// Returns the current state of the burst engine.
    ///
    /// # Returns
    /// * `RuntimeStatus` - Current runtime status
    ///
    async fn get_status(&self) -> ServiceResult<RuntimeStatus>;

    /// Set burst frequency
    ///
    /// Changes the burst execution frequency (Hz).
    ///
    /// # Arguments
    /// * `frequency_hz` - New frequency in Hz (e.g., 30.0)
    ///
    /// # Errors
    /// * `ServiceError::InvalidInput` - Invalid frequency (must be > 0)
    ///
    async fn set_frequency(&self, frequency_hz: f64) -> ServiceResult<()>;

    /// Get current burst count
    ///
    /// Returns the total number of bursts executed since start.
    ///
    /// # Returns
    /// * `u64` - Total burst count
    ///
    async fn get_burst_count(&self) -> ServiceResult<u64>;

    /// Reset burst count
    ///
    /// Resets the burst counter to zero.
    ///
    async fn reset_burst_count(&self) -> ServiceResult<()>;
}

