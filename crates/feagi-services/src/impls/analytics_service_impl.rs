/*!
Analytics service implementation.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::traits::AnalyticsService;
use crate::types::*;
use async_trait::async_trait;
use feagi_bdu::ConnectomeManager;
use feagi_burst_engine::BurstLoopRunner;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::debug;

/// Default implementation of AnalyticsService
pub struct AnalyticsServiceImpl {
    connectome: Arc<RwLock<ConnectomeManager<f32>>>,
    burst_runner: Option<Arc<RwLock<BurstLoopRunner>>>,
}

impl AnalyticsServiceImpl {
    pub fn new(
        connectome: Arc<RwLock<ConnectomeManager<f32>>>,
        burst_runner: Option<Arc<RwLock<BurstLoopRunner>>>,
    ) -> Self {
        Self {
            connectome,
            burst_runner,
        }
    }
}

#[async_trait]
impl AnalyticsService for AnalyticsServiceImpl {
    async fn get_system_health(&self) -> ServiceResult<SystemHealth> {
        debug!(target: "feagi-services","Getting system health");
        
        let neuron_count = self.get_total_neuron_count().await?;
        let manager = self.connectome.read();
        let cortical_area_count = manager.get_cortical_area_count();
        
        // Get burst engine status from BurstLoopRunner
        let (burst_engine_active, burst_count) = if let Some(ref runner) = self.burst_runner {
            let runner_lock = runner.read();
            (runner_lock.is_running(), runner_lock.get_burst_count())
        } else {
            (false, 0)
        };
        
        // Brain is ready ONLY if genome is loaded AND burst engine is actively running
        // This prevents the Brain Visualizer from exiting loading screen prematurely
        let brain_readiness = cortical_area_count > 0 && burst_engine_active;
        
        Ok(SystemHealth {
            burst_engine_active,
            brain_readiness,
            neuron_count,
            cortical_area_count,
            burst_count,
        })
    }

    async fn get_cortical_area_stats(
        &self,
        cortical_id: &str,
    ) -> ServiceResult<CorticalAreaStats> {
        debug!(target: "feagi-services","Getting cortical area stats: {}", cortical_id);
        
        let manager = self.connectome.read();
        let neuron_count = manager.get_neuron_count_in_area(cortical_id);
        let synapse_count = manager.get_synapse_count_in_area(cortical_id);
        let density = manager.get_neuron_density(cortical_id);
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
        debug!(target: "feagi-services","Getting all cortical area stats");
        
        let all_stats = self.connectome.read().get_all_area_stats();
        
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
        debug!(target: "feagi-services",
            "Getting connectivity stats: {} -> {}",
            source_area,
            target_area
        );
        
        let manager = self.connectome.read();
        
        // Verify both areas exist
        if !manager.has_cortical_area(source_area) {
            return Err(ServiceError::NotFound {
                resource: "CorticalArea".to_string(),
                id: source_area.to_string(),
            });
        }
        if !manager.has_cortical_area(target_area) {
            return Err(ServiceError::NotFound {
                resource: "CorticalArea".to_string(),
                id: target_area.to_string(),
            });
        }
        
        // Get all neurons in source area
        let source_neurons = manager.get_neurons_in_area(source_area);
        
        // Count synapses going from source to target
        let mut synapse_count = 0;
        let mut total_weight: u64 = 0;
        let mut excitatory_count = 0;
        let mut inhibitory_count = 0;
        
        for source_neuron_id in source_neurons {
            // Get outgoing synapses from this neuron
            let outgoing = manager.get_outgoing_synapses(source_neuron_id);
            
            for (target_neuron_id, weight, _conductance, synapse_type) in outgoing {
                // Check if target neuron is in target area
                if let Some(target_cortical_id) = manager.get_neuron_cortical_id(target_neuron_id as u64) {
                    if target_cortical_id == target_area {
                        synapse_count += 1;
                        total_weight += weight as u64;
                        
                        // synapse_type: 0 = excitatory, 1 = inhibitory (from feagi-types)
                        if synapse_type == 0 {
                            excitatory_count += 1;
                        } else {
                            inhibitory_count += 1;
                        }
                    }
                }
            }
        }
        
        let avg_weight = if synapse_count > 0 {
            (total_weight as f64 / synapse_count as f64) as f32
        } else {
            0.0
        };
        
        Ok(ConnectivityStats {
            source_area: source_area.to_string(),
            target_area: target_area.to_string(),
            synapse_count,
            avg_weight,
            excitatory_count,
            inhibitory_count,
        })
    }

    async fn get_total_neuron_count(&self) -> ServiceResult<usize> {
        debug!(target: "feagi-services","Getting total neuron count");
        
        let count = self.connectome.read().get_neuron_count();
        Ok(count)
    }

    async fn get_total_synapse_count(&self) -> ServiceResult<usize> {
        debug!(target: "feagi-services","Getting total synapse count");
        
        let count = self.connectome.read().get_synapse_count();
        Ok(count)
    }

    async fn get_populated_areas(&self) -> ServiceResult<Vec<(String, usize)>> {
        debug!(target: "feagi-services","Getting populated areas");
        
        let areas = self.connectome.read().get_populated_areas();
        Ok(areas)
    }

    async fn get_neuron_density(&self, cortical_id: &str) -> ServiceResult<f32> {
        debug!(target: "feagi-services","Getting neuron density for area: {}", cortical_id);
        
        let density = self.connectome.read().get_neuron_density(cortical_id);
        Ok(density)
    }

    async fn is_brain_initialized(&self) -> ServiceResult<bool> {
        debug!(target: "feagi-services","Checking if brain is initialized");
        
        let initialized = self.connectome.read().is_initialized();
        Ok(initialized)
    }

    async fn is_burst_engine_ready(&self) -> ServiceResult<bool> {
        debug!(target: "feagi-services","Checking if burst engine is ready");
        
        let ready = self.connectome.read().has_npu();
        Ok(ready)
    }
}

