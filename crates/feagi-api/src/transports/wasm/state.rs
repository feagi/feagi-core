// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! ApiState builder for WASM
//!
//! Creates ApiState from RuntimeGenome with WASM-specific service implementations.

use crate::transports::http::server::ApiState;
use crate::transports::wasm::services::*;
use feagi_evolutionary::RuntimeGenome;
use std::sync::Arc;

/// Create ApiState from RuntimeGenome
///
/// This function creates an ApiState instance with WASM-specific service
/// implementations that extract data from the RuntimeGenome.
pub fn create_api_state_from_genome(genome: Arc<RuntimeGenome>) -> ApiState {
    let connectome_service = Arc::new(WasmConnectomeService::new(Arc::clone(&genome)));
    let genome_service = Arc::new(WasmGenomeService::new(Arc::clone(&genome)));
    let analytics_service = Arc::new(WasmAnalyticsService::new(Arc::clone(&genome)));
    let runtime_service = Arc::new(WasmRuntimeService::new());
    let neuron_service = Arc::new(WasmNeuronService::new());
    let system_service = Arc::new(WasmSystemService::new());

    ApiState {
        agent_service: None, // No agents in WASM standalone mode
        genome_service: genome_service
            as Arc<dyn feagi_services::traits::GenomeService + Send + Sync>,
        connectome_service: connectome_service
            as Arc<dyn feagi_services::traits::ConnectomeService + Send + Sync>,
        analytics_service: analytics_service
            as Arc<dyn feagi_services::traits::AnalyticsService + Send + Sync>,
        runtime_service: runtime_service
            as Arc<dyn feagi_services::traits::RuntimeService + Send + Sync>,
        neuron_service: neuron_service
            as Arc<dyn feagi_services::traits::NeuronService + Send + Sync>,
        system_service: system_service
            as Arc<dyn feagi_services::traits::SystemService + Send + Sync>,
        snapshot_service: None, // TODO: Implement if needed
        feagi_session_timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64,
        memory_stats_cache: None,
        amalgamation_state: ApiState::init_amalgamation_state(),
        #[cfg(feature = "feagi-agent")]
        agent_connectors: ApiState::init_agent_connectors(),
        #[cfg(feature = "feagi-agent")]
        agent_registration_handler: ApiState::init_agent_registration_handler(),
    }
}
