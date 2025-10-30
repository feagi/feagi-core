// HTTP server implementation (Axum)
//
// This module sets up the HTTP API server with Axum, including routing,
// middleware, and state management.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    common::{ApiError, ApiResponse},
    endpoints,
    openapi::ApiDoc,
    security::AuthContext,
    v1::dtos::{HealthCheckResponseV1, ReadinessCheckResponseV1},
};
use feagi_services::AnalyticsService;

/// Application state shared across all HTTP handlers
#[derive(Clone)]
pub struct ApiState {
    pub analytics_service: Arc<dyn AnalyticsService + Send + Sync>,
    // TODO: Add more services as they're implemented
    // pub cortical_area_service: Arc<dyn CorticalAreaService + Send + Sync>,
    // pub genome_service: Arc<dyn GenomeService + Send + Sync>,
}

/// Create the main HTTP server application
pub fn create_http_server(state: ApiState) -> Router {
    Router::new()
        // Swagger UI at /swagger-ui/
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/openapi.json", ApiDoc::openapi())
        )
        // OpenAPI spec endpoint
        .route("/openapi.json", get(openapi_spec))
        
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
        // TODO: Add more V1 endpoints
        // .route("/cortical-areas", get(list_cortical_areas).post(create_cortical_area))
        // .route("/cortical-areas/:id", get(get_cortical_area).put(update_cortical_area).delete(delete_cortical_area))
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
