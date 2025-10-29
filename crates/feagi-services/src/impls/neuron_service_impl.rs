/*!
Neuron service implementation.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::traits::NeuronService;
use crate::types::*;
use async_trait::async_trait;
use feagi_bdu::ConnectomeManager;
use std::sync::Arc;

/// Default implementation of NeuronService
pub struct NeuronServiceImpl {
    connectome: Arc<ConnectomeManager>,
}

impl NeuronServiceImpl {
    pub fn new(connectome: Arc<ConnectomeManager>) -> Self {
        Self { connectome }
    }
}

#[async_trait]
impl NeuronService for NeuronServiceImpl {
    async fn create_neuron(&self, _params: CreateNeuronParams) -> ServiceResult<NeuronInfo> {
        // TODO: Implement neuron creation
        Err(ServiceError::Internal("Not yet implemented".to_string()))
    }

    async fn delete_neuron(&self, _neuron_id: u64) -> ServiceResult<()> {
        // TODO: Implement neuron deletion
        Err(ServiceError::Internal("Not yet implemented".to_string()))
    }

    async fn get_neuron(&self, _neuron_id: u64) -> ServiceResult<NeuronInfo> {
        // TODO: Implement neuron retrieval
        Err(ServiceError::Internal("Not yet implemented".to_string()))
    }

    async fn get_neuron_at_coordinates(
        &self,
        _cortical_id: &str,
        _coordinates: (u32, u32, u32),
    ) -> ServiceResult<Option<NeuronInfo>> {
        // TODO: Implement coordinate-based lookup
        Err(ServiceError::Internal("Not yet implemented".to_string()))
    }

    async fn list_neurons_in_area(
        &self,
        cortical_id: &str,
        limit: Option<usize>,
    ) -> ServiceResult<Vec<NeuronInfo>> {
        log::debug!("Listing neurons in area: {}", cortical_id);
        
        // Get neurons from ConnectomeManager
        let neuron_ids = self.connectome.get_neurons_in_area(cortical_id);
        
        let neurons: Vec<NeuronInfo> = neuron_ids
            .iter()
            .take(limit.unwrap_or(usize::MAX))
            .filter_map(|&id| {
                // Get coordinates and cortical idx
                let coords = self.connectome.get_neuron_coordinates(id)?;
                let cortical_idx = self.connectome.get_neuron_cortical_idx(id)?;
                
                Some(NeuronInfo {
                    id,
                    cortical_id: cortical_id.to_string(),
                    cortical_idx,
                    coordinates: coords,
                    properties: std::collections::HashMap::new(),  // TODO: Get properties
                })
            })
            .collect();
        
        Ok(neurons)
    }

    async fn get_neuron_count(&self, cortical_id: &str) -> ServiceResult<usize> {
        log::debug!("Getting neuron count for area: {}", cortical_id);
        
        let count = self.connectome
            .get_neuron_count_in_area(cortical_id);
        
        Ok(count)
    }

    async fn neuron_exists(&self, neuron_id: u64) -> ServiceResult<bool> {
        log::debug!("Checking if neuron exists: {}", neuron_id);
        
        let exists = self.connectome.has_neuron(neuron_id)?;
        
        Ok(exists)
    }
}

