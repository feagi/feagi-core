// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// OpenAPI documentation generation
//
// This module generates the OpenAPI 3.0 specification at compile-time
// using utoipa, ensuring the documentation stays in sync with the code.

use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::OpenApi;

use crate::{
    common::ApiError,
    v1::{
        BrainRegionDetail,
        BrainRegionListResponse,
        // Brain region DTOs
        BrainRegionSummary,
        BurstCountResponse,

        ComponentReadiness,

        ConnectivityStatsResponse,
        ConnectomeAnalyticsResponse,
        Coordinates3D,
        CorticalAreaDetail,
        CorticalAreaListResponse,
        CorticalAreaStatsResponse,
        // Cortical area DTOs
        CorticalAreaSummary,
        CreateBrainRegionRequest,

        CreateCorticalAreaRequest,
        CreateNeuronRequest,
        Dimensions3D,

        // Genome DTOs
        GenomeInfoResponse,
        // Health DTOs
        HealthCheckResponseV1,
        LoadGenomeRequest,
        NeuronCountResponse,

        NeuronDensityResponse,
        // Neuron DTOs
        NeuronInfoResponse,
        NeuronListResponse,
        PopulatedAreaInfo,
        PopulatedAreasResponse,
        ReadinessCheckResponseV1,
        // Runtime DTOs
        RuntimeStatusResponse,
        SaveGenomeRequest,
        SaveGenomeResponse,
        SetFrequencyRequest,
        // Analytics DTOs
        SystemHealthResponse,
        UpdateCorticalAreaRequest,
        ValidateGenomeRequest,
        ValidateGenomeResponse,
    },
};

/// OpenAPI documentation for FEAGI REST API
#[derive(OpenApi)]
#[openapi(
    info(
        title = "FEAGI REST API",
        version = "1.0.0",
        description = "Comprehensive REST API for FEAGI (Foundational Engine for Artificial General Intelligence)",
        license(
            name = "Apache-2.0",
            url = "https://www.apache.org/licenses/LICENSE-2.0"
        ),
        contact(
            name = "FEAGI Team",
            url = "https://feagi.org",
            email = "contact@feagi.org"
        )
    ),
    servers(
        (url = "http://localhost:8000", description = "Default FEAGI server"),
        (url = "http://localhost:8080", description = "Alternative port")
    ),
    paths(
        // Agent endpoints (14 total)
        crate::endpoints::agent::register_agent,
        crate::endpoints::agent::heartbeat,
        crate::endpoints::agent::list_agents,
        crate::endpoints::agent::get_agent_properties,
        crate::endpoints::agent::get_agent_properties_path,
        crate::endpoints::agent::get_shared_memory,
        crate::endpoints::agent::deregister_agent,
        crate::endpoints::agent::manual_stimulation,
        crate::endpoints::agent::get_fq_sampler_status,
        crate::endpoints::agent::get_capabilities,
        crate::endpoints::agent::get_agent_info,
        crate::endpoints::agent::post_configure,
        crate::endpoints::agent::export_device_registrations,
        crate::endpoints::agent::import_device_registrations,

        // System endpoints (21 total)
        crate::endpoints::system::get_health_check,
        crate::endpoints::system::get_cortical_area_visualization_skip_rate,
        crate::endpoints::system::set_cortical_area_visualization_skip_rate,
        crate::endpoints::system::get_cortical_area_visualization_suppression_threshold,
        crate::endpoints::system::set_cortical_area_visualization_suppression_threshold,
        crate::endpoints::system::get_version,
        crate::endpoints::system::get_versions,
        crate::endpoints::system::get_configuration,
        crate::endpoints::system::get_user_preferences,
        crate::endpoints::system::put_user_preferences,
        crate::endpoints::system::get_cortical_area_types_list,
        crate::endpoints::system::post_enable_visualization_fq_sampler,
        crate::endpoints::system::post_disable_visualization_fq_sampler,
        crate::endpoints::system::get_fcl_status_system,
        crate::endpoints::system::post_fcl_reset_system,
        crate::endpoints::system::get_processes,
        crate::endpoints::system::get_unique_logs,
        crate::endpoints::system::post_logs,
        crate::endpoints::system::get_beacon_subscribers,
        crate::endpoints::system::post_beacon_subscribe,
        crate::endpoints::system::delete_beacon_unsubscribe,
        crate::endpoints::system::get_global_activity_visualization,
        crate::endpoints::system::put_global_activity_visualization,
        crate::endpoints::system::post_circuit_library_path,
        crate::endpoints::system::get_influxdb_test,
        crate::endpoints::system::post_register_system,

        // Cortical Area endpoints
        crate::endpoints::cortical_area::get_ipu,
        crate::endpoints::cortical_area::get_ipu_types,
        crate::endpoints::cortical_area::get_opu,
        crate::endpoints::cortical_area::get_opu_types,
        crate::endpoints::cortical_area::get_cortical_area_id_list,
        crate::endpoints::cortical_area::get_cortical_area_name_list,
        crate::endpoints::cortical_area::get_cortical_id_name_mapping,
        crate::endpoints::cortical_area::get_cortical_types,
        crate::endpoints::cortical_area::get_cortical_map_detailed,
        crate::endpoints::cortical_area::get_cortical_locations_2d,
        crate::endpoints::cortical_area::get_cortical_area_geometry,
        crate::endpoints::cortical_area::get_cortical_visibility,
        crate::endpoints::cortical_area::post_cortical_name_location,
        crate::endpoints::cortical_area::post_cortical_area_properties,
        crate::endpoints::cortical_area::post_multi_cortical_area_properties,
        crate::endpoints::cortical_area::post_cortical_area,
        crate::endpoints::cortical_area::put_cortical_area,
        crate::endpoints::cortical_area::delete_cortical_area,
        crate::endpoints::cortical_area::post_custom_cortical_area,
        crate::endpoints::cortical_area::post_clone,
        crate::endpoints::cortical_area::put_multi_cortical_area,
        crate::endpoints::cortical_area::delete_multi_cortical_area,
        crate::endpoints::cortical_area::put_coord_2d,
        crate::endpoints::cortical_area::put_suppress_cortical_visibility,
        crate::endpoints::cortical_area::put_reset,
        crate::endpoints::cortical_area::get_visualization,
        crate::endpoints::cortical_area::post_batch_operations,
        crate::endpoints::cortical_area::get_ipu_list,
        crate::endpoints::cortical_area::get_opu_list,
        crate::endpoints::cortical_area::put_coordinates_3d,
        crate::endpoints::cortical_area::delete_bulk,
        crate::endpoints::cortical_area::post_clone,
        crate::endpoints::cortical_area::post_resize,
        crate::endpoints::cortical_area::post_reposition,
        crate::endpoints::cortical_area::post_voxel_neurons,
        crate::endpoints::cortical_area::get_cortical_area_index_list,
        crate::endpoints::cortical_area::get_cortical_idx_mapping,
        crate::endpoints::cortical_area::get_mapping_restrictions_query,
        crate::endpoints::cortical_area::get_memory_usage,
        crate::endpoints::cortical_area::get_area_neuron_count,
        crate::endpoints::cortical_area::post_cortical_type_options,
        crate::endpoints::cortical_area::post_mapping_restrictions,
        crate::endpoints::cortical_area::post_mapping_restrictions_between_areas,
        crate::endpoints::cortical_area::put_coord_3d,

        // Morphology endpoints
        crate::endpoints::morphology::get_morphology_list,
        crate::endpoints::morphology::get_morphology_types,
        crate::endpoints::morphology::get_list_types,
        crate::endpoints::morphology::get_morphologies,
        crate::endpoints::morphology::post_morphology,
        crate::endpoints::morphology::put_morphology,
        crate::endpoints::morphology::delete_morphology_by_name,
        crate::endpoints::morphology::delete_morphology,  // This is delete_by_id
        crate::endpoints::morphology::post_morphology_properties,
        crate::endpoints::morphology::post_morphology_usage,
        crate::endpoints::morphology::get_list,
        crate::endpoints::morphology::get_info,
        crate::endpoints::morphology::post_create,
        crate::endpoints::morphology::put_update,

        // Genome endpoints (22 total)
        crate::endpoints::genome::get_file_name,
        crate::endpoints::genome::get_circuits,
        crate::endpoints::genome::post_amalgamation_destination,
        crate::endpoints::genome::delete_amalgamation_cancellation,
        crate::endpoints::genome::post_genome_append,
        crate::endpoints::genome::post_upload_barebones_genome,
        crate::endpoints::genome::post_upload_essential_genome,
        crate::endpoints::genome::get_name,
        crate::endpoints::genome::get_timestamp,
        crate::endpoints::genome::post_save,
        crate::endpoints::genome::post_load,
        crate::endpoints::genome::post_upload,
        crate::endpoints::genome::get_download,
        crate::endpoints::genome::get_properties,
        crate::endpoints::genome::post_validate,
        crate::endpoints::genome::post_transform,
        crate::endpoints::genome::post_clone,
        crate::endpoints::genome::post_reset,
        crate::endpoints::genome::get_metadata,
        crate::endpoints::genome::post_merge,
        crate::endpoints::genome::get_diff,
        crate::endpoints::genome::post_export_format,
        crate::endpoints::genome::get_amalgamation,
        crate::endpoints::genome::get_amalgamation_history_exact,
        crate::endpoints::genome::get_cortical_template,
        crate::endpoints::genome::get_defaults_files,
        crate::endpoints::genome::get_download_region,
        crate::endpoints::genome::get_genome_number,
        crate::endpoints::genome::post_amalgamation_by_filename,
        crate::endpoints::genome::post_amalgamation_by_payload,
        crate::endpoints::genome::post_amalgamation_by_upload,
        crate::endpoints::genome::post_append_file,
        crate::endpoints::genome::post_upload_file,
        crate::endpoints::genome::post_upload_file_edit,
        crate::endpoints::genome::post_upload_string,

        // Cortical Mapping endpoints (8 total)
        crate::endpoints::cortical_mapping::post_afferents,
        crate::endpoints::cortical_mapping::post_efferents,
        crate::endpoints::cortical_mapping::post_mapping_properties,
        crate::endpoints::cortical_mapping::put_mapping_properties,
        crate::endpoints::cortical_mapping::get_mapping,
        crate::endpoints::cortical_mapping::get_mapping_list,
        crate::endpoints::cortical_mapping::delete_mapping,
        crate::endpoints::cortical_mapping::post_batch_update,
        crate::endpoints::cortical_mapping::post_mapping,
        crate::endpoints::cortical_mapping::put_mapping,

        // Region endpoints (12 total)
        crate::endpoints::region::get_regions_members,
        crate::endpoints::region::post_region,
        crate::endpoints::region::put_region,
        crate::endpoints::region::delete_region,
        crate::endpoints::region::post_clone,
        crate::endpoints::region::put_relocate_members,
        crate::endpoints::region::delete_region_and_members,
        crate::endpoints::region::get_regions,
        crate::endpoints::region::get_region_titles,
        crate::endpoints::region::get_region_detail,
        crate::endpoints::region::put_change_region_parent,
        crate::endpoints::region::put_change_cortical_area_region,

        // Connectome endpoints (21 total)
        crate::endpoints::connectome::get_cortical_areas_list_detailed,
        crate::endpoints::connectome::get_properties_dimensions,
        crate::endpoints::connectome::get_properties_mappings,
        crate::endpoints::connectome::get_snapshot,
        crate::endpoints::connectome::get_stats,
        crate::endpoints::connectome::post_batch_neuron_operations,
        crate::endpoints::connectome::post_batch_synapse_operations,
        crate::endpoints::connectome::get_neuron_count,
        crate::endpoints::connectome::get_synapse_count,
        crate::endpoints::connectome::get_paths,
        crate::endpoints::connectome::get_cumulative_stats,
        crate::endpoints::connectome::get_area_details,
        crate::endpoints::connectome::post_rebuild,
        crate::endpoints::connectome::get_structure,
        crate::endpoints::connectome::post_clear,
        crate::endpoints::connectome::get_validation,
        crate::endpoints::connectome::get_topology,
        crate::endpoints::connectome::post_optimize,
        crate::endpoints::connectome::get_connectivity_matrix,
        crate::endpoints::connectome::post_neurons_batch,
        crate::endpoints::connectome::post_synapses_batch,
        crate::endpoints::connectome::get_cortical_areas_list_summary,
        crate::endpoints::connectome::get_cortical_areas_list_transforming,
        crate::endpoints::connectome::get_cortical_area_list_types,
        crate::endpoints::connectome::get_cortical_area_neurons,
        crate::endpoints::connectome::get_area_synapses,
        crate::endpoints::connectome::get_cortical_info,
        crate::endpoints::connectome::get_stats_cortical_cumulative,
        crate::endpoints::connectome::get_neuron_properties_by_id,
        crate::endpoints::connectome::get_neuron_properties_query,
        crate::endpoints::connectome::get_area_neurons_query,
        crate::endpoints::connectome::get_fire_queue_area,
        crate::endpoints::connectome::get_plasticity_info,
        crate::endpoints::connectome::get_path_query,
        crate::endpoints::connectome::get_download_connectome,
        crate::endpoints::connectome::get_download_cortical_area,
        crate::endpoints::connectome::post_upload_connectome,
        crate::endpoints::connectome::post_upload_cortical_area,

        // Burst Engine endpoints (14 total)
        crate::endpoints::burst_engine::get_simulation_timestep,
        crate::endpoints::burst_engine::post_simulation_timestep,
        crate::endpoints::burst_engine::get_fcl,
        crate::endpoints::burst_engine::get_fire_queue,
        crate::endpoints::burst_engine::post_fcl_reset,
        crate::endpoints::burst_engine::get_fcl_status,
        crate::endpoints::burst_engine::get_fcl_sampler_config,
        crate::endpoints::burst_engine::post_fcl_sampler_config,
        crate::endpoints::burst_engine::get_area_fcl_sample_rate,
        crate::endpoints::burst_engine::post_area_fcl_sample_rate,
        crate::endpoints::burst_engine::get_fire_ledger_default_window_size,
        crate::endpoints::burst_engine::put_fire_ledger_default_window_size,
        crate::endpoints::burst_engine::get_fire_ledger_areas_window_config,
        crate::endpoints::burst_engine::get_stats,
        crate::endpoints::burst_engine::get_status,
        crate::endpoints::burst_engine::post_control,
        crate::endpoints::burst_engine::get_burst_counter,
        crate::endpoints::burst_engine::post_start,
        crate::endpoints::burst_engine::post_stop,
        crate::endpoints::burst_engine::post_hold,
        crate::endpoints::burst_engine::post_resume,
        crate::endpoints::burst_engine::get_config,
        crate::endpoints::burst_engine::put_config,
        crate::endpoints::burst_engine::get_fire_ledger_area_window_size,
        crate::endpoints::burst_engine::put_fire_ledger_area_window_size,
        crate::endpoints::burst_engine::get_fire_ledger_history,
        crate::endpoints::burst_engine::get_membrane_potentials,
        crate::endpoints::burst_engine::put_membrane_potentials,
        crate::endpoints::burst_engine::get_frequency_status,
        crate::endpoints::burst_engine::post_measure_frequency,
        crate::endpoints::burst_engine::get_frequency_history,
        crate::endpoints::burst_engine::post_force_connectome_integration,

        // Insight endpoints
        crate::endpoints::insight::post_neurons_membrane_potential_status,
        crate::endpoints::insight::post_neuron_synaptic_potential_status,
        crate::endpoints::insight::post_neurons_membrane_potential_set,
        crate::endpoints::insight::post_neuron_synaptic_potential_set,

        // Neuroplasticity endpoints (7 total)
        crate::endpoints::neuroplasticity::get_plasticity_queue_depth,
        crate::endpoints::neuroplasticity::put_plasticity_queue_depth,
        crate::endpoints::neuroplasticity::get_status,
        crate::endpoints::neuroplasticity::get_transforming,
        crate::endpoints::neuroplasticity::post_configure,
        crate::endpoints::neuroplasticity::post_enable_area,
        crate::endpoints::neuroplasticity::post_disable_area,

        // Input endpoints
        crate::endpoints::input::get_vision,
        crate::endpoints::input::post_vision,

        // Outputs endpoints
        crate::endpoints::outputs::get_targets,
        crate::endpoints::outputs::post_configure,

        // Physiology endpoints
        crate::endpoints::physiology::get_physiology,
        crate::endpoints::physiology::put_physiology,

        // Simulation endpoints
        crate::endpoints::simulation::post_stimulation_upload,
        crate::endpoints::simulation::post_reset,
        crate::endpoints::simulation::get_status,
        crate::endpoints::simulation::get_stats,
        crate::endpoints::simulation::post_config,

        // Training endpoints
        crate::endpoints::training::post_shock,
        crate::endpoints::training::get_shock_options,
        crate::endpoints::training::get_shock_status,
        crate::endpoints::training::post_reward_intensity,
        crate::endpoints::training::post_punishment_intensity,
        crate::endpoints::training::post_gameover,
        crate::endpoints::training::get_brain_fitness,
        crate::endpoints::training::get_fitness_criteria,
        crate::endpoints::training::put_fitness_criteria,
        crate::endpoints::training::get_fitness_stats,
        crate::endpoints::training::get_training_report,
        crate::endpoints::training::get_status,
        crate::endpoints::training::get_stats,
        crate::endpoints::training::post_config,

        // Visualization endpoints
        crate::endpoints::visualization::post_register_client,
        crate::endpoints::visualization::post_unregister_client,
        crate::endpoints::visualization::post_heartbeat,
        crate::endpoints::visualization::get_status,

        // Monitoring endpoints
        crate::endpoints::monitoring::get_status,
        crate::endpoints::monitoring::get_metrics,
        crate::endpoints::monitoring::get_data,

        // Evolution endpoints
        crate::endpoints::evolution::get_status,
        crate::endpoints::evolution::post_config,

        // Snapshot endpoints
        // TODO: Implement snapshot endpoints
        // crate::endpoints::snapshot::post_create,
        // crate::endpoints::snapshot::post_restore,
        // crate::endpoints::snapshot::get_list,
        // crate::endpoints::snapshot::delete_snapshot,
        // crate::endpoints::snapshot::get_artifact,
        // crate::endpoints::snapshot::post_compare,
        // crate::endpoints::snapshot::post_upload,

        // Network endpoints
        crate::endpoints::network::get_status,
        crate::endpoints::network::post_config,
    ),
    components(
        schemas(
            // Common
            ApiError,

            // Health
            HealthCheckResponseV1,
            ReadinessCheckResponseV1,
            ComponentReadiness,

            // Agent
            crate::v1::AgentRegistrationRequest,

            // Outputs
            crate::v1::OutputTargetsResponse,
            crate::v1::OutputConfigRequest,
            crate::v1::OutputConfigResponse,

            // Physiology
            crate::v1::PhysiologyResponse,
            crate::v1::PhysiologyParameters,
            crate::v1::PhysiologyUpdateRequest,
            crate::v1::PhysiologyUpdateResponse,

            // Burst Engine
            crate::v1::FCLResponse,
            crate::v1::FireQueueResponse,
            crate::v1::FCLStatusResponse,
            crate::v1::FireLedgerConfigResponse,
            crate::v1::BurstEngineStats,
            crate::v1::BurstEngineStatus,
            crate::v1::BurstEngineControlRequest,

            // Monitoring
            crate::v1::MonitoringStatusResponse,
            crate::v1::SystemMetricsResponse,
            crate::v1::MonitoringData,
            crate::v1::MonitoringDataResponse,

            // Simulation
            crate::v1::StimulationUploadRequest,
            crate::v1::SimulationControlRequest,
            crate::v1::SimulationStatusResponse,
            crate::v1::SimulationStatsResponse,
            crate::v1::SimulationSuccessResponse,

            // Training
            crate::v1::ShockConfigRequest,
            crate::v1::ShockOptionsResponse,
            crate::v1::ShockStatusResponse,
            crate::v1::IntensityRequest,
            crate::v1::BrainFitnessResponse,
            crate::v1::FitnessCriteriaResponse,
            crate::v1::FitnessCriteriaUpdateRequest,
            crate::v1::FitnessStatsResponse,
            crate::v1::TrainingReportResponse,
            crate::v1::TrainingStatusResponse,
            crate::v1::TrainingStatsResponse,
            crate::v1::TrainingConfigRequest,
            crate::v1::TrainingSuccessResponse,

            // Visualization
            crate::v1::VisualizationClientRequest,
            crate::v1::VisualizationClientResponse,
            crate::v1::VisualizationHeartbeatRequest,
            crate::v1::VisualizationStatusResponse,
            crate::v1::VisualizationSuccessResponse,

            // Evolution
            crate::v1::EvolutionStatusResponse,
            crate::v1::EvolutionConfigRequest,
            crate::v1::EvolutionSuccessResponse,

            // Snapshot
            crate::v1::SnapshotCreateRequest,
            crate::v1::SnapshotCreateResponse,
            crate::v1::SnapshotRestoreRequest,
            crate::v1::SnapshotListResponse,
            crate::v1::SnapshotInfo,
            crate::v1::SnapshotArtifactResponse,
            crate::v1::SnapshotCompareRequest,
            crate::v1::SnapshotCompareResponse,
            crate::v1::SnapshotUploadRequest,
            crate::v1::SnapshotUploadResponse,
            crate::v1::SnapshotSuccessResponse,

            // Network
            crate::v1::NetworkStatusResponse,
            crate::v1::NetworkConfigRequest,
            crate::v1::NetworkSuccessResponse,
            crate::v1::AgentRegistrationResponse,
            crate::v1::HeartbeatRequest,
            crate::v1::HeartbeatResponse,
            crate::v1::AgentListResponse,
            crate::v1::AgentPropertiesResponse,
            crate::v1::AgentDeregistrationRequest,
            crate::v1::SuccessResponse,
            crate::v1::ManualStimulationRequest,
            crate::v1::ManualStimulationResponse,
            crate::v1::DeviceRegistrationExportResponse,
            crate::v1::DeviceRegistrationImportRequest,
            crate::v1::DeviceRegistrationImportResponse,

            // System
            crate::endpoints::system::HealthCheckResponse,

            // Cortical Area
            crate::endpoints::cortical_area::CorticalAreaIdListResponse,
            crate::endpoints::cortical_area::CorticalAreaNameListResponse,
            crate::endpoints::cortical_area::CorticalTypeMetadata,

            // Morphology
            crate::endpoints::morphology::MorphologyListResponse,

            // Cortical Areas
            CorticalAreaSummary,
            CorticalAreaDetail,
            CorticalAreaListResponse,
            CreateCorticalAreaRequest,
            UpdateCorticalAreaRequest,
            Coordinates3D,
            Dimensions3D,

            // Brain Regions
            BrainRegionSummary,
            BrainRegionDetail,
            BrainRegionListResponse,
            CreateBrainRegionRequest,

            // Genome
            GenomeInfoResponse,
            LoadGenomeRequest,
            SaveGenomeRequest,
            SaveGenomeResponse,
            ValidateGenomeRequest,
            ValidateGenomeResponse,

            // Neurons
            NeuronInfoResponse,
            CreateNeuronRequest,
            NeuronListResponse,
            NeuronCountResponse,

            // Runtime
            RuntimeStatusResponse,
            SetFrequencyRequest,
            BurstCountResponse,

            // Analytics
            SystemHealthResponse,
            CorticalAreaStatsResponse,
            ConnectivityStatsResponse,
            ConnectomeAnalyticsResponse,
            PopulatedAreasResponse,
            PopulatedAreaInfo,
            NeuronDensityResponse,
        )
    ),
    tags(
        (name = "agent", description = "Agent registration and heartbeat"),
        (name = "system", description = "System health and configuration"),
        (name = "cortical_area", description = "Cortical area management"),
        (name = "morphology", description = "Morphology management"),
        (name = "genome", description = "Genome operations"),
        (name = "cortical_mapping", description = "Cortical mapping operations"),
        (name = "region", description = "Brain region management"),
        (name = "connectome", description = "Connectome operations"),
        (name = "burst_engine", description = "Burst engine configuration and FCL/Fire Queue"),
        (name = "insight", description = "Neuron insight operations"),
        (name = "neuroplasticity", description = "Neuroplasticity configuration"),
        (name = "input", description = "Input operations"),
        (name = "outputs", description = "Output/motor target management"),
        (name = "physiology", description = "Physiology parameter configuration"),
        (name = "simulation", description = "Simulation control and stimulation"),
        (name = "training", description = "Reinforcement learning and training"),
        (name = "visualization", description = "Visualization client management"),
        (name = "monitoring", description = "System monitoring and metrics"),
        (name = "evolution", description = "Evolutionary algorithms"),
        (name = "snapshot", description = "Brain snapshot management"),
        (name = "network", description = "Network configuration"),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Security scheme configuration
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            // API Key authentication (for future use)
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-API-Key"))),
            );

            // JWT Bearer authentication (for future use)
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

/// Get OpenAPI JSON specification
pub fn get_openapi_json() -> String {
    ApiDoc::openapi()
        .to_pretty_json()
        .unwrap_or_else(|e| format!(r#"{{"error": "Failed to generate OpenAPI spec: {}"}}"#, e))
}

/// Get OpenAPI YAML specification
pub fn get_openapi_yaml() -> String {
    // utoipa supports YAML output, but we need to implement it
    // For now, return a notice
    "# OpenAPI YAML generation not yet implemented\n# Use /openapi.json instead".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_generation() {
        let json = get_openapi_json();
        assert!(json.contains("FEAGI REST API"));
        assert!(json.contains("health"));
        assert!(json.contains("ready"));
    }

    #[test]
    fn test_openapi_components() {
        let openapi = ApiDoc::openapi();
        assert!(openapi.components.is_some());

        let components = openapi.components.unwrap();
        assert!(components.schemas.contains_key("HealthCheckResponseV1"));
        assert!(components.schemas.contains_key("ApiError"));
    }

    #[test]
    fn test_security_schemes() {
        let openapi = ApiDoc::openapi();
        let components = openapi.components.unwrap();

        assert!(components.security_schemes.contains_key("api_key"));
        assert!(components.security_schemes.contains_key("bearer_auth"));
    }
}
