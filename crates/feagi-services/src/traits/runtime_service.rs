// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

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

    /// Get FCL (Fire Candidate List) snapshot for monitoring
    ///
    /// Returns vector of (neuron_id, potential) pairs from last burst
    ///
    /// # Returns
    /// * `Vec<(u64, f32)>` - Neuron IDs and their membrane potentials
    ///
    async fn get_fcl_snapshot(&self) -> ServiceResult<Vec<(u64, f32)>>;

    /// Get Fire Candidate List snapshot with cortical area information
    ///
    /// Returns the last FCL snapshot with cortical_idx for each neuron.
    /// This avoids the need to query cortical_area for each neuron separately.
    ///
    /// # Returns
    /// * `Vec<(u64, u32, f32)>` - (neuron_id, cortical_idx, membrane_potential) tuples
    ///
    async fn get_fcl_snapshot_with_cortical_idx(&self) -> ServiceResult<Vec<(u64, u32, f32)>>;

    /// Get Fire Queue sample for monitoring
    ///
    /// Returns neurons that actually fired in the last burst, organized by cortical area
    ///
    /// # Returns
    /// * `HashMap<u32, (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<f32>)>` - Area data
    ///
    async fn get_fire_queue_sample(
        &self,
    ) -> ServiceResult<
        std::collections::HashMap<u32, (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<f32>)>,
    >;

    /// Get Fire Ledger window configurations for all cortical areas
    ///
    /// # Returns
    /// * `Vec<(u32, usize)>` - (cortical_idx, window_size) pairs
    ///
    async fn get_fire_ledger_configs(&self) -> ServiceResult<Vec<(u32, usize)>>;

    /// Configure Fire Ledger window size for a cortical area
    ///
    /// # Arguments
    /// * `cortical_idx` - Cortical area index
    /// * `window_size` - Number of bursts to retain in history
    ///
    async fn configure_fire_ledger_window(
        &self,
        cortical_idx: u32,
        window_size: usize,
    ) -> ServiceResult<()>;

    /// Get FCL/FQ sampler configuration
    ///
    /// # Returns
    /// * `(f64, u32)` - (frequency_hz, consumer_type) where consumer: 1=viz, 2=motor, 3=both
    ///
    async fn get_fcl_sampler_config(&self) -> ServiceResult<(f64, u32)>;

    /// Set FCL/FQ sampler configuration
    ///
    /// # Arguments
    /// * `frequency` - Optional sampling frequency in Hz
    /// * `consumer` - Optional consumer type (1=viz, 2=motor, 3=both)
    ///
    async fn set_fcl_sampler_config(
        &self,
        frequency: Option<f64>,
        consumer: Option<u32>,
    ) -> ServiceResult<()>;

    /// Get FCL sample rate for a specific cortical area
    ///
    /// # Arguments
    /// * `area_id` - Cortical area ID (cortical_idx)
    ///
    /// # Returns
    /// * `f64` - Sample rate in Hz
    ///
    async fn get_area_fcl_sample_rate(&self, area_id: u32) -> ServiceResult<f64>;

    /// Set FCL sample rate for a specific cortical area
    ///
    /// # Arguments
    /// * `area_id` - Cortical area ID (cortical_idx)
    /// * `sample_rate` - Sample rate in Hz
    ///
    async fn set_area_fcl_sample_rate(&self, area_id: u32, sample_rate: f64) -> ServiceResult<()>;

    /// Inject sensory data by cortical area ID and coordinates
    ///
    /// Takes cortical ID (base64 string) and coordinates with potential values,
    /// converts coordinates to neuron IDs, and injects them into FCL.
    ///
    /// # Arguments
    /// * `cortical_id` - Base64 encoded cortical area ID
    /// * `xyzp_data` - Vector of (x, y, z, potential) tuples
    ///
    /// # Returns
    /// * Number of neurons successfully injected
    ///
    /// # Errors
    /// * `ServiceError::NotFound` - Cortical area not found
    /// * `ServiceError::InvalidInput` - Invalid cortical ID format
    ///
    async fn inject_sensory_by_coordinates(
        &self,
        cortical_id: &str,
        xyzp_data: &[(u32, u32, u32, f32)],
    ) -> ServiceResult<usize>;
}
