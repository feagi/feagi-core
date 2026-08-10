// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Neuron service (stub - per-neuron introspection is not ported to the current NPU)

use async_trait::async_trait;
use feagi_services::traits::neuron_service::NeuronService;
use feagi_services::types::errors::{ServiceError, ServiceResult};
use feagi_services::types::*;

pub struct GenomeNeuronService;

impl GenomeNeuronService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl NeuronService for GenomeNeuronService {
    async fn create_neuron(&self, _params: CreateNeuronParams) -> ServiceResult<NeuronInfo> {
        Err(ServiceError::NotImplemented("genome-backed service is read-only".to_string()))
    }

    async fn delete_neuron(&self, _neuron_id: u64) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented("genome-backed service is read-only".to_string()))
    }

    async fn get_neuron(&self, _neuron_id: u64) -> ServiceResult<NeuronInfo> {
        Err(ServiceError::NotImplemented("neuron lookup is not yet implemented".to_string()))
    }

    async fn get_neuron_at_coordinates(&self, _cortical_id: &str, _coordinates: (u32, u32, u32)) -> ServiceResult<Option<NeuronInfo>> {
        Err(ServiceError::NotImplemented(
            "neuron lookup by coordinates is not yet implemented".to_string(),
        ))
    }

    async fn list_neurons_in_area(&self, _cortical_id: &str, _limit: Option<usize>) -> ServiceResult<Vec<NeuronInfo>> {
        Err(ServiceError::NotImplemented("neuron listing is not yet implemented".to_string()))
    }

    async fn get_neuron_count(&self, _cortical_id: &str) -> ServiceResult<usize> {
        Err(ServiceError::NotImplemented("neuron count is not yet implemented".to_string()))
    }

    async fn neuron_exists(&self, _neuron_id: u64) -> ServiceResult<bool> {
        Err(ServiceError::NotImplemented("neuron existence check is not yet implemented".to_string()))
    }
}
