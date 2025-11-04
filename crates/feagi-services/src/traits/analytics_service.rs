/*!
Analytics and monitoring service trait.

Defines the stable interface for statistics, metrics, and system health.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::types::*;
use async_trait::async_trait;

/// Analytics and monitoring service (transport-agnostic)
#[async_trait]
pub trait AnalyticsService: Send + Sync {
    /// Get system health status
    ///
    /// # Returns
    /// * `SystemHealth` - Current system health information
    ///
    async fn get_system_health(&self) -> ServiceResult<SystemHealth>;

    /// Get statistics for a cortical area
    ///
    /// # Arguments
    /// * `cortical_id` - Cortical area identifier
    ///
    /// # Returns
    /// * `CorticalAreaStats` - Statistics for the cortical area
    ///
    /// # Errors
    /// * `ServiceError::NotFound` - Cortical area not found
    ///
    async fn get_cortical_area_stats(
        &self,
        cortical_id: &str,
    ) -> ServiceResult<CorticalAreaStats>;

    /// Get statistics for all cortical areas
    ///
    /// # Returns
    /// * `Vec<CorticalAreaStats>` - Statistics for all cortical areas
    ///
    async fn get_all_cortical_area_stats(&self) -> ServiceResult<Vec<CorticalAreaStats>>;

    /// Get connectivity statistics between two cortical areas
    ///
    /// # Arguments
    /// * `source_area` - Source cortical area identifier
    /// * `target_area` - Target cortical area identifier
    ///
    /// # Returns
    /// * `ConnectivityStats` - Connectivity statistics
    ///
    /// # Errors
    /// * `ServiceError::NotFound` - One or both cortical areas not found
    ///
    async fn get_connectivity_stats(
        &self,
        source_area: &str,
        target_area: &str,
    ) -> ServiceResult<ConnectivityStats>;

    /// Get total neuron count across all cortical areas
    ///
    /// # Returns
    /// * `usize` - Total neuron count
    ///
    async fn get_total_neuron_count(&self) -> ServiceResult<usize>;

    /// Get total synapse count across all cortical areas
    ///
    /// # Returns
    /// * `usize` - Total synapse count
    ///
    async fn get_total_synapse_count(&self) -> ServiceResult<usize>;

    /// Get list of populated cortical areas (areas with neurons)
    ///
    /// # Returns
    /// * `Vec<(String, usize)>` - List of (cortical_id, neuron_count) for populated areas
    ///
    async fn get_populated_areas(&self) -> ServiceResult<Vec<(String, usize)>>;

    /// Get neuron density for a cortical area
    ///
    /// Density = neuron_count / total_voxels
    ///
    /// # Arguments
    /// * `cortical_id` - Cortical area identifier
    ///
    /// # Returns
    /// * `f32` - Neuron density (0.0 to 1.0)
    ///
    /// # Errors
    /// * `ServiceError::NotFound` - Cortical area not found
    ///
    async fn get_neuron_density(&self, cortical_id: &str) -> ServiceResult<f32>;

    /// Check if the brain is initialized (has cortical areas)
    ///
    /// # Returns
    /// * `bool` - True if brain is initialized
    ///
    async fn is_brain_initialized(&self) -> ServiceResult<bool>;

    /// Check if the burst engine is ready
    ///
    /// # Returns
    /// * `bool` - True if burst engine is active
    ///
    async fn is_burst_engine_ready(&self) -> ServiceResult<bool>;
}





