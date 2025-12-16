// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// HTTP server implementation (Axum)
//
// This module sets up the HTTP API server with Axum, including routing,
// middleware, and state management.

use axum::{
    extract::State,
    http::{Request, StatusCode},
    response::{Html, IntoResponse, Json, Redirect, Response},
    routing::{get, put},
    middleware::{self, Next},
    body::Body,
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use utoipa::OpenApi;
use http_body_util::BodyExt;

#[cfg(feature = "http")]
use crate::openapi::ApiDoc;
#[cfg(feature = "services")]
use feagi_services::{AnalyticsService, ConnectomeService, GenomeService, NeuronService, RuntimeService};
#[cfg(feature = "services")]
use feagi_services::traits::{AgentService, SystemService};

/// Application state shared across all HTTP handlers
#[derive(Clone)]
pub struct ApiState {
    pub agent_service: Option<Arc<dyn AgentService + Send + Sync>>,
    pub analytics_service: Arc<dyn AnalyticsService + Send + Sync>,
    pub connectome_service: Arc<dyn ConnectomeService + Send + Sync>,
    pub genome_service: Arc<dyn GenomeService + Send + Sync>,
    pub neuron_service: Arc<dyn NeuronService + Send + Sync>,
    pub runtime_service: Arc<dyn RuntimeService + Send + Sync>,
    pub system_service: Arc<dyn SystemService + Send + Sync>,
    pub snapshot_service: Option<Arc<dyn feagi_services::SnapshotService + Send + Sync>>,
    /// FEAGI session timestamp in milliseconds (Unix timestamp when FEAGI started)
    /// This is a unique identifier for each FEAGI instance/session
    pub feagi_session_timestamp: i64,
}

/// Create the main HTTP server application
pub fn create_http_server(state: ApiState) -> Router {
    Router::new()
        // Root redirect to custom Swagger UI
        .route("/", get(root_redirect))
        
        // Custom Swagger UI with FEAGI branding at /swagger-ui/
        .route("/swagger-ui/", get(custom_swagger_ui))
        
        // OpenAPI spec endpoint
        .route("/api-docs/openapi.json", get(|| async {
            Json(ApiDoc::openapi())
        }))
        
        // Python-compatible paths: /v1/* (ONLY this, matching Python exactly)
        .nest("/v1", create_v1_router())
        
        // Catch-all route for debugging unmatched requests
        .fallback(|| async {
            tracing::warn!(target: "feagi-api", "⚠️ Unmatched request - 404 Not Found");
            (StatusCode::NOT_FOUND, "404 Not Found")
        })
        
        // Add state
        .with_state(state)
        
        // Add middleware
        .layer(middleware::from_fn(log_request_response_bodies))
        .layer(create_cors_layer())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    tracing::span!(
                        target: "feagi-api",
                        tracing::Level::DEBUG,
                        "request",
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                    )
                })
                .on_request(|request: &axum::http::Request<_>, _span: &tracing::Span| {
                    tracing::debug!(target: "feagi-api", "📥 Incoming request: {} {}", request.method(), request.uri());
                })
                .on_response(|response: &axum::http::Response<_>, latency: std::time::Duration, span: &tracing::Span| {
                    tracing::debug!(
                        target: "feagi-api",
                        "📤 Response: status={}, latency={:?}",
                        response.status(),
                        latency
                    );
                    span.record("status", response.status().as_u16());
                    span.record("latency_ms", latency.as_millis());
                })
                .on_body_chunk(|chunk: &axum::body::Bytes, latency: std::time::Duration, _span: &tracing::Span| {
                    tracing::trace!(target: "feagi-api", "Response chunk: {} bytes, latency={:?}", chunk.len(), latency);
                })
                .on_eos(|_trailers: Option<&axum::http::HeaderMap>, stream_duration: std::time::Duration, _span: &tracing::Span| {
                    tracing::trace!(target: "feagi-api", "Stream ended, duration={:?}", stream_duration);
                })
                .on_failure(|_error: tower_http::classify::ServerErrorsFailureClass, latency: std::time::Duration, _span: &tracing::Span| {
                    tracing::error!(target: "feagi-api", "❌ Request failed, latency={:?}", latency);
                })
        )
}

/// Create V1 API router - Match Python structure EXACTLY
/// Format: /v1/{module}/{snake_case_endpoint}
fn create_v1_router() -> Router<ApiState> {
    use crate::endpoints::{agent, system};
    use crate::endpoints::agent::*;  // Import agent functions for routes
    use crate::endpoints::cortical_area;
    use crate::endpoints::morphology;
    use crate::endpoints::genome;
    use crate::endpoints::cortical_mapping;
    use crate::endpoints::region;
    use crate::endpoints::connectome;
    use crate::endpoints::burst_engine;
    use crate::endpoints::insight;
    use crate::endpoints::neuroplasticity;
    use crate::endpoints::input;
    use crate::endpoints::outputs;
    use crate::endpoints::physiology;
    use crate::endpoints::simulation;
    use crate::endpoints::training;
    use crate::endpoints::visualization;
    use crate::endpoints::monitoring;
    use crate::endpoints::evolution;
    use crate::endpoints::snapshot;
    use crate::endpoints::network;
    
    Router::new()
        // ===== AGENT MODULE (12 endpoints) =====
        .route("/agent/register", axum::routing::post(register_agent))
        .route("/agent/heartbeat", axum::routing::post(heartbeat))
        .route("/agent/list", get(list_agents))
        .route("/agent/properties", get(get_agent_properties))
        .route("/agent/properties/:agent_id", get(agent::get_agent_properties_path))
        .route("/agent/shared_mem", get(get_shared_memory))
        .route("/agent/deregister", axum::routing::delete(deregister_agent))
        .route("/agent/manual_stimulation", axum::routing::post(manual_stimulation))
        .route("/agent/fq_sampler_status", get(agent::get_fq_sampler_status))
        .route("/agent/capabilities", get(agent::get_capabilities))
        .route("/agent/info/:agent_id", get(agent::get_agent_info))
        .route("/agent/configure", axum::routing::post(agent::post_configure))
        
        // ===== SYSTEM MODULE (21 endpoints) =====
        .route("/system/health_check", get(system::get_health_check))
        .route("/system/cortical_area_visualization_skip_rate", 
            get(system::get_cortical_area_visualization_skip_rate)
            .put(system::set_cortical_area_visualization_skip_rate))
        .route("/system/cortical_area_visualization_suppression_threshold",
            get(system::get_cortical_area_visualization_suppression_threshold)
            .put(system::set_cortical_area_visualization_suppression_threshold))
        .route("/system/version", get(system::get_version))
        .route("/system/versions", get(system::get_versions))
        .route("/system/configuration", get(system::get_configuration))
        .route("/system/user_preferences", get(system::get_user_preferences).put(system::put_user_preferences))
        .route("/system/cortical_area_types", get(system::get_cortical_area_types_list))
        .route("/system/enable_visualization_fq_sampler", axum::routing::post(system::post_enable_visualization_fq_sampler))
        .route("/system/disable_visualization_fq_sampler", axum::routing::post(system::post_disable_visualization_fq_sampler))
        .route("/system/fcl_status", get(system::get_fcl_status_system))
        .route("/system/fcl_reset", axum::routing::post(system::post_fcl_reset_system))
        .route("/system/processes", get(system::get_processes))
        .route("/system/unique_logs", get(system::get_unique_logs))
        .route("/system/logs", axum::routing::post(system::post_logs))
        .route("/system/beacon/subscribers", get(system::get_beacon_subscribers))
        .route("/system/beacon/subscribe", axum::routing::post(system::post_beacon_subscribe))
        .route("/system/beacon/unsubscribe", axum::routing::delete(system::delete_beacon_unsubscribe))
        .route("/system/global_activity_visualization", 
            get(system::get_global_activity_visualization).put(system::put_global_activity_visualization))
        .route("/system/circuit_library_path", axum::routing::post(system::post_circuit_library_path))
        .route("/system/db/influxdb/test", get(system::get_influxdb_test))
        .route("/system/register", axum::routing::post(system::post_register_system))
        
        // ===== CORTICAL_AREA MODULE (25 endpoints) =====
        .route("/cortical_area/ipu", get(cortical_area::get_ipu))
        .route("/cortical_area/ipu/types", get(cortical_area::get_ipu_types))
        .route("/cortical_area/opu", get(cortical_area::get_opu))
        .route("/cortical_area/opu/types", get(cortical_area::get_opu_types))
        .route("/cortical_area/cortical_area_id_list", get(cortical_area::get_cortical_area_id_list))
        .route("/cortical_area/cortical_area_name_list", get(cortical_area::get_cortical_area_name_list))
        .route("/cortical_area/cortical_id_name_mapping", get(cortical_area::get_cortical_id_name_mapping))
        .route("/cortical_area/cortical_types", get(cortical_area::get_cortical_types))
        .route("/cortical_area/cortical_map_detailed", get(cortical_area::get_cortical_map_detailed))
        .route("/cortical_area/cortical_locations_2d", get(cortical_area::get_cortical_locations_2d))
        .route("/cortical_area/cortical_area/geometry", get(cortical_area::get_cortical_area_geometry))
        .route("/cortical_area/cortical_visibility", get(cortical_area::get_cortical_visibility))
        .route("/cortical_area/cortical_name_location", axum::routing::post(cortical_area::post_cortical_name_location))
        .route("/cortical_area/cortical_area_properties", axum::routing::post(cortical_area::post_cortical_area_properties))
        .route("/cortical_area/multi/cortical_area_properties", axum::routing::post(cortical_area::post_multi_cortical_area_properties))
        .route("/cortical_area/cortical_area",
            axum::routing::post(cortical_area::post_cortical_area)
            .put(cortical_area::put_cortical_area)
            .delete(cortical_area::delete_cortical_area))
        .route("/cortical_area/custom_cortical_area", axum::routing::post(cortical_area::post_custom_cortical_area))
        .route("/cortical_area/clone", axum::routing::post(cortical_area::post_clone_area))
        .route("/cortical_area/multi/cortical_area",
            put(cortical_area::put_multi_cortical_area).delete(cortical_area::delete_multi_cortical_area))
        .route("/cortical_area/coord_2d", put(cortical_area::put_coord_2d))
        .route("/cortical_area/suppress_cortical_visibility", put(cortical_area::put_suppress_cortical_visibility))
        .route("/cortical_area/reset", put(cortical_area::put_reset))
        .route("/cortical_area/visualization", get(cortical_area::get_visualization))
        .route("/cortical_area/batch_operations", axum::routing::post(cortical_area::post_batch_operations))
        .route("/cortical_area/ipu/list", get(cortical_area::get_ipu_list))
        .route("/cortical_area/opu/list", get(cortical_area::get_opu_list))
        .route("/cortical_area/coordinates_3d", put(cortical_area::put_coordinates_3d))
        .route("/cortical_area/bulk_delete", axum::routing::delete(cortical_area::delete_bulk))
        .route("/cortical_area/resize", axum::routing::post(cortical_area::post_resize))
        .route("/cortical_area/reposition", axum::routing::post(cortical_area::post_reposition))
        .route("/cortical_area/voxel_neurons", axum::routing::post(cortical_area::post_voxel_neurons))
        .route("/cortical_area/cortical_area_index_list", get(cortical_area::get_cortical_area_index_list))
        .route("/cortical_area/cortical_idx_mapping", get(cortical_area::get_cortical_idx_mapping))
        .route("/cortical_area/mapping_restrictions", get(cortical_area::get_mapping_restrictions_query).post(cortical_area::post_mapping_restrictions))
        .route("/cortical_area/:cortical_id/memory_usage", get(cortical_area::get_memory_usage))
        .route("/cortical_area/:cortical_id/neuron_count", get(cortical_area::get_area_neuron_count))
        .route("/cortical_area/cortical_type_options", axum::routing::post(cortical_area::post_cortical_type_options))
        .route("/cortical_area/mapping_restrictions_between_areas", axum::routing::post(cortical_area::post_mapping_restrictions_between_areas))
        .route("/cortical_area/coord_3d", put(cortical_area::put_coord_3d))
        
        // ===== MORPHOLOGY MODULE (14 endpoints) =====
        .route("/morphology/morphology_list", get(morphology::get_morphology_list))
        .route("/morphology/morphology_types", get(morphology::get_morphology_types))
        .route("/morphology/list/types", get(morphology::get_list_types))
        .route("/morphology/morphologies", get(morphology::get_morphologies))
        .route("/morphology/morphology",
            axum::routing::post(morphology::post_morphology)
            .put(morphology::put_morphology)
            .delete(morphology::delete_morphology_by_name))
        .route("/morphology/morphology_properties", axum::routing::post(morphology::post_morphology_properties))
        .route("/morphology/morphology_usage", axum::routing::post(morphology::post_morphology_usage))
        .route("/morphology/list", get(morphology::get_list))
        .route("/morphology/info/:morphology_id", get(morphology::get_info))
        .route("/morphology/create", axum::routing::post(morphology::post_create))
        .route("/morphology/update", axum::routing::put(morphology::put_update))
        .route("/morphology/delete/:morphology_id", axum::routing::delete(morphology::delete_morphology))
        
        // ===== REGION MODULE (12 endpoints) =====
        .route("/region/regions_members", get(region::get_regions_members))
        .route("/region/region",
            axum::routing::post(region::post_region)
            .put(region::put_region)
            .delete(region::delete_region))
        .route("/region/clone", axum::routing::post(region::post_clone))
        .route("/region/relocate_members", put(region::put_relocate_members))
        .route("/region/region_and_members", axum::routing::delete(region::delete_region_and_members))
        .route("/region/regions", get(region::get_regions))
        .route("/region/region_titles", get(region::get_region_titles))
        .route("/region/region/:region_id", get(region::get_region_detail))
        .route("/region/change_region_parent", put(region::put_change_region_parent))
        .route("/region/change_cortical_area_region", put(region::put_change_cortical_area_region))
        
        // ===== CORTICAL_MAPPING MODULE (8 endpoints) =====
        .route("/cortical_mapping/afferents", axum::routing::post(cortical_mapping::post_afferents))
        .route("/cortical_mapping/efferents", axum::routing::post(cortical_mapping::post_efferents))
        .route("/cortical_mapping/mapping_properties",
            axum::routing::post(cortical_mapping::post_mapping_properties).put(cortical_mapping::put_mapping_properties))
        .route("/cortical_mapping/mapping", get(cortical_mapping::get_mapping).delete(cortical_mapping::delete_mapping))
        .route("/cortical_mapping/mapping_list", get(cortical_mapping::get_mapping_list))
        .route("/cortical_mapping/batch_update", axum::routing::post(cortical_mapping::post_batch_update))
        .route("/cortical_mapping/mapping", axum::routing::post(cortical_mapping::post_mapping).put(cortical_mapping::put_mapping))
        
        // ===== CONNECTOME MODULE (21 endpoints) =====
        .route("/connectome/cortical_areas/list/detailed", get(connectome::get_cortical_areas_list_detailed))
        .route("/connectome/properties/dimensions", get(connectome::get_properties_dimensions))
        .route("/connectome/properties/mappings", get(connectome::get_properties_mappings))
        .route("/connectome/snapshot", get(connectome::get_snapshot))
        .route("/connectome/stats", get(connectome::get_stats))
        .route("/connectome/batch_neuron_operations", axum::routing::post(connectome::post_batch_neuron_operations))
        .route("/connectome/batch_synapse_operations", axum::routing::post(connectome::post_batch_synapse_operations))
        .route("/connectome/neuron_count", get(connectome::get_neuron_count))
        .route("/connectome/synapse_count", get(connectome::get_synapse_count))
        .route("/connectome/paths", get(connectome::get_paths))
        .route("/connectome/cumulative_stats", get(connectome::get_cumulative_stats))
        .route("/connectome/area_details", get(connectome::get_area_details))
        .route("/connectome/rebuild", axum::routing::post(connectome::post_rebuild))
        .route("/connectome/structure", get(connectome::get_structure))
        .route("/connectome/clear", axum::routing::post(connectome::post_clear))
        .route("/connectome/validation", get(connectome::get_validation))
        .route("/connectome/topology", get(connectome::get_topology))
        .route("/connectome/optimize", axum::routing::post(connectome::post_optimize))
        .route("/connectome/connectivity_matrix", get(connectome::get_connectivity_matrix))
        .route("/connectome/neurons/batch", axum::routing::post(connectome::post_neurons_batch))
        .route("/connectome/synapses/batch", axum::routing::post(connectome::post_synapses_batch))
        .route("/connectome/cortical_areas/list/summary", get(connectome::get_cortical_areas_list_summary))
        .route("/connectome/cortical_areas/list/transforming", get(connectome::get_cortical_areas_list_transforming))
        .route("/connectome/cortical_area/list/types", get(connectome::get_cortical_area_list_types))
        .route("/connectome/cortical_area/:cortical_id/neurons", get(connectome::get_cortical_area_neurons))
        .route("/connectome/:cortical_area_id/synapses", get(connectome::get_area_synapses))
        .route("/connectome/cortical_info/:cortical_area", get(connectome::get_cortical_info))
        .route("/connectome/stats/cortical/cumulative/:cortical_area", get(connectome::get_stats_cortical_cumulative))
        .route("/connectome/neuron/:neuron_id/properties", get(connectome::get_neuron_properties_by_id))
        .route("/connectome/neuron_properties", get(connectome::get_neuron_properties_query))
        .route("/connectome/area_neurons", get(connectome::get_area_neurons_query))
        .route("/connectome/fire_queue/:cortical_area", get(connectome::get_fire_queue_area))
        .route("/connectome/plasticity", get(connectome::get_plasticity_info))
        .route("/connectome/path", get(connectome::get_path_query))
        .route("/connectome/download", get(connectome::get_download_connectome))
        .route("/connectome/download-cortical-area/:cortical_area", get(connectome::get_download_cortical_area))
        .route("/connectome/upload", axum::routing::post(connectome::post_upload_connectome))
        .route("/connectome/upload-cortical-area", axum::routing::post(connectome::post_upload_cortical_area))
        
        // ===== BURST_ENGINE MODULE (14 endpoints) =====
        .route("/burst_engine/simulation_timestep",
            get(burst_engine::get_simulation_timestep).post(burst_engine::post_simulation_timestep))
        .route("/burst_engine/fcl", get(burst_engine::get_fcl))
        .route("/burst_engine/fire_queue", get(burst_engine::get_fire_queue))
        .route("/burst_engine/fcl_reset", axum::routing::post(burst_engine::post_fcl_reset))
        .route("/burst_engine/fcl_status", get(burst_engine::get_fcl_status))
        .route("/burst_engine/fcl_sampler/config",
            get(burst_engine::get_fcl_sampler_config).post(burst_engine::post_fcl_sampler_config))
        .route("/burst_engine/fcl_sampler/area/:area_id/sample_rate",
            get(burst_engine::get_area_fcl_sample_rate).post(burst_engine::post_area_fcl_sample_rate))
        .route("/burst_engine/fire_ledger/default_window_size",
            get(burst_engine::get_fire_ledger_default_window_size)
            .put(burst_engine::put_fire_ledger_default_window_size))
        .route("/burst_engine/fire_ledger/areas_window_config", 
            get(burst_engine::get_fire_ledger_areas_window_config))
        .route("/burst_engine/stats", get(burst_engine::get_stats))
        .route("/burst_engine/status", get(burst_engine::get_status))
        .route("/burst_engine/control", axum::routing::post(burst_engine::post_control))
        .route("/burst_engine/burst_counter", get(burst_engine::get_burst_counter))
        .route("/burst_engine/start", axum::routing::post(burst_engine::post_start))
        .route("/burst_engine/stop", axum::routing::post(burst_engine::post_stop))
        .route("/burst_engine/hold", axum::routing::post(burst_engine::post_hold))
        .route("/burst_engine/resume", axum::routing::post(burst_engine::post_resume))
        .route("/burst_engine/config", get(burst_engine::get_config).put(burst_engine::put_config))
        .route("/burst_engine/fire_ledger/area/:area_id/window_size",
            get(burst_engine::get_fire_ledger_area_window_size).put(burst_engine::put_fire_ledger_area_window_size))
        .route("/burst_engine/fire_ledger/area/:area_id/history", get(burst_engine::get_fire_ledger_history))
        .route("/burst_engine/membrane_potentials",
            get(burst_engine::get_membrane_potentials).put(burst_engine::put_membrane_potentials))
        .route("/burst_engine/frequency_status", get(burst_engine::get_frequency_status))
        .route("/burst_engine/measure_frequency", axum::routing::post(burst_engine::post_measure_frequency))
        .route("/burst_engine/frequency_history", get(burst_engine::get_frequency_history))
        .route("/burst_engine/force_connectome_integration", axum::routing::post(burst_engine::post_force_connectome_integration))
        
        // ===== GENOME MODULE (22 endpoints) =====
        .route("/genome/file_name", get(genome::get_file_name))
        .route("/genome/circuits", get(genome::get_circuits))
        .route("/genome/amalgamation_destination", axum::routing::post(genome::post_amalgamation_destination))
        .route("/genome/amalgamation_cancellation", axum::routing::delete(genome::delete_amalgamation_cancellation))
        .route("/feagi/genome/append", axum::routing::post(genome::post_genome_append))
        .route("/genome/upload/barebones", axum::routing::post(genome::post_upload_barebones_genome))
        .route("/genome/upload/essential", axum::routing::post(genome::post_upload_essential_genome))
        .route("/genome/name", get(genome::get_name))
        .route("/genome/timestamp", get(genome::get_timestamp))
        .route("/genome/save", axum::routing::post(genome::post_save))
        .route("/genome/load", axum::routing::post(genome::post_load))
        .route("/genome/upload", axum::routing::post(genome::post_upload))
        .route("/genome/download", get(genome::get_download))
        .route("/genome/properties", get(genome::get_properties))
        .route("/genome/validate", axum::routing::post(genome::post_validate))
        .route("/genome/transform", axum::routing::post(genome::post_transform))
        .route("/genome/clone", axum::routing::post(genome::post_clone))
        .route("/genome/reset", axum::routing::post(genome::post_reset))
        .route("/genome/metadata", get(genome::get_metadata))
        .route("/genome/merge", axum::routing::post(genome::post_merge))
        .route("/genome/diff", get(genome::get_diff))
        .route("/genome/export_format", axum::routing::post(genome::post_export_format))
        .route("/genome/amalgamation", get(genome::get_amalgamation))
        .route("/genome/amalgamation_history", get(genome::get_amalgamation_history_exact))
        .route("/genome/cortical_template", get(genome::get_cortical_template))
        .route("/genome/defaults/files", get(genome::get_defaults_files))
        .route("/genome/download_region", get(genome::get_download_region))
        .route("/genome/genome_number", get(genome::get_genome_number))
        .route("/genome/amalgamation_by_filename", axum::routing::post(genome::post_amalgamation_by_filename))
        .route("/genome/amalgamation_by_payload", axum::routing::post(genome::post_amalgamation_by_payload))
        .route("/genome/amalgamation_by_upload", axum::routing::post(genome::post_amalgamation_by_upload))
        .route("/genome/append-file", axum::routing::post(genome::post_append_file))
        .route("/genome/upload/file", axum::routing::post(genome::post_upload_file))
        .route("/genome/upload/file/edit", axum::routing::post(genome::post_upload_file_edit))
        .route("/genome/upload/string", axum::routing::post(genome::post_upload_string))
        
        // ===== NEUROPLASTICITY MODULE (7 endpoints) =====
        .route("/neuroplasticity/plasticity_queue_depth",
            get(neuroplasticity::get_plasticity_queue_depth).put(neuroplasticity::put_plasticity_queue_depth))
        .route("/neuroplasticity/status", get(neuroplasticity::get_status))
        .route("/neuroplasticity/transforming", get(neuroplasticity::get_transforming))
        .route("/neuroplasticity/configure", axum::routing::post(neuroplasticity::post_configure))
        .route("/neuroplasticity/enable/:area_id", axum::routing::post(neuroplasticity::post_enable_area))
        .route("/neuroplasticity/disable/:area_id", axum::routing::post(neuroplasticity::post_disable_area))
        
        // ===== INSIGHT MODULE (6 endpoints) =====
        .route("/insight/neurons/membrane_potential_status", axum::routing::post(insight::post_neurons_membrane_potential_status))
        .route("/insight/neuron/synaptic_potential_status", axum::routing::post(insight::post_neuron_synaptic_potential_status))
        .route("/insight/neurons/membrane_potential_set", axum::routing::post(insight::post_neurons_membrane_potential_set))
        .route("/insight/neuron/synaptic_potential_set", axum::routing::post(insight::post_neuron_synaptic_potential_set))
        .route("/insight/analytics", get(insight::get_analytics))
        .route("/insight/data", get(insight::get_data))
        
        // ===== INPUT MODULE (4 endpoints) =====
        .route("/input/vision",
            get(input::get_vision).post(input::post_vision))
        .route("/input/sources", get(input::get_sources))
        .route("/input/configure", axum::routing::post(input::post_configure))
        
        // ===== OUTPUTS MODULE (2 endpoints) - Python uses /v1/output (singular)
        .route("/output/targets", get(outputs::get_targets))
        .route("/output/configure", axum::routing::post(outputs::post_configure))
        
        // ===== PHYSIOLOGY MODULE (2 endpoints) =====
        .route("/physiology/",
            get(physiology::get_physiology).put(physiology::put_physiology))
        
        // ===== SIMULATION MODULE (6 endpoints) =====
        .route("/simulation/upload/string", axum::routing::post(simulation::post_stimulation_upload))
        .route("/simulation/reset", axum::routing::post(simulation::post_reset))
        .route("/simulation/status", get(simulation::get_status))
        .route("/simulation/stats", get(simulation::get_stats))
        .route("/simulation/config", axum::routing::post(simulation::post_config))
        .route("/simulation/configure", axum::routing::post(simulation::post_configure))
        
        // ===== TRAINING MODULE (25 endpoints) =====
        .route("/training/shock", axum::routing::post(training::post_shock))
        .route("/training/shock/options", get(training::get_shock_options))
        .route("/training/shock/status", get(training::get_shock_status))
        .route("/training/shock/activate", axum::routing::post(training::post_shock_activate))
        .route("/training/reward/intensity", axum::routing::post(training::post_reward_intensity))
        .route("/training/reward", axum::routing::post(training::post_reward))
        .route("/training/punishment/intensity", axum::routing::post(training::post_punishment_intensity))
        .route("/training/punishment", axum::routing::post(training::post_punishment))
        .route("/training/gameover", axum::routing::post(training::post_gameover))
        .route("/training/brain_fitness", get(training::get_brain_fitness))
        .route("/training/fitness_criteria",
            get(training::get_fitness_criteria).put(training::put_fitness_criteria).post(training::post_fitness_criteria))
        .route("/training/fitness_stats",
            get(training::get_fitness_stats).put(training::put_fitness_stats).delete(training::delete_fitness_stats))
        .route("/training/reset_fitness_stats", axum::routing::delete(training::delete_reset_fitness_stats))
        .route("/training/training_report", get(training::get_training_report))
        .route("/training/status", get(training::get_status))
        .route("/training/stats", get(training::get_stats))
        .route("/training/config", axum::routing::post(training::post_config))
        .route("/training/configure", axum::routing::post(training::post_configure))
        
        // ===== VISUALIZATION MODULE (4 endpoints) =====
        .route("/visualization/register_client", axum::routing::post(visualization::post_register_client))
        .route("/visualization/unregister_client", axum::routing::post(visualization::post_unregister_client))
        .route("/visualization/heartbeat", axum::routing::post(visualization::post_heartbeat))
        .route("/visualization/status", get(visualization::get_status))
        
        // ===== MONITORING MODULE (4 endpoints) =====
        .route("/monitoring/status", get(monitoring::get_status))
        .route("/monitoring/metrics", get(monitoring::get_metrics))
        .route("/monitoring/data", get(monitoring::get_data))
        .route("/monitoring/performance", get(monitoring::get_performance))
        
        // ===== EVOLUTION MODULE (3 endpoints) =====
        .route("/evolution/status", get(evolution::get_status))
        .route("/evolution/config", axum::routing::post(evolution::post_config))
        .route("/evolution/configure", axum::routing::post(evolution::post_configure))
        
        // ===== SNAPSHOT MODULE (12 endpoints) =====
        // TODO: Implement snapshot endpoints
        // .route("/snapshot/create", axum::routing::post(snapshot::post_create))
        // .route("/snapshot/restore", axum::routing::post(snapshot::post_restore))
        // .route("/snapshot/", get(snapshot::get_list))
        // .route("/snapshot/:snapshot_id", axum::routing::delete(snapshot::delete_snapshot))
        // .route("/snapshot/:snapshot_id/artifact/:fmt", get(snapshot::get_artifact))
        // .route("/snapshot/compare", axum::routing::post(snapshot::post_compare))
        // .route("/snapshot/upload", axum::routing::post(snapshot::post_upload))
        // // Python uses /v1/snapshots/* (note the S)
        // .route("/snapshots/connectome", axum::routing::post(snapshot::post_snapshots_connectome))
        // .route("/snapshots/connectome/:snapshot_id/restore", axum::routing::post(snapshot::post_snapshots_connectome_restore))
        // .route("/snapshots/:snapshot_id/restore", axum::routing::post(snapshot::post_snapshots_restore))
        // .route("/snapshots/:snapshot_id", axum::routing::delete(snapshot::delete_snapshots_by_id))
        // .route("/snapshots/:snapshot_id/artifact/:fmt", get(snapshot::get_snapshots_artifact))
        
        // ===== NETWORK MODULE (3 endpoints) =====
        .route("/network/status", get(network::get_status))
        .route("/network/config", axum::routing::post(network::post_config))
        .route("/network/configure", axum::routing::post(network::post_configure))
}

/// OpenAPI spec handler
#[allow(dead_code)]  // In development - will be wired to OpenAPI route
async fn openapi_spec() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

// ============================================================================
// CORS CONFIGURATION
// ============================================================================

/// Create CORS layer for the API
/// 
/// TODO: Configure for production:
/// - Restrict allowed origins
/// - Allowed methods restricted
/// - Credentials support as needed
fn create_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

/// Middleware to log request and response bodies for debugging
async fn log_request_response_bodies(
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let (parts, body) = request.into_parts();
    
    // Only log bodies for POST/PUT/PATCH requests
    let should_log_request = matches!(
        parts.method.as_str(),
        "POST" | "PUT" | "PATCH"
    );
    
    let body_bytes = if should_log_request {
        // Collect body bytes
        match body.collect().await {
            Ok(collected) => {
                let bytes = collected.to_bytes();
                // Log request body if it's JSON
                if let Ok(body_str) = String::from_utf8(bytes.to_vec()) {
                    if !body_str.is_empty() {
                        tracing::debug!(target: "feagi-api", "📥 Request body: {}", body_str);
                    }
                }
                bytes
            }
            Err(_) => {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else {
        axum::body::Bytes::new()
    };
    
    // Reconstruct request with original body
    let request = Request::from_parts(parts, Body::from(body_bytes));
    
    // Call the next handler
    let response = next.run(request).await;
    
    // Log response body
    let (parts, body) = response.into_parts();
    
    match body.collect().await {
        Ok(collected) => {
            let bytes = collected.to_bytes();
            // Log response body if it's JSON and not too large
            if bytes.len() < 10000 {  // Only log responses < 10KB
                if let Ok(body_str) = String::from_utf8(bytes.to_vec()) {
                    if !body_str.is_empty() && body_str.starts_with('{') {
                        tracing::debug!(target: "feagi-api", "📤 Response body: {}", body_str);
                    }
                }
            }
            // Reconstruct response
            Ok(Response::from_parts(parts, Body::from(bytes)))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// ============================================================================
// HELPER HANDLERS
// ============================================================================

/// Root redirect handler - redirects to Swagger UI
async fn root_redirect() -> Redirect {
    Redirect::permanent("/swagger-ui/")
}

// Custom Swagger UI with FEAGI branding and dark/light themes
// Embedded from templates/custom-swagger-ui.html at compile time
async fn custom_swagger_ui() -> Html<&'static str> {
    const CUSTOM_SWAGGER_HTML: &str = include_str!("../../../templates/custom-swagger-ui.html");
    Html(CUSTOM_SWAGGER_HTML)
}

// ============================================================================
// PLACEHOLDER HANDLERS (for endpoints not yet implemented)
// ============================================================================

/// Placeholder handler for unimplemented endpoints
/// Returns 501 Not Implemented with a clear message
#[allow(dead_code)]  // In development - will be used for placeholder routes
async fn placeholder_handler(
    State(_state): State<ApiState>,
) -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "error": "Not yet implemented",
            "message": "This endpoint is registered but not yet implemented in Rust. See Python implementation."
        }))
    ).into_response()
}

/// Placeholder health check - returns basic response
#[allow(dead_code)]  // In development - will be used for basic health route
async fn placeholder_health_check(
    State(_state): State<ApiState>,
) -> Response {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "message": "Health check placeholder - Python-compatible path structure confirmed",
            "burst_engine": false,
            "brain_readiness": false
        }))
    ).into_response()
}
