// HTTP server implementation (Axum)
//
// This module sets up the HTTP API server with Axum, including routing,
// middleware, and state management.

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json, Redirect, Response},
    routing::{delete, get, post, put},
    Router,
};
use tower_http::services::ServeDir;
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    common::{ApiError, ApiResponse, EmptyResponse},
    endpoints,
    openapi::ApiDoc,
    security::AuthContext,
    v1::dtos::{HealthCheckResponseV1, ReadinessCheckResponseV1},
};
use feagi_services::{AnalyticsService, ConnectomeService, GenomeService, NeuronService, RuntimeService};
use feagi_services::traits::AgentService;

/// Application state shared across all HTTP handlers
#[derive(Clone)]
pub struct ApiState {
    pub agent_service: Option<Arc<dyn AgentService + Send + Sync>>,
    pub analytics_service: Arc<dyn AnalyticsService + Send + Sync>,
    pub connectome_service: Arc<dyn ConnectomeService + Send + Sync>,
    pub genome_service: Arc<dyn GenomeService + Send + Sync>,
    pub neuron_service: Arc<dyn NeuronService + Send + Sync>,
    pub runtime_service: Arc<dyn RuntimeService + Send + Sync>,
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
    use crate::endpoints::agent::*;
    use crate::endpoints::system;
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
    
    Router::new()
        // ===== AGENT MODULE (7 endpoints) =====
        .route("/agent/register", axum::routing::post(register_agent))
        .route("/agent/heartbeat", axum::routing::post(heartbeat))
        .route("/agent/list", get(list_agents))
        .route("/agent/properties", get(get_agent_properties))
        .route("/agent/shared_mem", get(get_shared_memory))
        .route("/agent/manual_stimulation", axum::routing::post(manual_stimulation))
        .route("/agent/deregister", axum::routing::delete(deregister_agent))
        
        // ===== SYSTEM MODULE (5 endpoints) =====
        .route("/system/health_check", get(system::get_health_check))
        .route("/system/cortical_area_visualization_skip_rate", 
            get(system::get_cortical_area_visualization_skip_rate)
            .put(system::set_cortical_area_visualization_skip_rate))
        .route("/system/cortical_area_visualization_suppression_threshold",
            get(system::get_cortical_area_visualization_suppression_threshold)
            .put(system::set_cortical_area_visualization_suppression_threshold))
        
        // ===== CORTICAL_AREA MODULE (23 endpoints) =====
        .route("/cortical_area/ipu", get(cortical_area::get_ipu))
        .route("/cortical_area/opu", get(cortical_area::get_opu))
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
        .route("/cortical_area/clone", axum::routing::post(cortical_area::post_clone))
        .route("/cortical_area/multi/cortical_area",
            put(cortical_area::put_multi_cortical_area).delete(cortical_area::delete_multi_cortical_area))
        .route("/cortical_area/coord_2d", put(cortical_area::put_coord_2d))
        .route("/cortical_area/suppress_cortical_visibility", put(cortical_area::put_suppress_cortical_visibility))
        .route("/cortical_area/reset", put(cortical_area::put_reset))
        
        // ===== MORPHOLOGY MODULE (9 endpoints) =====
        .route("/morphology/morphology_list", get(morphology::get_morphology_list))
        .route("/morphology/morphology_types", get(morphology::get_morphology_types))
        .route("/morphology/list/types", get(morphology::get_list_types))
        .route("/morphology/morphologies", get(morphology::get_morphologies))
        .route("/morphology/morphology",
            axum::routing::post(morphology::post_morphology)
            .put(morphology::put_morphology)
            .delete(morphology::delete_morphology))
        .route("/morphology/morphology_properties", axum::routing::post(morphology::post_morphology_properties))
        .route("/morphology/morphology_usage", axum::routing::post(morphology::post_morphology_usage))
        
        // ===== REGION MODULE (7 endpoints) =====
        .route("/region/regions_members", get(region::get_regions_members))
        .route("/region/region",
            axum::routing::post(region::post_region)
            .put(region::put_region)
            .delete(region::delete_region))
        .route("/region/clone", axum::routing::post(region::post_clone))
        .route("/region/relocate_members", put(region::put_relocate_members))
        .route("/region/region_and_members", axum::routing::delete(region::delete_region_and_members))
        
        // ===== CORTICAL_MAPPING MODULE (4 endpoints) =====
        .route("/cortical_mapping/afferents", axum::routing::post(cortical_mapping::post_afferents))
        .route("/cortical_mapping/efferents", axum::routing::post(cortical_mapping::post_efferents))
        .route("/cortical_mapping/mapping_properties",
            axum::routing::post(cortical_mapping::post_mapping_properties).put(cortical_mapping::put_mapping_properties))
        
        // ===== CONNECTOME MODULE (3 endpoints) =====
        .route("/connectome/cortical_areas/list/detailed", get(connectome::get_cortical_areas_list_detailed))
        .route("/connectome/properties/dimensions", get(connectome::get_properties_dimensions))
        .route("/connectome/properties/mappings", get(connectome::get_properties_mappings))
        
        // ===== BURST_ENGINE MODULE (2 endpoints) =====
        .route("/burst_engine/simulation_timestep",
            get(burst_engine::get_simulation_timestep).post(burst_engine::post_simulation_timestep))
        
        // ===== GENOME MODULE (7 endpoints) =====
        .route("/genome/file_name", get(genome::get_file_name))
        .route("/genome/circuits", get(genome::get_circuits))
        .route("/genome/amalgamation_destination", axum::routing::post(genome::post_amalgamation_destination))
        .route("/genome/amalgamation_cancellation", axum::routing::delete(genome::delete_amalgamation_cancellation))
        .route("/feagi/genome/append", axum::routing::post(genome::post_genome_append))
        .route("/genome/upload/barebones", axum::routing::post(genome::post_upload_barebones_genome))
        .route("/genome/upload/essential", axum::routing::post(genome::post_upload_essential_genome))
        
        // ===== NEUROPLASTICITY MODULE (2 endpoints) =====
        .route("/neuroplasticity/plasticity_queue_depth",
            get(neuroplasticity::get_plasticity_queue_depth).put(neuroplasticity::put_plasticity_queue_depth))
        
        // ===== INSIGHT MODULE (4 endpoints) =====
        .route("/insight/neurons/membrane_potential_status", axum::routing::post(insight::post_neurons_membrane_potential_status))
        .route("/insight/neuron/synaptic_potential_status", axum::routing::post(insight::post_neuron_synaptic_potential_status))
        .route("/insight/neurons/membrane_potential_set", axum::routing::post(insight::post_neurons_membrane_potential_set))
        .route("/insight/neuron/synaptic_potential_set", axum::routing::post(insight::post_neuron_synaptic_potential_set))
        
        // ===== INPUT MODULE (2 endpoints) =====
        .route("/input/vision",
            get(input::get_vision).post(input::post_vision))
}

/// OpenAPI spec handler
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
