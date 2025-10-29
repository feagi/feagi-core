/*!
Analytics service implementation.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::traits::AnalyticsService;
use crate::types::*;
use async_trait::async_trait;
use feagi_bdu::ConnectomeManager;
use std::sync::Arc;

/// Default implementation of AnalyticsService
pub struct AnalyticsServiceImpl {
    connectome: Arc<ConnectomeManager>,
}

impl AnalyticsServiceImpl {
    pub fn new(connectome: Arc<ConnectomeManager>) -> Self {
        Self { connectome }
    }
}

#[async_trait]
impl AnalyticsService for AnalyticsServiceImpl {
    async fn get_system_health(&self) -> ServiceResult<SystemHealth> {
        log::debug!("Getting system health");
        
        let neuron_count = self.get_total_neuron_count().await?;
        let cortical_area_count = self.connectome.get_cortical_area_count();
        let brain_initialized = cortical_area_count > 0;
        
        // TODO: Get actual burst engine status
        let burst_engine_active = self.connectome.has_npu();
        
        Ok(SystemHealth {
            burst_engine_active,
            brain_readiness: brain_initialized,
            neuron_count,
            cortical_area_count,
            burst_count: 0,  // TODO: Get from burst engine
        })
    }

    async fn get_cortical_area_stats(
        &self,
        cortical_id: &str,
    ) -> ServiceResult<CorticalAreaStats> {
        log::debug!("Getting cortical area stats: {}", cortical_id);
        
        let neuron_count = self.connectome.get_neuron_count_in_area(cortical_id);
        let synapse_count = self.connectome.get_synapse_count_in_area(cortical_id);
        let density = self.connectome.get_neuron_density(cortical_id);
        let populated = neuron_count > 0;
        
        Ok(CorticalAreaStats {
            cortical_id: cortical_id.to_string(),
            neuron_count,
            synapse_count,
            density,
            populated,
        })
    }

    async fn get_all_cortical_area_stats(&self) -> ServiceResult<Vec<CorticalAreaStats>> {
        log::debug!("Getting all cortical area stats");
        
        let all_stats = self.connectome.get_all_area_stats();
        
        let stats: Vec<CorticalAreaStats> = all_stats
            .into_iter()
            .map(|(cortical_id, neuron_count, synapse_count, density)| CorticalAreaStats {
                cortical_id,
                neuron_count,
                synapse_count,
                density,
                populated: neuron_count > 0,
            })
            .collect();
        
        Ok(stats)
    }

    async fn get_connectivity_stats(
        &self,
        source_area: &str,
        target_area: &str,
    ) -> ServiceResult<ConnectivityStats> {
        log::debug!(
            "Getting connectivity stats: {} -> {}",
            source_area,
            target_area
        );
        
        // TODO: Implement proper connectivity stats between two areas
        // For now, return placeholder data
        log::warn!("Connectivity stats between areas not yet fully implemented");
        
        Ok(ConnectivityStats {
            source_area: source_area.to_string(),
            target_area: target_area.to_string(),
            synapse_count: 0,
            avg_weight: 0.0,
            excitatory_count: 0,
            inhibitory_count: 0,
        })
    }

    async fn get_total_neuron_count(&self) -> ServiceResult<usize> {
        log::debug!("Getting total neuron count");
        
        let count = self.connectome.get_neuron_count();
        Ok(count)
    }

    async fn get_total_synapse_count(&self) -> ServiceResult<usize> {
        log::debug!("Getting total synapse count");
        
        let count = self.connectome.get_synapse_count();
        Ok(count)
    }

    async fn get_populated_areas(&self) -> ServiceResult<Vec<(String, usize)>> {
        log::debug!("Getting populated areas");
        
        let areas = self.connectome.get_populated_areas();
        Ok(areas)
    }

    async fn get_neuron_density(&self, cortical_id: &str) -> ServiceResult<f32> {
        log::debug!("Getting neuron density for area: {}", cortical_id);
        
        let density = self.connectome.get_neuron_density(cortical_id);
        Ok(density)
    }

    async fn is_brain_initialized(&self) -> ServiceResult<bool> {
        log::debug!("Checking if brain is initialized");
        
        let initialized = self.connectome.is_initialized();
        Ok(initialized)
    }

    async fn is_burst_engine_ready(&self) -> ServiceResult<bool> {
        log::debug!("Checking if burst engine is ready");
        
        let ready = self.connectome.has_npu();
        Ok(ready)
    }
}

