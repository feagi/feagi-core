// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WASM Neuron Service (stub - read-only)

use async_trait::async_trait;
#[cfg(feature = "services")]::::traits::neuron_service::NeuronService;
#[cfg(feature = "services")]::::types::errors::{ServiceError, ServiceResult};
#[cfg(feature = "services")]::::types::*;

pub struct WasmNeuronService;

impl WasmNeuronService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl NeuronService for WasmNeuronService {
    async fn create_neuron(&self, _params: CreateNeuronParams) -> ServiceResult<NeuronInfo> {
        Err(ServiceError::NotImplemented("WASM mode is read-only".to_string()))
    }

    async fn delete_neuron(&self, _neuron_id: u64) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented("WASM mode is read-only".to_string()))
    }

    async fn get_neuron(&self, _neuron_id: u64) -> ServiceResult<NeuronInfo> {
        Err(ServiceError::NotImplemented("Neuron lookup not yet implemented in WASM".to_string()))
    }

    async fn get_neuron_at_coordinates(
        &self,
        _cortical_id: &str,
        _coordinates: (u32, u32, u32),
    ) -> ServiceResult<Option<NeuronInfo>> {
        Err(ServiceError::NotImplemented("Neuron lookup by coordinates not yet implemented in WASM".to_string()))
    }

    async fn list_neurons_in_area(
        &self,
        _cortical_id: &str,
        _limit: Option<usize>,
    ) -> ServiceResult<Vec<NeuronInfo>> {
        Err(ServiceError::NotImplemented("Neuron listing not yet implemented in WASM".to_string()))
    }

    async fn get_neuron_count(&self, _cortical_id: &str) -> ServiceResult<usize> {
        Err(ServiceError::NotImplemented("Neuron count not yet implemented in WASM".to_string()))
    }

    async fn neuron_exists(&self, _neuron_id: u64) -> ServiceResult<bool> {
        Err(ServiceError::NotImplemented("Neuron existence check not yet implemented in WASM".to_string()))
    }
}

