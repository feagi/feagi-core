// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Genome-backed implementation of [`GenomeService`].

use async_trait::async_trait;
use feagi_brain_development::corticogenesis::develop_connectome_requests;
use feagi_services::traits::genome_service::GenomeService;
use feagi_services::types::errors::{ServiceError, ServiceResult};
use feagi_services::types::*;
use feagi_state_manager::StateManager;

use crate::services::{NpuAccess, SharedGenome};

/// What realising a genome in the engine produced.
///
/// Carries more than the [`GenomeInfo`] the service trait returns, because callers that drive a
/// load directly (the server's `--genome` flag) report on the connectome that was built, not just
/// on the genome document that described it.
#[derive(Debug, Clone)]
pub struct GenomeRealisation {
    pub info: GenomeInfo,
    pub areas_added: usize,
    pub neurons_added: u64,
    /// Mappings the genome declares that corticogenesis could not realise. Non-zero means the
    /// brain has areas but no synapses between them, so nothing propagates.
    pub mappings_deferred: usize,
}

/// Parses a genome document and realises it in the engine.
///
/// The single loading path: both `/v1/genome/*` and the server's startup genome flag come through
/// here, so a genome loaded over REST and one loaded from the command line produce the same brain.
///
/// The genome is published to `genome_handle` only after corticogenesis succeeds, so the REST layer
/// never reports a genome the engine was unable to realise.
pub fn realise_genome_json(json: &str, genome_handle: &SharedGenome, npu: &dyn NpuAccess) -> ServiceResult<GenomeRealisation> {
    // The migration chain reports validator findings rather than failing on them, so a genome with
    // blocking errors still loads and the finding is published as `genome_validity` for /health.
    let (mut genome, chain_report) =
        feagi_evolutionary::load_genome_with_report(json).map_err(|e| ServiceError::InvalidInput(format!("Failed to parse genome: {}", e)))?;

    if let Some(state_manager) = StateManager::instance().try_read() {
        state_manager.get_core_state().set_genome_validity(Some(chain_report.is_blocking_clean()));
    }

    if !chain_report.is_blocking_clean() {
        tracing::warn!(
            target: "feagi-api",
            "Loaded genome has {} blocking validator error(s); running in degraded mode (genome_validity=false). First: {}",
            chain_report.blocking_errors.len(),
            chain_report.blocking_errors.first().map(String::as_str).unwrap_or("")
        );
    }

    let (_areas_added, morphologies_added) = feagi_evolutionary::ensure_core_components(&mut genome);
    if morphologies_added > 0 {
        tracing::info!(target: "feagi-api", "Added {} missing core morphologies during genome load", morphologies_added);
    }

    let (requests, report) = develop_connectome_requests(&genome).map_err(|e| ServiceError::Backend(format!("Corticogenesis failed: {}", e)))?;

    npu.submit_connectome_requests(requests);

    let info = GenomeInfo {
        genome_id: genome.metadata.genome_id.clone(),
        genome_title: genome.metadata.genome_title.clone(),
        version: genome.metadata.version.clone(),
        cortical_area_count: genome.cortical_areas.len(),
        brain_region_count: genome.brain_regions.len(),
        // The burst frequency is derived from this by the caller, so it has to be the genome's own
        // value rather than a placeholder.
        simulation_timestep: genome.physiology.simulation_timestep,
        genome_num: None,
        genome_timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|since_epoch| since_epoch.as_secs() as i64),
    };

    *genome_handle.write() = Some(genome);

    tracing::info!(
        target: "feagi-api",
        "Genome '{}' realised: {} areas, {} neurons",
        info.genome_title,
        report.areas_added,
        report.neurons_added
    );

    if report.mappings_deferred > 0 {
        tracing::warn!(
            target: "feagi-api",
            "Genome declares {} mapping(s) that corticogenesis cannot realise yet; the brain has no \
             synapses and activity will not propagate between areas",
            report.mappings_deferred
        );
    }

    Ok(GenomeRealisation {
        info,
        areas_added: report.areas_added,
        neurons_added: report.neurons_added,
        mappings_deferred: report.mappings_deferred,
    })
}

pub struct GenomeGenomeService {
    genome: crate::services::SharedGenome,
    /// Needed because loading a genome and creating a custom area are engine operations rather
    /// than genome edits. Without one, both report the NPU as unavailable.
    npu: crate::services::OptionalNpu,
}

impl GenomeGenomeService {
    pub fn new(genome: crate::services::SharedGenome, npu: crate::services::OptionalNpu) -> Self {
        Self { genome, npu }
    }
}

#[async_trait]
impl GenomeService for GenomeGenomeService {
    async fn load_genome(&self, params: LoadGenomeParams) -> ServiceResult<GenomeInfo> {
        let npu = self.npu.as_deref().ok_or_else(|| crate::services::npu_unavailable("genome loading"))?;
        realise_genome_json(&params.json_str, &self.genome, npu).map(|realisation| realisation.info)
    }

    async fn save_genome(&self, _params: SaveGenomeParams) -> ServiceResult<String> {
        // TODO: Serialize RuntimeGenome to JSON
        Err(ServiceError::NotImplemented("genome saving is not yet implemented".to_string()))
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
            simulation_timestep: g.physiology.simulation_timestep,
            genome_num: None,
            genome_timestamp: Some(g.metadata.timestamp as i64),
        })
    }

    async fn validate_genome(&self, _json_str: String) -> ServiceResult<bool> {
        Err(ServiceError::NotImplemented("genome validation is not yet implemented".to_string()))
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
