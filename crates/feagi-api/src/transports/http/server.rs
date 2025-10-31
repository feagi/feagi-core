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
        
        // Add state
        .with_state(state)
        
        // Add middleware
        .layer(create_cors_layer())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(tower_http::trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_response(tower_http::trace::DefaultOnResponse::new().level(tracing::Level::INFO))
        )
}

/// Create V1 API router - Match Python structure EXACTLY
/// Format: /v1/{module}/{snake_case_endpoint}
fn create_v1_router() -> Router<ApiState> {
    use crate::endpoints::agent::*;
    use crate::endpoints::system;
    
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
        .route("/cortical_area/ipu", get(placeholder_handler))
        .route("/cortical_area/opu", get(placeholder_handler))
        .route("/cortical_area/cortical_area_id_list", get(placeholder_handler))
        .route("/cortical_area/cortical_area_name_list", get(placeholder_handler))
        .route("/cortical_area/cortical_id_name_mapping", get(placeholder_handler))
        .route("/cortical_area/cortical_types", get(placeholder_handler))
        .route("/cortical_area/cortical_map_detailed", get(placeholder_handler))
        .route("/cortical_area/cortical_locations_2d", get(placeholder_handler))
        .route("/cortical_area/cortical_area/geometry", get(placeholder_handler))
        .route("/cortical_area/cortical_visibility", get(placeholder_handler))
        .route("/cortical_area/cortical_name_location", axum::routing::post(placeholder_handler))
        .route("/cortical_area/cortical_area_properties", axum::routing::post(placeholder_handler))
        .route("/cortical_area/multi/cortical_area_properties", axum::routing::post(placeholder_handler))
        .route("/cortical_area/cortical_area",
            axum::routing::post(placeholder_handler)
            .put(placeholder_handler)
            .delete(placeholder_handler))
        .route("/cortical_area/custom_cortical_area", axum::routing::post(placeholder_handler))
        .route("/cortical_area/clone", axum::routing::post(placeholder_handler))
        .route("/cortical_area/multi/cortical_area",
            put(placeholder_handler).delete(placeholder_handler))
        .route("/cortical_area/coord_2d", put(placeholder_handler))
        .route("/cortical_area/suppress_cortical_visibility", put(placeholder_handler))
        .route("/cortical_area/reset", put(placeholder_handler))
        
        // ===== MORPHOLOGY MODULE (9 endpoints) =====
        .route("/morphology/morphology_list", get(placeholder_handler))
        .route("/morphology/morphology_types", get(placeholder_handler))
        .route("/morphology/list/types", get(placeholder_handler))
        .route("/morphology/morphologies", get(placeholder_handler))
        .route("/morphology/morphology",
            axum::routing::post(placeholder_handler)
            .put(placeholder_handler)
            .delete(placeholder_handler))
        .route("/morphology/morphology_properties", axum::routing::post(placeholder_handler))
        .route("/morphology/morphology_usage", axum::routing::post(placeholder_handler))
        
        // ===== REGION MODULE (7 endpoints) =====
        .route("/region/regions_members", get(placeholder_handler))
        .route("/region/region",
            axum::routing::post(placeholder_handler)
            .put(placeholder_handler)
            .delete(placeholder_handler))
        .route("/region/clone", axum::routing::post(placeholder_handler))
        .route("/region/relocate_members", put(placeholder_handler))
        .route("/region/region_and_members", axum::routing::delete(placeholder_handler))
        
        // ===== CORTICAL_MAPPING MODULE (4 endpoints) =====
        .route("/cortical_mapping/afferents", axum::routing::post(placeholder_handler))
        .route("/cortical_mapping/efferents", axum::routing::post(placeholder_handler))
        .route("/cortical_mapping/mapping_properties",
            axum::routing::post(placeholder_handler).put(placeholder_handler))
        
        // ===== CONNECTOME MODULE (3 endpoints) =====
        .route("/connectome/cortical_areas/list/detailed", get(placeholder_handler))
        .route("/connectome/properties/dimensions", get(placeholder_handler))
        .route("/connectome/properties/mappings", get(placeholder_handler))
        
        // ===== BURST_ENGINE MODULE (2 endpoints) =====
        .route("/burst_engine/simulation_timestep",
            get(placeholder_handler).post(placeholder_handler))
        
        // ===== GENOME MODULE (4 endpoints) =====
        .route("/genome/file_name", get(placeholder_handler))
        .route("/genome/circuits", get(placeholder_handler))
        .route("/genome/amalgamation_destination", axum::routing::post(placeholder_handler))
        .route("/genome/amalgamation_cancellation", axum::routing::delete(placeholder_handler))
        .route("/feagi/genome/append", axum::routing::post(placeholder_handler))
        
        // ===== NEUROPLASTICITY MODULE (2 endpoints) =====
        .route("/neuroplasticity/plasticity_queue_depth",
            get(placeholder_handler).put(placeholder_handler))
        
        // ===== INSIGHT MODULE (4 endpoints) =====
        .route("/insight/neurons/membrane_potential_status", axum::routing::post(placeholder_handler))
        .route("/insight/neuron/synaptic_potential_status", axum::routing::post(placeholder_handler))
        .route("/insight/neurons/membrane_potential_set", axum::routing::post(placeholder_handler))
        .route("/insight/neuron/synaptic_potential_set", axum::routing::post(placeholder_handler))
        
        // ===== INPUT MODULE (2 endpoints) =====
        .route("/input/vision",
            get(placeholder_handler).post(placeholder_handler))
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
