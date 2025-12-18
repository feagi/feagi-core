// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WASM Genome Service (stub - read-only)

use async_trait::async_trait;
use feagi_evo::RuntimeGenome;
use feagi_services::traits::genome_service::GenomeService;
use feagi_services::types::errors::{ServiceError, ServiceResult};
use feagi_services::types::*;
use std::sync::Arc;

pub struct WasmGenomeService {
    genome: Arc<RuntimeGenome>,
}

impl WasmGenomeService {
    pub fn new(genome: Arc<RuntimeGenome>) -> Self {
        Self { genome }
    }
}

#[async_trait]
impl GenomeService for WasmGenomeService {
    async fn load_genome(&self, _params: LoadGenomeParams) -> ServiceResult<GenomeInfo> {
        Err(ServiceError::NotImplemented(
            "WASM mode genome loading handled by FeagiEngine".to_string(),
        ))
    }

    async fn save_genome(&self, _params: SaveGenomeParams) -> ServiceResult<String> {
        // TODO: Serialize RuntimeGenome to JSON
        Err(ServiceError::NotImplemented(
            "WASM mode genome saving not yet implemented".to_string(),
        ))
    }

    async fn get_genome_info(&self) -> ServiceResult<GenomeInfo> {
        Ok(GenomeInfo {
            genome_id: self.genome.metadata.genome_id.clone(),
            genome_title: self.genome.metadata.genome_title.clone(),
            version: self.genome.metadata.version.clone(),
            cortical_area_count: self.genome.cortical_areas.len(),
            brain_region_count: self.genome.brain_regions.len(),
            simulation_timestep: 0.0, // TODO: Extract from physiology config
            genome_num: None,
            genome_timestamp: Some(self.genome.metadata.timestamp as i64),
        })
    }

    async fn validate_genome(&self, _json_str: String) -> ServiceResult<bool> {
        Err(ServiceError::NotImplemented(
            "WASM mode genome validation not yet implemented".to_string(),
        ))
    }

    async fn reset_connectome(&self) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented(
            "WASM mode is read-only".to_string(),
        ))
    }

    async fn update_cortical_area(
        &self,
        _cortical_id: &str,
        _changes: std::collections::HashMap<String, serde_json::Value>,
    ) -> ServiceResult<CorticalAreaInfo> {
        Err(ServiceError::NotImplemented(
            "WASM mode is read-only".to_string(),
        ))
    }

    async fn create_cortical_areas(
        &self,
        _params: Vec<CreateCorticalAreaParams>,
    ) -> ServiceResult<Vec<CorticalAreaInfo>> {
        Err(ServiceError::NotImplemented(
            "WASM mode is read-only".to_string(),
        ))
    }
}
