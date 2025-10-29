/*!
Genome management service trait.

Defines the stable interface for genome operations (genotype I/O).

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::types::*;
use async_trait::async_trait;

/// Genome management service (transport-agnostic)
#[async_trait]
pub trait GenomeService: Send + Sync {
    /// Load a genome from JSON
    ///
    /// Parses genome JSON and creates cortical areas and brain regions in the connectome.
    ///
    /// # Arguments
    /// * `params` - Load parameters (JSON string)
    ///
    /// # Returns
    /// * `GenomeInfo` - Metadata about the loaded genome
    ///
    /// # Errors
    /// * `ServiceError::InvalidInput` - Invalid JSON or malformed genome
    /// * `ServiceError::Backend` - Failed to create connectome from genome
    ///
    async fn load_genome(&self, params: LoadGenomeParams) -> ServiceResult<GenomeInfo>;

    /// Save the current connectome as a genome JSON
    ///
    /// Serializes the current brain state (cortical areas, brain regions) to genome format.
    ///
    /// # Arguments
    /// * `params` - Save parameters (optional genome_id and title)
    ///
    /// # Returns
    /// * `String` - Genome JSON
    ///
    /// # Errors
    /// * `ServiceError::Backend` - Failed to serialize genome
    ///
    async fn save_genome(&self, params: SaveGenomeParams) -> ServiceResult<String>;

    /// Get information about the currently loaded genome
    ///
    /// # Returns
    /// * `GenomeInfo` - Metadata about the current genome
    ///
    async fn get_genome_info(&self) -> ServiceResult<GenomeInfo>;

    /// Validate a genome JSON without loading it
    ///
    /// # Arguments
    /// * `json_str` - Genome JSON string
    ///
    /// # Returns
    /// * `bool` - True if valid
    ///
    /// # Errors
    /// * `ServiceError::InvalidInput` - Invalid JSON or malformed genome
    ///
    async fn validate_genome(&self, json_str: String) -> ServiceResult<bool>;

    /// Reset the connectome (clear all cortical areas and brain regions)
    ///
    /// # Errors
    /// * `ServiceError::Backend` - Failed to reset connectome
    ///
    async fn reset_connectome(&self) -> ServiceResult<()>;
}

