// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Connectome management service trait.

Defines the stable interface for cortical area and brain region operations.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::types::*;
use async_trait::async_trait;
use std::collections::HashMap;

/// Connectome management service (transport-agnostic)
#[async_trait]
pub trait ConnectomeService: Send + Sync {
    // ========================================================================
    // CORTICAL AREA OPERATIONS
    // ========================================================================

    /// Create a cortical area
    ///
    /// ⚠️ INTERNAL USE ONLY - DO NOT CALL FROM API ENDPOINTS ⚠️
    ///
    /// ARCHITECTURE VIOLATION WARNING:
    /// This method directly modifies ConnectomeManager and bypasses:
    /// - Runtime genome updates (source of truth)
    /// - Neuroembryogenesis process
    /// - Neuron and synapse creation
    ///
    /// PROPER FLOW: API endpoints MUST use GenomeService::create_cortical_areas()
    /// which follows: genome update → neuroembryogenesis → connectome → NPU
    ///
    /// This method should ONLY be called by:
    /// - Neuroembryogenesis (during genome loading)
    /// - GenomeService internal implementation
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

    /// Update a cortical area
    ///
    /// ⚠️ DEPRECATED - Use GenomeService::update_cortical_area() instead ⚠️
    ///
    /// This method is blocked and returns NotImplemented.
    /// All cortical area updates MUST go through GenomeService for proper
    /// genome synchronization and change classification.
    ///
    /// # Arguments
    /// * `cortical_id` - Cortical area identifier
    /// * `params` - Update parameters
    ///
    /// # Errors
    /// * `ServiceError::NotFound` - Cortical area not found
    /// * `ServiceError::InvalidInput` - Invalid parameters
    /// * `ServiceError::NotImplemented` - Always returned (use GenomeService)
    ///
    async fn update_cortical_area(
        &self,
        cortical_id: &str,
        params: UpdateCorticalAreaParams,
    ) -> ServiceResult<CorticalAreaInfo>;

    /// Delete a cortical area
    ///
    /// ⚠️ ARCHITECTURE WARNING - Should use GenomeService ⚠️
    ///
    /// This method directly modifies ConnectomeManager without:
    /// - Updating runtime genome (source of truth)
    /// - Proper cleanup of neurons/synapses
    ///
    /// Consider adding GenomeService::delete_cortical_areas() for proper flow.
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

    /// Get cortical area properties as a flat HashMap
    ///
    /// # Arguments
    /// * `cortical_id` - Cortical area identifier
    ///
    /// # Returns
    /// * `HashMap<String, serde_json::Value>` - Flattened properties of the cortical area
    ///
    /// # Errors
    /// * `ServiceError::NotFound` - Cortical area not found
    ///
    async fn get_cortical_area_properties(
        &self,
        cortical_id: &str,
    ) -> ServiceResult<std::collections::HashMap<String, serde_json::Value>>;

    /// Get properties of all cortical areas
    ///
    /// # Returns
    /// * `Vec<HashMap<String, serde_json::Value>>` - List of property maps for all areas
    ///
    async fn get_all_cortical_area_properties(
        &self,
    ) -> ServiceResult<Vec<std::collections::HashMap<String, serde_json::Value>>>;

    // ========================================================================
    // NEURON INSPECTION (DEBUG/INTROSPECTION)
    // ========================================================================

    /// Get a neuron's live properties/state snapshot (for visualization/debugging).
    ///
    /// This surfaces runtime state like membrane potential, threshold, refractory countdown,
    /// and consecutive fire tracking.
    async fn get_neuron_properties(
        &self,
        neuron_id: u64,
    ) -> ServiceResult<HashMap<String, serde_json::Value>>;

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

    /// Update a brain region
    ///
    /// # Arguments
    /// * `region_id` - Brain region identifier
    /// * `properties` - Map of property names to new values
    ///
    /// # Returns
    /// * `BrainRegionInfo` - Updated brain region information
    ///
    /// # Errors
    /// * `ServiceError::NotFound` - Brain region not found
    /// * `ServiceError::InvalidInput` - Invalid parameters
    ///
    async fn update_brain_region(
        &self,
        region_id: &str,
        properties: std::collections::HashMap<String, serde_json::Value>,
    ) -> ServiceResult<BrainRegionInfo>;

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

    // ========================================================================
    // MORPHOLOGY OPERATIONS
    // ========================================================================

    /// Get all morphologies from the loaded genome
    ///
    /// # Returns
    /// * `HashMap<String, MorphologyInfo>` - All morphology definitions
    ///
    async fn get_morphologies(
        &self,
    ) -> ServiceResult<std::collections::HashMap<String, MorphologyInfo>>;

    /// Create a new morphology definition.
    ///
    /// This mutates the currently loaded RuntimeGenome (source of truth) and the in-memory
    /// morphology registry used by ConnectomeManager.
    ///
    /// # Errors
    /// * `ServiceError::InvalidState` - No genome loaded
    /// * `ServiceError::AlreadyExists` - Morphology already exists
    /// * `ServiceError::InvalidInput` - Invalid morphology payload
    async fn create_morphology(
        &self,
        morphology_id: String,
        morphology: feagi_evolutionary::Morphology,
    ) -> ServiceResult<()>;

    /// Update an existing morphology definition.
    ///
    /// This mutates the currently loaded RuntimeGenome (source of truth) and the in-memory
    /// morphology registry used by ConnectomeManager.
    ///
    /// # Errors
    /// * `ServiceError::InvalidState` - No genome loaded
    /// * `ServiceError::NotFound` - Morphology does not exist
    /// * `ServiceError::InvalidInput` - Invalid morphology payload
    async fn update_morphology(
        &self,
        morphology_id: String,
        morphology: feagi_evolutionary::Morphology,
    ) -> ServiceResult<()>;

    /// Delete a morphology definition by ID.
    ///
    /// This mutates the currently loaded RuntimeGenome (source of truth) and the in-memory
    /// morphology registry used by ConnectomeManager.
    ///
    /// # Errors
    /// * `ServiceError::InvalidState` - No genome loaded
    /// * `ServiceError::NotFound` - Morphology does not exist
    async fn delete_morphology(&self, morphology_id: &str) -> ServiceResult<()>;

    // ========================================================================
    // CORTICAL MAPPING OPERATIONS
    // ========================================================================

    /// Update cortical mapping between two cortical areas
    ///
    /// # Arguments
    /// * `src_area_id` - Source cortical area ID
    /// * `dst_area_id` - Destination cortical area ID
    /// * `mapping_data` - List of connection specifications
    ///
    /// # Returns
    /// * `usize` - Number of synapses created
    ///
    /// # Errors
    /// * `ServiceError::NotFound` - Source or destination area not found
    /// * `ServiceError::InvalidInput` - Invalid mapping data
    ///
    async fn update_cortical_mapping(
        &self,
        src_area_id: String,
        dst_area_id: String,
        mapping_data: Vec<serde_json::Value>,
    ) -> ServiceResult<usize>;

    // ========================================================================
    // CONNECTOME I/O OPERATIONS
    // ========================================================================

    /// Export the current connectome as a snapshot
    ///
    /// Captures the complete state of the NPU including all neurons, synapses,
    /// and runtime state. This is the service layer interface for connectome export.
    ///
    /// # Returns
    /// * `ConnectomeSnapshot` - Complete connectome state snapshot
    ///
    /// # Errors
    /// * `ServiceError::Backend` - Failed to export connectome from NPU
    /// * `ServiceError::NotImplemented` - Connectome I/O not available (feature disabled)
    ///
    #[cfg(feature = "connectome-io")]
    async fn export_connectome(
        &self,
    ) -> ServiceResult<feagi_npu_neural::types::connectome::ConnectomeSnapshot>;

    /// Import a connectome snapshot
    ///
    /// Replaces the entire NPU state with data from a saved connectome.
    /// This is the service layer interface for connectome import.
    ///
    /// # Arguments
    /// * `snapshot` - The connectome snapshot to import
    ///
    /// # Errors
    /// * `ServiceError::Backend` - Failed to import connectome into NPU
    /// * `ServiceError::InvalidInput` - Invalid snapshot data
    /// * `ServiceError::NotImplemented` - Connectome I/O not available (feature disabled)
    ///
    #[cfg(feature = "connectome-io")]
    async fn import_connectome(
        &self,
        snapshot: feagi_npu_neural::types::connectome::ConnectomeSnapshot,
    ) -> ServiceResult<()>;
}
