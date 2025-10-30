// HTTP server implementation (Axum)

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use std::sync::Arc;

use crate::{
    common::{ApiError, ApiResponse},
    endpoints,
    middleware,
    security::AuthContext,
    v1::dtos::{HealthCheckResponseV1, ReadinessCheckResponseV1},
};
use feagi_services::AnalyticsService;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct ApiState {
    pub analytics_service: Arc<dyn AnalyticsService + Send + Sync>,
    // TODO: Add more services as needed
    // pub connectome_service: Arc<dyn ConnectomeService + Send + Sync>,
    // pub genome_service: Arc<dyn GenomeService + Send + Sync>,
}

/// Create the main Axum application
pub fn create_app(state: ApiState) -> Router {
    // Create version-specific routers
    let v1_router = create_v1_router();
    
    Router::new()
        // Version-agnostic endpoints
        .route("/health", get(health_check_handler))
        .route("/ready", get(readiness_check_handler))
        
        // V1 API
        .nest("/api/v1", v1_router.clone())
        
        // Default /api/* routes to v1 (for backward compatibility)
        .nest("/api", v1_router)
        
        // Add middleware
        .layer(middleware::cors::create_cors_layer())
        .layer(tower_http::trace::TraceLayer::new_for_http())
        
        .with_state(state)
}

/// Create V1 router
fn create_v1_router() -> Router<ApiState> {
    Router::new()
        .route("/health", get(health_check_handler))
        .route("/ready", get(readiness_check_handler))
        // TODO: Add more V1 endpoints
        // .route("/cortical-areas", get(list_cortical_areas).post(create_cortical_area))
        // .route("/cortical-areas/:id", get(get_cortical_area).delete(delete_cortical_area))
}

/// HTTP handler for health check
async fn health_check_handler(
    State(state): State<ApiState>,
) -> Result<Json<ApiResponse<HealthCheckResponseV1>>, HttpError> {
    // Create anonymous auth context (stub)
    let auth_ctx = AuthContext::anonymous();
    
    // Call transport-agnostic endpoint
    let result = endpoints::health::health_check(&auth_ctx, state.analytics_service.clone()).await;
    
    // Convert to HTTP response
    Ok(Json(ApiResponse::from(result)))
}

/// HTTP handler for readiness check
async fn readiness_check_handler(
    State(state): State<ApiState>,
) -> Result<Json<ApiResponse<ReadinessCheckResponseV1>>, HttpError> {
    let auth_ctx = AuthContext::anonymous();
    
    let result = endpoints::health::readiness_check(&auth_ctx, state.analytics_service.clone()).await;
    
    Ok(Json(ApiResponse::from(result)))
}

/// HTTP error wrapper for Axum
pub struct HttpError(ApiError);

impl From<ApiError> for HttpError {
    fn from(err: ApiError) -> Self {
        HttpError(err)
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let status = match self.0.code {
            Some(crate::common::ApiErrorCode::NotFound) => StatusCode::NOT_FOUND,
            Some(crate::common::ApiErrorCode::InvalidInput) => StatusCode::BAD_REQUEST,
            Some(crate::common::ApiErrorCode::AlreadyExists) => StatusCode::CONFLICT,
            Some(crate::common::ApiErrorCode::Unauthorized) => StatusCode::UNAUTHORIZED,
            Some(crate::common::ApiErrorCode::Forbidden) => StatusCode::FORBIDDEN,
            Some(crate::common::ApiErrorCode::NotImplemented) => StatusCode::NOT_IMPLEMENTED,
            Some(crate::common::ApiErrorCode::ServiceUnavailable) => StatusCode::SERVICE_UNAVAILABLE,
            Some(crate::common::ApiErrorCode::Internal) | None => StatusCode::INTERNAL_SERVER_ERROR,
        };
        
        (status, Json(self.0)).into_response()
    }
}
