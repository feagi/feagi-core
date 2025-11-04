/*!
Neuron management service trait.

Defines the stable interface for neuron operations, independent of transport.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::types::*;
use async_trait::async_trait;

/// Neuron management service (transport-agnostic)
#[async_trait]
pub trait NeuronService: Send + Sync {
    /// Create a neuron in a cortical area
    ///
    /// # Arguments
    /// * `params` - Neuron creation parameters (cortical_id, coordinates, properties)
    ///
    /// # Returns
    /// * `NeuronInfo` - Information about the created neuron
    ///
    /// # Errors
    /// * `ServiceError::NotFound` - Cortical area not found
    /// * `ServiceError::InvalidInput` - Invalid coordinates or parameters
    /// * `ServiceError::AlreadyExists` - Neuron already exists at coordinates
    ///
    async fn create_neuron(
        &self,
        params: CreateNeuronParams,
    ) -> ServiceResult<NeuronInfo>;

    /// Delete a neuron by ID
    ///
    /// # Arguments
    /// * `neuron_id` - Global neuron ID
    ///
    /// # Errors
    /// * `ServiceError::NotFound` - Neuron not found
    ///
    async fn delete_neuron(&self, neuron_id: u64) -> ServiceResult<()>;

    /// Get neuron information
    ///
    /// # Arguments
    /// * `neuron_id` - Global neuron ID
    ///
    /// # Returns
    /// * `NeuronInfo` - Information about the neuron
    ///
    /// # Errors
    /// * `ServiceError::NotFound` - Neuron not found
    ///
    async fn get_neuron(&self, neuron_id: u64) -> ServiceResult<NeuronInfo>;

    /// Get neuron by coordinates in a cortical area
    ///
    /// # Arguments
    /// * `cortical_id` - Cortical area identifier
    /// * `coordinates` - (x, y, z) coordinates within the area
    ///
    /// # Returns
    /// * `Option<NeuronInfo>` - Neuron at coordinates, or None if empty
    ///
    /// # Errors
    /// * `ServiceError::NotFound` - Cortical area not found
    /// * `ServiceError::InvalidInput` - Invalid coordinates
    ///
    async fn get_neuron_at_coordinates(
        &self,
        cortical_id: &str,
        coordinates: (u32, u32, u32),
    ) -> ServiceResult<Option<NeuronInfo>>;

    /// List all neurons in a cortical area
    ///
    /// # Arguments
    /// * `cortical_id` - Cortical area identifier
    /// * `limit` - Optional limit on number of neurons returned
    ///
    /// # Returns
    /// * `Vec<NeuronInfo>` - List of neurons in the area
    ///
    /// # Errors
    /// * `ServiceError::NotFound` - Cortical area not found
    ///
    async fn list_neurons_in_area(
        &self,
        cortical_id: &str,
        limit: Option<usize>,
    ) -> ServiceResult<Vec<NeuronInfo>>;

    /// Get the count of neurons in a cortical area
    ///
    /// # Arguments
    /// * `cortical_id` - Cortical area identifier
    ///
    /// # Returns
    /// * `usize` - Number of neurons in the area
    ///
    /// # Errors
    /// * `ServiceError::NotFound` - Cortical area not found
    ///
    async fn get_neuron_count(&self, cortical_id: &str) -> ServiceResult<usize>;

    /// Check if a neuron exists
    ///
    /// # Arguments
    /// * `neuron_id` - Global neuron ID
    ///
    /// # Returns
    /// * `bool` - True if neuron exists
    ///
    async fn neuron_exists(&self, neuron_id: u64) -> ServiceResult<bool>;
}





