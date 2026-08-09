// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WASM Analytics Service (stub)

use async_trait::async_trait;
use feagi_services::traits::analytics_service::AnalyticsService;
use feagi_services::types::errors::{ServiceError, ServiceResult};
use feagi_services::types::*;

pub struct GenomeAnalyticsService {
    genome: crate::services::SharedGenome,
}

impl GenomeAnalyticsService {
    pub fn new(genome: crate::services::SharedGenome) -> Self {
        Self { genome }
    }
}

#[async_trait]
impl AnalyticsService for GenomeAnalyticsService {
    async fn get_system_health(&self) -> ServiceResult<SystemHealth> {
        // Genome-derived health; live counters come from the runtime services
        let cortical_area_count =
            crate::services::with_genome(&self.genome, |g| g.cortical_areas.len())?;
        Ok(SystemHealth {
            burst_engine_active: true,
            brain_readiness: cortical_area_count > 0,
            // The validator verdict lives on the core state atomic, which a genome-backed
            // service cannot see; `None` reports "no verdict recorded" rather than asserting one.
            genome_validity: None,
            neuron_count: 0,     // TODO: Get from NPU if available
            neuron_capacity: 0,  // TODO: Get from runtime if available
            synapse_capacity: 0, // TODO: Get from runtime if available
            cortical_area_count,
            burst_count: 0, // TODO: Get from NPU if available
        })
    }

    async fn get_cortical_area_stats(
        &self,
        _cortical_id: &str,
    ) -> ServiceResult<CorticalAreaStats> {
        Err(ServiceError::NotImplemented(
            "Cortical area stats not yet implemented in WASM".to_string(),
        ))
    }

    async fn get_all_cortical_area_stats(&self) -> ServiceResult<Vec<CorticalAreaStats>> {
        Err(ServiceError::NotImplemented(
            "Cortical area stats not yet implemented in WASM".to_string(),
        ))
    }

    async fn get_connectivity_stats(
        &self,
        _source_area: &str,
        _target_area: &str,
    ) -> ServiceResult<ConnectivityStats> {
        Err(ServiceError::NotImplemented(
            "Connectivity stats not yet implemented in WASM".to_string(),
        ))
    }

    async fn get_total_neuron_count(&self) -> ServiceResult<usize> {
        Ok(0) // TODO: Get from NPU if available
    }

    async fn get_total_synapse_count(&self) -> ServiceResult<usize> {
        Ok(0) // TODO: Get from NPU if available
    }

    async fn get_populated_areas(&self) -> ServiceResult<Vec<(String, usize)>> {
        Ok(vec![]) // TODO: Get from NPU if available
    }

    async fn get_neuron_density(&self, _cortical_id: &str) -> ServiceResult<f32> {
        Err(ServiceError::NotImplemented(
            "Neuron density not yet implemented in WASM".to_string(),
        ))
    }

    async fn is_brain_initialized(&self) -> ServiceResult<bool> {
        // A brain is initialized once a genome with cortical areas is loaded; no genome loaded is
        // a definite "not initialized" rather than an error.
        Ok(crate::services::with_genome(&self.genome, |g| {
            !g.cortical_areas.is_empty()
        })
        .unwrap_or(false))
    }

    async fn is_burst_engine_ready(&self) -> ServiceResult<bool> {
        Ok(true) // Always ready in WASM
    }

    async fn get_regular_neuron_count(&self) -> ServiceResult<usize> {
        Ok(0) // TODO: Get from NPU if available
    }

    async fn get_memory_neuron_count(&self) -> ServiceResult<usize> {
        Ok(0) // TODO: Get from NPU if available
    }
}
