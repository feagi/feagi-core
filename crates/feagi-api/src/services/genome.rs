// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WASM Genome Service (stub - read-only)

use async_trait::async_trait;
use feagi_services::traits::genome_service::GenomeService;
use feagi_services::types::errors::{ServiceError, ServiceResult};
use feagi_services::types::*;

pub struct GenomeGenomeService {
    genome: crate::services::SharedGenome,
    /// Needed because the custom-area route creates areas through this service, and creation is
    /// an engine operation rather than a genome edit.
    npu: crate::services::OptionalNpu,
}

impl GenomeGenomeService {
    pub fn new(genome: crate::services::SharedGenome, npu: crate::services::OptionalNpu) -> Self {
        Self { genome, npu }
    }
}

#[async_trait]
impl GenomeService for GenomeGenomeService {
    async fn load_genome(&self, _params: LoadGenomeParams) -> ServiceResult<GenomeInfo> {
        Err(ServiceError::NotImplemented(
            "WASM mode genome loading handled by FeagiEngine".to_string(),
        ))
    }

    async fn save_genome(&self, _params: SaveGenomeParams) -> ServiceResult<String> {
        // TODO: Serialize RuntimeGenome to JSON
        Err(ServiceError::NotImplemented("WASM mode genome saving not yet implemented".to_string()))
    }

    async fn export_region_genome(&self, region_id: String) -> ServiceResult<String> {
        let subset = crate::services::with_genome(&self.genome, |g| {
            feagi_evolutionary::subset_runtime_genome_for_region_branch(g, &region_id)
        })?
        .map_err(|e| match e {
            feagi_evolutionary::EvoError::InvalidRegion(key) => ServiceError::InvalidInput(key.message.to_string()),
            other => ServiceError::Internal(other.to_string()),
        })?;
        feagi_evolutionary::save_genome_to_json(&subset).map_err(|e| ServiceError::Internal(format!("Failed to serialize region genome: {}", e)))
    }

    async fn get_genome_info(&self) -> ServiceResult<GenomeInfo> {
        crate::services::with_genome(&self.genome, |g| GenomeInfo {
            genome_id: g.metadata.genome_id.clone(),
            genome_title: g.metadata.genome_title.clone(),
            version: g.metadata.version.clone(),
            cortical_area_count: g.cortical_areas.len(),
            brain_region_count: g.brain_regions.len(),
            simulation_timestep: 0.0, // TODO: Extract from physiology config
            genome_num: None,
            genome_timestamp: Some(g.metadata.timestamp as i64),
        })
    }

    async fn validate_genome(&self, _json_str: String) -> ServiceResult<bool> {
        Err(ServiceError::NotImplemented(
            "WASM mode genome validation not yet implemented".to_string(),
        ))
    }

    async fn reset_connectome(&self) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented("genome-backed service is read-only".to_string()))
    }

    async fn update_cortical_area(
        &self,
        _cortical_id: &str,
        _changes: std::collections::HashMap<String, serde_json::Value>,
    ) -> ServiceResult<CorticalAreaInfo> {
        Err(ServiceError::NotImplemented("genome-backed service is read-only".to_string()))
    }

    async fn create_cortical_areas(&self, params: Vec<CreateCorticalAreaParams>) -> ServiceResult<Vec<CorticalAreaInfo>> {
        let npu = self
            .npu
            .as_deref()
            .ok_or_else(|| crate::services::npu_unavailable("cortical area creation"))?;

        // Realised one at a time and reported in request order. The engine has no batch entry
        // point, so a later failure leaves earlier areas in place rather than rolling back.
        params
            .into_iter()
            .map(|p| crate::services::connectome::create_area_in_npu(npu, p))
            .collect()
    }
}
