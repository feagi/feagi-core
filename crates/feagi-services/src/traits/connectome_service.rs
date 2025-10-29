/*!
Connectome management service trait.

Defines the stable interface for cortical area and brain region operations.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::types::*;
use async_trait::async_trait;

/// Connectome management service (transport-agnostic)
#[async_trait]
pub trait ConnectomeService: Send + Sync {
    // ========================================================================
    // CORTICAL AREA OPERATIONS
    // ========================================================================

    /// Create a cortical area
    ///
    /// # Arguments
    /// * `params` - Cortical area creation parameters
    ///
    /// # Returns
    /// * `CorticalAreaInfo` - Information about the created area
    ///
    /// # Errors
    /// * `ServiceError::AlreadyExists` - Cortical area ID already exists
    /// * `ServiceError::InvalidInput` - Invalid parameters
    ///
    async fn create_cortical_area(
        &self,
        params: CreateCorticalAreaParams,
    ) -> ServiceResult<CorticalAreaInfo>;

    /// Delete a cortical area
    ///
    /// # Arguments
    /// * `cortical_id` - Cortical area identifier
    ///
    /// # Errors
    /// * `ServiceError::NotFound` - Cortical area not found
    ///
    async fn delete_cortical_area(&self, cortical_id: &str) -> ServiceResult<()>;

    /// Get cortical area information
    ///
    /// # Arguments
    /// * `cortical_id` - Cortical area identifier
    ///
    /// # Returns
    /// * `CorticalAreaInfo` - Information about the cortical area
    ///
    /// # Errors
    /// * `ServiceError::NotFound` - Cortical area not found
    ///
    async fn get_cortical_area(&self, cortical_id: &str) -> ServiceResult<CorticalAreaInfo>;

    /// List all cortical areas
    ///
    /// # Returns
    /// * `Vec<CorticalAreaInfo>` - List of all cortical areas
    ///
    async fn list_cortical_areas(&self) -> ServiceResult<Vec<CorticalAreaInfo>>;

    /// Get cortical area IDs
    ///
    /// # Returns
    /// * `Vec<String>` - List of cortical area IDs
    ///
    async fn get_cortical_area_ids(&self) -> ServiceResult<Vec<String>>;

    /// Check if a cortical area exists
    ///
    /// # Arguments
    /// * `cortical_id` - Cortical area identifier
    ///
    /// # Returns
    /// * `bool` - True if cortical area exists
    ///
    async fn cortical_area_exists(&self, cortical_id: &str) -> ServiceResult<bool>;

    // ========================================================================
    // BRAIN REGION OPERATIONS
    // ========================================================================

    /// Create a brain region
    ///
    /// # Arguments
    /// * `params` - Brain region creation parameters
    ///
    /// # Returns
    /// * `BrainRegionInfo` - Information about the created region
    ///
    /// # Errors
    /// * `ServiceError::AlreadyExists` - Brain region ID already exists
    /// * `ServiceError::InvalidInput` - Invalid parameters
    /// * `ServiceError::NotFound` - Parent region not found
    ///
    async fn create_brain_region(
        &self,
        params: CreateBrainRegionParams,
    ) -> ServiceResult<BrainRegionInfo>;

    /// Delete a brain region
    ///
    /// # Arguments
    /// * `region_id` - Brain region identifier
    ///
    /// # Errors
    /// * `ServiceError::NotFound` - Brain region not found
    ///
    async fn delete_brain_region(&self, region_id: &str) -> ServiceResult<()>;

    /// Get brain region information
    ///
    /// # Arguments
    /// * `region_id` - Brain region identifier
    ///
    /// # Returns
    /// * `BrainRegionInfo` - Information about the brain region
    ///
    /// # Errors
    /// * `ServiceError::NotFound` - Brain region not found
    ///
    async fn get_brain_region(&self, region_id: &str) -> ServiceResult<BrainRegionInfo>;

    /// List all brain regions
    ///
    /// # Returns
    /// * `Vec<BrainRegionInfo>` - List of all brain regions
    ///
    async fn list_brain_regions(&self) -> ServiceResult<Vec<BrainRegionInfo>>;

    /// Get brain region IDs
    ///
    /// # Returns
    /// * `Vec<String>` - List of brain region IDs
    ///
    async fn get_brain_region_ids(&self) -> ServiceResult<Vec<String>>;

    /// Check if a brain region exists
    ///
    /// # Arguments
    /// * `region_id` - Brain region identifier
    ///
    /// # Returns
    /// * `bool` - True if brain region exists
    ///
    async fn brain_region_exists(&self, region_id: &str) -> ServiceResult<bool>;
}

