// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! `ApiState` builder for the genome-backed services.
//!
//! Every service shares the same [`SharedGenome`] handle, so a genome reload through
//! `/v1/genome/*` is visible to all of them without rebuilding the router.

use crate::services::*;
use crate::transports::http::server::ApiState;
use std::sync::Arc;

/// Builds an [`ApiState`] whose services read from `genome` and drive `npu`.
///
/// `npu` is `None` for a server running without an engine, in which case engine-dependent
/// endpoints report the NPU as unavailable. `version_info` is supplied by the caller because only
/// the final binary knows which crate versions were linked into it.
pub fn create_api_state_from_genome(
    genome: SharedGenome,
    npu: OptionalNpu,
    version_info: feagi_services::types::VersionInfo,
) -> ApiState {
    let connectome_service = Arc::new(GenomeConnectomeService::new(
        Arc::clone(&genome),
        npu.clone(),
    ));
    let genome_service = Arc::new(GenomeGenomeService::new(Arc::clone(&genome), npu.clone()));
    let analytics_service = Arc::new(GenomeAnalyticsService::new(Arc::clone(&genome)));
    let runtime_service = Arc::new(GenomeRuntimeService::new(npu));
    let neuron_service = Arc::new(GenomeNeuronService::new());
    let system_service = Arc::new(GenomeSystemService::new(Arc::clone(&genome), version_info));

    let (genome_transition_lock, genome_transition_in_progress) =
        ApiState::init_genome_transition_controls();
    let filesystem_data_root = ApiState::filesystem_data_root_from_config(std::path::Path::new(""));
    ApiState {
        network_connection_info_provider: None,
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
        filesystem_data_root,
        memory_stats_cache: None,
        amalgamation_state: ApiState::init_amalgamation_state(),
        genome_transition_lock,
        genome_transition_in_progress,
        #[cfg(feature = "feagi-agent")]
        agent_handler: Some(ApiState::init_agent_registration_handler()),
    }
}
