// HTTP server implementation (Axum)
//
// This module sets up the HTTP API server with Axum, including routing,
// middleware, and state management.

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json, Redirect, Response},
    routing::get,
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

/// Application state shared across all HTTP handlers
#[derive(Clone)]
pub struct ApiState {
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
        
        // Version-agnostic health endpoints (for backward compatibility)
        .route("/health", get(health_check_handler))
        .route("/ready", get(readiness_check_handler))
        
        // V1 API routes
        .nest("/api/v1", create_v1_router())
        
        // Backward compatibility: /api without version
        .nest("/api", create_v1_router())
        
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

/// Create V1 API router
fn create_v1_router() -> Router<ApiState> {
    Router::new()
        .route("/health", get(health_check_handler))
        .route("/ready", get(readiness_check_handler))
        
        // Cortical areas
        .route("/cortical-areas", get(list_cortical_areas_handler).post(create_cortical_area_handler))
        .route("/cortical-areas/:id", 
            get(get_cortical_area_handler)
            .put(update_cortical_area_handler)
            .delete(delete_cortical_area_handler))
        
        // Brain regions
        .route("/brain-regions", get(list_brain_regions_handler).post(create_brain_region_handler))
        .route("/brain-regions/:id", 
            get(get_brain_region_handler)
            .delete(delete_brain_region_handler))
        
        // Genome operations
        .route("/genome", get(get_genome_info_handler))
        .route("/genome/load", axum::routing::post(load_genome_handler))
        .route("/genome/save", axum::routing::post(save_genome_handler))
        .route("/genome/validate", axum::routing::post(validate_genome_handler))
        .route("/genome/reset", axum::routing::post(reset_connectome_handler))
        
        // Neuron operations
        .route("/neurons", get(list_neurons_handler).post(create_neuron_handler))
        .route("/neurons/count", get(get_neuron_count_handler))
        .route("/neurons/:id", get(get_neuron_handler).delete(delete_neuron_handler))
        
        // Runtime control
        .route("/runtime/status", get(get_runtime_status_handler))
        .route("/runtime/start", axum::routing::post(start_runtime_handler))
        .route("/runtime/stop", axum::routing::post(stop_runtime_handler))
        .route("/runtime/pause", axum::routing::post(pause_runtime_handler))
        .route("/runtime/resume", axum::routing::post(resume_runtime_handler))
        .route("/runtime/step", axum::routing::post(step_runtime_handler))
        .route("/runtime/frequency", axum::routing::post(set_frequency_handler))
        .route("/runtime/burst-count", get(get_burst_count_handler))
        .route("/runtime/reset-count", axum::routing::post(reset_burst_count_handler))
        
        // Analytics & Statistics
        .route("/analytics/health", get(get_system_health_handler))
        .route("/analytics/areas/stats", get(get_all_cortical_area_stats_handler))
        .route("/analytics/areas/populated", get(get_populated_areas_handler))
        .route("/analytics/areas/:id/stats", get(get_cortical_area_stats_handler))
        .route("/analytics/areas/:id/density", get(get_neuron_density_handler))
        .route("/analytics/connectivity/:source/:target", get(get_connectivity_stats_handler))
        .route("/analytics/connectome/stats", get(get_connectome_stats_handler))
}

/// OpenAPI spec handler
async fn openapi_spec() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

/// Health check handler (HTTP adapter)
async fn health_check_handler(
    State(state): State<ApiState>,
) -> Response {
    // Create anonymous auth context (stub - security not yet implemented)
    let auth_ctx = AuthContext::anonymous();
    
    // Call transport-agnostic endpoint
    match endpoints::health::health_check(&auth_ctx, state.analytics_service).await {
        Ok(health_data) => {
            (StatusCode::OK, Json(ApiResponse::success(health_data))).into_response()
        }
        Err(error) => error.into_response(),
    }
}

/// Readiness check handler (HTTP adapter)
async fn readiness_check_handler(
    State(state): State<ApiState>,
) -> Response {
    // Create anonymous auth context (stub - security not yet implemented)
    let auth_ctx = AuthContext::anonymous();
    
    // Call transport-agnostic endpoint
    match endpoints::health::readiness_check(&auth_ctx, state.analytics_service).await {
        Ok(readiness_data) => {
            (StatusCode::OK, Json(ApiResponse::success(readiness_data))).into_response()
        }
        Err(error) => error.into_response(),
    }
}

// ============================================================================
// CORTICAL AREA HANDLERS
// ============================================================================

/// List cortical areas
async fn list_cortical_areas_handler(
    State(state): State<ApiState>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::cortical_areas::list_cortical_areas(&auth_ctx, state.connectome_service).await {
        Ok(areas) => (StatusCode::OK, Json(ApiResponse::success(areas))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Get cortical area by ID
async fn get_cortical_area_handler(
    State(state): State<ApiState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::cortical_areas::get_cortical_area(&auth_ctx, state.connectome_service, id).await {
        Ok(area) => (StatusCode::OK, Json(ApiResponse::success(area))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Create cortical area
async fn create_cortical_area_handler(
    State(state): State<ApiState>,
    Json(request): Json<crate::v1::CreateCorticalAreaRequest>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::cortical_areas::create_cortical_area(&auth_ctx, state.connectome_service, request).await {
        Ok(area) => (StatusCode::CREATED, Json(ApiResponse::success(area))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Update cortical area
async fn update_cortical_area_handler(
    State(state): State<ApiState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(request): Json<crate::v1::UpdateCorticalAreaRequest>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::cortical_areas::update_cortical_area(&auth_ctx, state.connectome_service, id, request).await {
        Ok(area) => (StatusCode::OK, Json(ApiResponse::success(area))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Delete cortical area
async fn delete_cortical_area_handler(
    State(state): State<ApiState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::cortical_areas::delete_cortical_area(&auth_ctx, state.connectome_service, id).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success(EmptyResponse::new("Cortical area deleted successfully")))).into_response(),
        Err(error) => error.into_response(),
    }
}

// ============================================================================
// BRAIN REGION HANDLERS
// ============================================================================

/// List brain regions
async fn list_brain_regions_handler(
    State(state): State<ApiState>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::brain_regions::list_brain_regions(&auth_ctx, state.connectome_service).await {
        Ok(regions) => (StatusCode::OK, Json(ApiResponse::success(regions))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Get brain region by ID
async fn get_brain_region_handler(
    State(state): State<ApiState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::brain_regions::get_brain_region(&auth_ctx, state.connectome_service, id).await {
        Ok(region) => (StatusCode::OK, Json(ApiResponse::success(region))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Create brain region
async fn create_brain_region_handler(
    State(state): State<ApiState>,
    Json(request): Json<crate::v1::CreateBrainRegionRequest>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::brain_regions::create_brain_region(&auth_ctx, state.connectome_service, request).await {
        Ok(region) => (StatusCode::CREATED, Json(ApiResponse::success(region))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Delete brain region
async fn delete_brain_region_handler(
    State(state): State<ApiState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::brain_regions::delete_brain_region(&auth_ctx, state.connectome_service, id).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success(EmptyResponse::new("Brain region deleted successfully")))).into_response(),
        Err(error) => error.into_response(),
    }
}

// ============================================================================
// GENOME HANDLERS
// ============================================================================

/// Get genome info
async fn get_genome_info_handler(
    State(state): State<ApiState>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::genome::get_genome_info(&auth_ctx, state.genome_service).await {
        Ok(info) => (StatusCode::OK, Json(ApiResponse::success(info))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Load genome
async fn load_genome_handler(
    State(state): State<ApiState>,
    Json(request): Json<crate::v1::LoadGenomeRequest>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::genome::load_genome(&auth_ctx, state.genome_service, request).await {
        Ok(info) => (StatusCode::OK, Json(ApiResponse::success(info))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Save genome
async fn save_genome_handler(
    State(state): State<ApiState>,
    Json(request): Json<crate::v1::SaveGenomeRequest>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::genome::save_genome(&auth_ctx, state.genome_service, request).await {
        Ok(response) => (StatusCode::OK, Json(ApiResponse::success(response))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Validate genome
async fn validate_genome_handler(
    State(state): State<ApiState>,
    Json(request): Json<crate::v1::ValidateGenomeRequest>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::genome::validate_genome(&auth_ctx, state.genome_service, request).await {
        Ok(response) => (StatusCode::OK, Json(ApiResponse::success(response))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Reset connectome
async fn reset_connectome_handler(
    State(state): State<ApiState>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::genome::reset_connectome(&auth_ctx, state.genome_service).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success(EmptyResponse::new("Connectome reset successfully")))).into_response(),
        Err(error) => error.into_response(),
    }
}

// ============================================================================
// NEURON HANDLERS
// ============================================================================

/// List neurons
async fn list_neurons_handler(
    State(state): State<ApiState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    let cortical_area = params.get("cortical_area").cloned().unwrap_or_default();
    let limit = params.get("limit").and_then(|s| s.parse().ok());
    
    if cortical_area.is_empty() {
        return ApiError::invalid_input("cortical_area query parameter is required").into_response();
    }
    
    match endpoints::neurons::list_neurons(&auth_ctx, state.neuron_service, cortical_area, limit).await {
        Ok(neurons) => (StatusCode::OK, Json(ApiResponse::success(neurons))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Get neuron by ID
async fn get_neuron_handler(
    State(state): State<ApiState>,
    axum::extract::Path(id): axum::extract::Path<u64>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::neurons::get_neuron(&auth_ctx, state.neuron_service, id).await {
        Ok(neuron) => (StatusCode::OK, Json(ApiResponse::success(neuron))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Create neuron
async fn create_neuron_handler(
    State(state): State<ApiState>,
    Json(request): Json<crate::v1::CreateNeuronRequest>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::neurons::create_neuron(&auth_ctx, state.neuron_service, request).await {
        Ok(neuron) => (StatusCode::CREATED, Json(ApiResponse::success(neuron))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Delete neuron
async fn delete_neuron_handler(
    State(state): State<ApiState>,
    axum::extract::Path(id): axum::extract::Path<u64>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::neurons::delete_neuron(&auth_ctx, state.neuron_service, id).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success(EmptyResponse::new("Neuron deleted successfully")))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Get neuron count
async fn get_neuron_count_handler(
    State(state): State<ApiState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    let cortical_area = params.get("cortical_area").cloned().unwrap_or_default();
    
    if cortical_area.is_empty() {
        return ApiError::invalid_input("cortical_area query parameter is required").into_response();
    }
    
    match endpoints::neurons::get_neuron_count(&auth_ctx, state.neuron_service, cortical_area).await {
        Ok(count) => (StatusCode::OK, Json(ApiResponse::success(count))).into_response(),
        Err(error) => error.into_response(),
    }
}

// ============================================================================
// RUNTIME HANDLERS
// ============================================================================

/// Get runtime status
async fn get_runtime_status_handler(
    State(state): State<ApiState>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::runtime::get_runtime_status(&auth_ctx, state.runtime_service).await {
        Ok(status) => (StatusCode::OK, Json(ApiResponse::success(status))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Start runtime
async fn start_runtime_handler(
    State(state): State<ApiState>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::runtime::start_runtime(&auth_ctx, state.runtime_service).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success(EmptyResponse::new("Burst engine started successfully")))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Stop runtime
async fn stop_runtime_handler(
    State(state): State<ApiState>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::runtime::stop_runtime(&auth_ctx, state.runtime_service).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success(EmptyResponse::new("Burst engine stopped successfully")))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Pause runtime
async fn pause_runtime_handler(
    State(state): State<ApiState>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::runtime::pause_runtime(&auth_ctx, state.runtime_service).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success(EmptyResponse::new("Burst engine paused successfully")))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Resume runtime
async fn resume_runtime_handler(
    State(state): State<ApiState>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::runtime::resume_runtime(&auth_ctx, state.runtime_service).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success(EmptyResponse::new("Burst engine resumed successfully")))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Step runtime
async fn step_runtime_handler(
    State(state): State<ApiState>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::runtime::step_runtime(&auth_ctx, state.runtime_service).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success(EmptyResponse::new("Burst step executed successfully")))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Set frequency
async fn set_frequency_handler(
    State(state): State<ApiState>,
    Json(request): Json<crate::v1::SetFrequencyRequest>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::runtime::set_frequency(&auth_ctx, state.runtime_service, request).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success(EmptyResponse::new("Frequency set successfully")))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Get burst count
async fn get_burst_count_handler(
    State(state): State<ApiState>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::runtime::get_burst_count(&auth_ctx, state.runtime_service).await {
        Ok(count) => (StatusCode::OK, Json(ApiResponse::success(count))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Reset burst count
async fn reset_burst_count_handler(
    State(state): State<ApiState>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::runtime::reset_burst_count(&auth_ctx, state.runtime_service).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success(EmptyResponse::new("Burst count reset successfully")))).into_response(),
        Err(error) => error.into_response(),
    }
}

// ============================================================================
// ANALYTICS HANDLERS
// ============================================================================

/// Get system health
async fn get_system_health_handler(
    State(state): State<ApiState>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::analytics::get_system_health(&auth_ctx, state.analytics_service).await {
        Ok(health) => (StatusCode::OK, Json(ApiResponse::success(health))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Get cortical area stats
async fn get_cortical_area_stats_handler(
    State(state): State<ApiState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::analytics::get_cortical_area_stats(&auth_ctx, state.analytics_service, id).await {
        Ok(stats) => (StatusCode::OK, Json(ApiResponse::success(stats))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Get all cortical area stats
async fn get_all_cortical_area_stats_handler(
    State(state): State<ApiState>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::analytics::get_all_cortical_area_stats(&auth_ctx, state.analytics_service).await {
        Ok(stats) => (StatusCode::OK, Json(ApiResponse::success(stats))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Get connectivity stats
async fn get_connectivity_stats_handler(
    State(state): State<ApiState>,
    axum::extract::Path((source, target)): axum::extract::Path<(String, String)>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::analytics::get_connectivity_stats(&auth_ctx, state.analytics_service, source, target).await {
        Ok(stats) => (StatusCode::OK, Json(ApiResponse::success(stats))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Get connectome stats
async fn get_connectome_stats_handler(
    State(state): State<ApiState>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::analytics::get_connectome_stats(&auth_ctx, state.analytics_service).await {
        Ok(stats) => (StatusCode::OK, Json(ApiResponse::success(stats))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Get populated areas
async fn get_populated_areas_handler(
    State(state): State<ApiState>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::analytics::get_populated_areas(&auth_ctx, state.analytics_service).await {
        Ok(areas) => (StatusCode::OK, Json(ApiResponse::success(areas))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Get neuron density
async fn get_neuron_density_handler(
    State(state): State<ApiState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Response {
    let auth_ctx = AuthContext::anonymous();
    
    match endpoints::analytics::get_neuron_density(&auth_ctx, state.analytics_service, id).await {
        Ok(density) => (StatusCode::OK, Json(ApiResponse::success(density))).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Create CORS layer (permissive for development)
///
/// TODO: In production, this should be configured based on environment:
/// - Allowed origins from config
/// - Allowed methods restricted
/// - Credentials support as needed
fn create_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

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
