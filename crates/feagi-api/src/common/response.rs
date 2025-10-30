use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::ApiError;

/// Generic API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiResponse<T> {
    /// Whether the request was successful
    pub success: bool,
    
    /// Response data (present on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    
    /// Error details (present on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    
    /// Response timestamp (ISO 8601)
    pub timestamp: String,
}

impl<T> ApiResponse<T> {
    /// Create a successful response
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    /// Create an error response
    pub fn error(error: ApiError) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

/// Convert Result to ApiResponse
impl<T> From<Result<T, ApiError>> for ApiResponse<T> {
    fn from(result: Result<T, ApiError>) -> Self {
        match result {
            Ok(data) => ApiResponse::success(data),
            Err(error) => ApiResponse::error(error),
        }
    }
}

/// Empty response for operations that don't return data
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EmptyResponse {}

impl EmptyResponse {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for EmptyResponse {
    fn default() -> Self {
        Self::new()
    }
}

