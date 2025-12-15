// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WASM Neuron Service (stub - read-only)

use async_trait::async_trait;
use feagi_services::traits::neuron_service::NeuronService;
use feagi_services::types::errors::{ServiceError, ServiceResult};
use feagi_services::types::*;

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

    async fn list_neurons(&self, _cortical_id: &str) -> ServiceResult<Vec<NeuronInfo>> {
        Err(ServiceError::NotImplemented("Neuron listing not yet implemented in WASM".to_string()))
    }
}

