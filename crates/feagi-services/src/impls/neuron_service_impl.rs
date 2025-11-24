/*!
Neuron service implementation.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::traits::NeuronService;
use crate::types::*;
use feagi_data_structures::genomic::cortical_area::CorticalID;
use async_trait::async_trait;
use feagi_bdu::ConnectomeManager;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::debug;

/// Default implementation of NeuronService
pub struct NeuronServiceImpl {
    connectome: Arc<RwLock<ConnectomeManager>>,
}

impl NeuronServiceImpl {
    pub fn new(connectome: Arc<RwLock<ConnectomeManager>>) -> Self {
        Self { connectome }
    }
}

#[async_trait]
impl NeuronService for NeuronServiceImpl {
    async fn create_neuron(&self, params: CreateNeuronParams) -> ServiceResult<NeuronInfo> {
        debug!(target: "feagi-services", "Creating neuron in area {} at {:?}", params.cortical_id, params.coordinates);
        
        // Convert String to CorticalID
        let cortical_id_typed = CorticalID::try_from_base_64(&params.cortical_id)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?;
        
        let mut manager = self.connectome.write();
        
        // Extract neural parameters from properties or use defaults
        let props = params.properties.as_ref();
        
        let firing_threshold = props
            .and_then(|p| p.get("firing_threshold"))
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;
        
        let leak_coefficient = props
            .and_then(|p| p.get("leak_coefficient"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        
        let resting_potential = props
            .and_then(|p| p.get("resting_potential"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32;
        
        let is_inhibitory = props
            .and_then(|p| p.get("is_inhibitory"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let refractory_period = props
            .and_then(|p| p.get("refractory_period"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as u16;
        
        let excitability = props
            .and_then(|p| p.get("excitability"))
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0) as f32;
        
        let consecutive_fire_limit = props
            .and_then(|p| p.get("consecutive_fire_limit"))
            .and_then(|v| v.as_i64())
            .unwrap_or(100) as u16;
        
        let snooze_length = props
            .and_then(|p| p.get("snooze_length"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as u16;
        
        let mp_charge_accumulation = props
            .and_then(|p| p.get("mp_charge_accumulation"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        // Add neuron via ConnectomeManager
        let neuron_id = manager.add_neuron(
            &cortical_id_typed,
            params.coordinates.0,
            params.coordinates.1,
            params.coordinates.2,
            firing_threshold,
            leak_coefficient,
            resting_potential,
            if is_inhibitory { 1 } else { 0 },
            refractory_period,
            excitability,
            consecutive_fire_limit,
            snooze_length,
            mp_charge_accumulation,
        ).map_err(ServiceError::from)?;
        
        let cortical_idx = manager.get_cortical_idx(&cortical_id_typed)
            .ok_or_else(|| ServiceError::NotFound {
                resource: "CorticalArea".to_string(),
                id: params.cortical_id.clone(),
            })?;
        
        Ok(NeuronInfo {
            id: neuron_id,
            cortical_id: params.cortical_id.clone(),
            cortical_idx,
            coordinates: params.coordinates,
            properties: params.properties.unwrap_or_default(),
        })
    }

    async fn delete_neuron(&self, neuron_id: u64) -> ServiceResult<()> {
        debug!(target: "feagi-services","Deleting neuron {}", neuron_id);
        
        let mut manager = self.connectome.write();
        let deleted = manager.delete_neuron(neuron_id).map_err(ServiceError::from)?;
        
        if !deleted {
            return Err(ServiceError::NotFound {
                resource: "Neuron".to_string(),
                id: neuron_id.to_string(),
            });
        }
        
        Ok(())
    }

    async fn get_neuron(&self, neuron_id: u64) -> ServiceResult<NeuronInfo> {
        debug!(target: "feagi-services","Getting neuron {}", neuron_id);
        
        let manager = self.connectome.read();
        
        // Check if neuron exists
        if !manager.has_neuron(neuron_id) {
            return Err(ServiceError::NotFound {
                resource: "Neuron".to_string(),
                id: neuron_id.to_string(),
            });
        }
        
        let coordinates = manager.get_neuron_coordinates(neuron_id);
        let cortical_idx = manager.get_neuron_cortical_idx(neuron_id);
        let cortical_id = manager.get_neuron_cortical_id(neuron_id)
            .map(|id| id.as_base_64())
            .unwrap_or_else(|| "unknown".to_string());
        
        Ok(NeuronInfo {
            id: neuron_id,
            cortical_id,
            cortical_idx,
            coordinates,
            properties: std::collections::HashMap::new(),
        })
    }

    async fn get_neuron_at_coordinates(
        &self,
        cortical_id: &str,
        coordinates: (u32, u32, u32),
    ) -> ServiceResult<Option<NeuronInfo>> {
        debug!(target: "feagi-services","Looking up neuron in area {} at {:?}", cortical_id, coordinates);
        
        let manager = self.connectome.read();
        
        // Verify area exists
        if !manager.has_cortical_area(&CorticalID::try_from_base_64(cortical_id).map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?) {
            return Err(ServiceError::NotFound {
                resource: "CorticalArea".to_string(),
                id: cortical_id.to_string(),
            });
        }
        
        // Get all neurons in the area and find one at coordinates
        let neurons = manager.get_neurons_in_area(&CorticalID::try_from_base_64(cortical_id).map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?);
        
        for neuron_id in neurons {
            let neuron_coords = manager.get_neuron_coordinates(neuron_id);
            if neuron_coords == coordinates {
                let cortical_idx = manager.get_neuron_cortical_idx(neuron_id);
                return Ok(Some(NeuronInfo {
                    id: neuron_id,
                    cortical_id: cortical_id.to_string(),
                    cortical_idx,
                    coordinates,
                    properties: std::collections::HashMap::new(),
                }));
            }
        }
        
        // No neuron at these coordinates
        Ok(None)
    }

    async fn list_neurons_in_area(
        &self,
        cortical_id: &str,
        limit: Option<usize>,
    ) -> ServiceResult<Vec<NeuronInfo>> {
        debug!(target: "feagi-services","Listing neurons in area: {}", cortical_id);
        
        // Get neurons from ConnectomeManager
        let manager = self.connectome.read();
        let neuron_ids = manager.get_neurons_in_area(&CorticalID::try_from_base_64(cortical_id).map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?);
        
        let neurons: Vec<NeuronInfo> = neuron_ids
            .iter()
            .take(limit.unwrap_or(usize::MAX))
            .map(|&id| {
                // Get coordinates and cortical idx
                let coordinates = manager.get_neuron_coordinates(id);
                let cortical_idx = manager.get_neuron_cortical_idx(id);
                
                // CRITICAL: (0,0,0) is a VALID coordinate for 1x1x1 areas like _power!
                // Do NOT filter out neurons at (0,0,0) - it's a legitimate position
                NeuronInfo {
                    id,
                    cortical_id: cortical_id.to_string(),
                    cortical_idx,
                    coordinates,
                    properties: std::collections::HashMap::new(),  // TODO: Get properties
                }
            })
            .collect();
        
        Ok(neurons)
    }

    async fn get_neuron_count(&self, cortical_id: &str) -> ServiceResult<usize> {
        debug!(target: "feagi-services","Getting neuron count for area: {}", cortical_id);
        
        // Convert String to CorticalID
        let cortical_id_typed = CorticalID::try_from_base_64(cortical_id)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid cortical ID: {}", e)))?;
        
        let count = self.connectome
            .read()
            .get_neuron_count_in_area(&cortical_id_typed);
        
        Ok(count)
    }

    async fn neuron_exists(&self, neuron_id: u64) -> ServiceResult<bool> {
        debug!(target: "feagi-services","Checking if neuron exists: {}", neuron_id);
        
        let exists = self.connectome.read().has_neuron(neuron_id);
        
        Ok(exists)
    }
}

