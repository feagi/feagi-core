// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// ZMQ server implementation (thin wrapper over feagi-pns infrastructure)
//
// This adapter translates ZMQ messages to endpoint calls, leveraging the
// existing api_control infrastructure in feagi-pns.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    common::{ApiError, ApiResponse},
    endpoints,
    security::AuthContext,
    v1::dtos::{HealthCheckResponseV1, ReadinessCheckResponseV1},
};
use feagi_services::AnalyticsService;

/// ZMQ request format (matches feagi-pns api_control format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZmqRequest {
    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    pub method: String,
    
    /// Request path (e.g., "/v1/system/health_check")
    pub path: String,
    
    /// Request body as JSON (optional)
    pub body: Option<serde_json::Value>,
}

/// ZMQ response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZmqResponse {
    /// HTTP-like status code
    pub status: u16,
    
    /// Response body as JSON
    pub body: serde_json::Value,
}

/// Application state for ZMQ server
#[derive(Clone)]
pub struct ZmqApiState {
    pub analytics_service: Arc<dyn AnalyticsService + Send + Sync>,
    // TODO: Add more services as needed
}

/// Route a ZMQ request to the appropriate endpoint
///
/// This is the main entry point for ZMQ-based API calls.
/// It routes requests to the same transport-agnostic endpoints used by HTTP.
pub async fn route_zmq_request(
    request: ZmqRequest,
    state: &ZmqApiState,
) -> ZmqResponse {
    // Create anonymous auth context (stub - same as HTTP)
    let auth_ctx = AuthContext::anonymous();
    
    // Route based on method and path
    match (request.method.as_str(), request.path.as_str()) {
        // Health check endpoints
        ("GET", "/health") | ("GET", "/v1/system/health_check") => {
            handle_health_check(&auth_ctx, state).await
        }
        
        ("GET", "/ready") | ("GET", "/v1/system/readiness_check") => {
            handle_readiness_check(&auth_ctx, state).await
        }
        
        // TODO: Add more endpoint routes
        // ("GET", path) if path.starts_with("/v1/cortical_area") => { ... }
        // ("POST", "/v1/cortical_area") => { ... }
        
        // Not found
        _ => {
            let error = ApiError::not_found("Endpoint", &request.path);
            ZmqResponse {
                status: 404,
                body: serde_json::to_value(error).unwrap_or(serde_json::json!({"detail": "Not found"})),
            }
        }
    }
}

/// Handle health check via ZMQ
async fn handle_health_check(
    auth_ctx: &AuthContext,
    state: &ZmqApiState,
) -> ZmqResponse {
    // Call the same transport-agnostic endpoint as HTTP
    match endpoints::health::health_check(auth_ctx, state.analytics_service.clone()).await {
        Ok(health_data) => {
            let response: ApiResponse<HealthCheckResponseV1> = ApiResponse::success(health_data);
            ZmqResponse {
                status: 200,
                body: serde_json::to_value(response).unwrap_or(serde_json::json!({"success": false})),
            }
        }
        Err(error) => {
            ZmqResponse {
                status: 500,
                body: serde_json::to_value(error).unwrap_or(serde_json::json!({"detail": "Internal error"})),
            }
        }
    }
}

/// Handle readiness check via ZMQ
async fn handle_readiness_check(
    auth_ctx: &AuthContext,
    state: &ZmqApiState,
) -> ZmqResponse {
    match endpoints::health::readiness_check(auth_ctx, state.analytics_service.clone()).await {
        Ok(readiness_data) => {
            let response: ApiResponse<ReadinessCheckResponseV1> = ApiResponse::success(readiness_data);
            ZmqResponse {
                status: 200,
                body: serde_json::to_value(response).unwrap_or(serde_json::json!({"success": false})),
            }
        }
        Err(error) => {
            ZmqResponse {
                status: 500,
                body: serde_json::to_value(error).unwrap_or(serde_json::json!({"detail": "Internal error"})),
            }
        }
    }
}

/// Integration point with feagi-pns api_control
///
/// This function can be called from feagi-pns::api_control when it receives
/// a REST-like request over ZMQ. It provides the business logic while
/// feagi-pns handles the transport.
///
/// Example usage in feagi-pns:
/// ```ignore
/// let response = feagi_api::transports::zmq::handle_api_control_request(
///     method, path, body, &api_state
/// ).await;
/// ```
pub async fn handle_api_control_request(
    method: String,
    path: String,
    body: Option<serde_json::Value>,
    state: &ZmqApiState,
) -> ZmqResponse {
    let request = ZmqRequest { method, path, body };
    route_zmq_request(request, state).await
}
