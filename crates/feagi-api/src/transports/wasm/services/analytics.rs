// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WASM Analytics Service (stub)

use async_trait::async_trait;
use feagi_services::traits::analytics_service::AnalyticsService;
use feagi_services::types::errors::{ServiceError, ServiceResult};
use feagi_services::types::*;
use feagi_evo::RuntimeGenome;
use std::sync::Arc;

pub struct WasmAnalyticsService {
    genome: Arc<RuntimeGenome>,
}

impl WasmAnalyticsService {
    pub fn new(genome: Arc<RuntimeGenome>) -> Self {
        Self { genome }
    }
}

#[async_trait]
impl AnalyticsService for WasmAnalyticsService {
    async fn get_system_health(&self) -> ServiceResult<SystemHealth> {
        // Return minimal health check for WASM
        Ok(SystemHealth {
            burst_engine: true,
            connected_agents: Some(0), // No agents in WASM standalone mode
            influxdb_availability: false,
            neuron_count_max: 0,
            synapse_count_max: 0,
            latest_changes_saved_externally: false,
            genome_availability: true,
            genome_validity: Some(true),
            brain_readiness: true,
            feagi_session: None,
            fitness: None,
            cortical_area_count: Some(self.genome.cortical_areas.len() as i32),
            neuron_count: None,
            memory_neuron_count: None,
            regular_neuron_count: None,
            synapse_count: None,
            estimated_brain_size_in_MB: None,
            genome_num: None,
            genome_timestamp: Some(self.genome.metadata.timestamp as i64),
            simulation_timestep: None,
            memory_area_stats: None,
            amalgamation_pending: None,
            brain_regions_root: self.genome.metadata.brain_regions_root.clone(),
        })
    }
}

