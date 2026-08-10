// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Analytics over the loaded genome and the running NPU.

use async_trait::async_trait;
use feagi_services::traits::analytics_service::AnalyticsService;
use feagi_services::types::errors::{ServiceError, ServiceResult};
use feagi_services::types::*;

pub struct GenomeAnalyticsService {
    genome: crate::services::SharedGenome,
    /// The running engine, when one was injected.
    npu: crate::services::OptionalNpu,
}

impl GenomeAnalyticsService {
    pub fn new(genome: crate::services::SharedGenome, npu: crate::services::OptionalNpu) -> Self {
        Self { genome, npu }
    }

    /// How many cortical areas the brain currently holds.
    ///
    /// The engine is asked first because areas can be created after the genome was loaded. A
    /// server with no genome and no engine holds none, which is a reportable state rather than an
    /// error: health has to answer before anything is loaded, or it cannot be used to tell whether
    /// the server came up.
    fn cortical_area_count(&self) -> usize {
        match self.npu.as_deref() {
            Some(npu) => npu.cortical_areas().len(),
            None => crate::services::with_genome(&self.genome, |g| g.cortical_areas.len()).unwrap_or(0),
        }
    }
}

#[async_trait]
impl AnalyticsService for GenomeAnalyticsService {
    /// Reports health without requiring a genome or an engine.
    ///
    /// This endpoint is what a monitor polls to learn whether the server is up, so it answers in
    /// every state and describes what is missing through its fields rather than failing.
    async fn get_system_health(&self) -> ServiceResult<SystemHealth> {
        let cortical_area_count = self.cortical_area_count();
        let (burst_engine_active, burst_count) = match self.npu.as_deref() {
            Some(npu) => (npu.is_running(), npu.burst_count()),
            None => (false, 0),
        };

        Ok(SystemHealth {
            burst_engine_active,
            // A brain is ready only once it has structure and the engine is turning it over.
            // Reporting readiness on either alone lets a visualizer leave its loading screen
            // before there is anything to show.
            brain_readiness: cortical_area_count > 0 && burst_engine_active,
            // The validator verdict lives on the core state atomic, which a genome-backed
            // service cannot see; `None` reports "no verdict recorded" rather than asserting one.
            genome_validity: None,
            neuron_count: self.get_total_neuron_count().await?,
            neuron_capacity: 0,  // TODO: Get from runtime if available
            synapse_capacity: 0, // TODO: Get from runtime if available
            cortical_area_count,
            burst_count,
        })
    }

    async fn get_cortical_area_stats(&self, _cortical_id: &str) -> ServiceResult<CorticalAreaStats> {
        Err(ServiceError::NotImplemented("cortical area stats are not yet implemented".to_string()))
    }

    async fn get_all_cortical_area_stats(&self) -> ServiceResult<Vec<CorticalAreaStats>> {
        Err(ServiceError::NotImplemented("cortical area stats are not yet implemented".to_string()))
    }

    async fn get_connectivity_stats(&self, _source_area: &str, _target_area: &str) -> ServiceResult<ConnectivityStats> {
        Err(ServiceError::NotImplemented("connectivity stats are not yet implemented".to_string()))
    }

    async fn get_total_neuron_count(&self) -> ServiceResult<usize> {
        // Counted from the engine's areas, which is what actually exists; a server without an
        // engine holds no neurons regardless of what a genome describes.
        Ok(match self.npu.as_deref() {
            Some(npu) => npu.cortical_areas().iter().map(|area| area.neuron_count as usize).sum(),
            None => 0,
        })
    }

    async fn get_total_synapse_count(&self) -> ServiceResult<usize> {
        Ok(0) // TODO: Get from NPU if available
    }

    async fn get_populated_areas(&self) -> ServiceResult<Vec<(String, usize)>> {
        Ok(vec![]) // TODO: Get from NPU if available
    }

    async fn get_neuron_density(&self, _cortical_id: &str) -> ServiceResult<f32> {
        Err(ServiceError::NotImplemented("neuron density is not yet implemented".to_string()))
    }

    async fn is_brain_initialized(&self) -> ServiceResult<bool> {
        // A brain is initialized once it holds cortical areas; nothing loaded is a definite
        // "not initialized" rather than an error.
        Ok(self.cortical_area_count() > 0)
    }

    async fn is_burst_engine_ready(&self) -> ServiceResult<bool> {
        Ok(self.npu.is_some())
    }

    async fn get_regular_neuron_count(&self) -> ServiceResult<usize> {
        Ok(0) // TODO: Get from NPU if available
    }

    async fn get_memory_neuron_count(&self) -> ServiceResult<usize> {
        Ok(0) // TODO: Get from NPU if available
    }
}
