// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// API error types and conversions

#[cfg(feature = "http")]
use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use feagi_services::ServiceError;

/// HTTP status codes for API errors
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ApiErrorCode {
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    Conflict = 409,
    UnprocessableEntity = 422,
    Internal = 500,
    NotImplemented = 501,
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiError {
    /// HTTP status code
    pub code: u16,

    /// Error message
    pub message: String,

    /// Optional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ApiError {
    /// Create a new API error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            code: ApiErrorCode::Internal as u16,
            message: message.into(),
            details: None,
        }
    }

    /// Set error code
    pub fn with_code(mut self, code: ApiErrorCode) -> Self {
        self.code = code as u16;
        self
    }

    /// Set error details
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Create a "not found" error
    pub fn not_found(resource: &str, id: &str) -> Self {
        Self::new(format!("{} '{}' not found", resource, id)).with_code(ApiErrorCode::NotFound)
    }

    /// Create an "invalid input" error
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(message).with_code(ApiErrorCode::BadRequest)
    }

    /// Create a "conflict" error
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(message).with_code(ApiErrorCode::Conflict)
    }

    /// Create an "internal error"
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(message).with_code(ApiErrorCode::Internal)
    }

    /// Create a "forbidden" error
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(message).with_code(ApiErrorCode::Forbidden)
    }

    /// Create a "not implemented" error
    pub fn not_implemented(message: impl Into<String>) -> Self {
        Self::new(message).with_code(ApiErrorCode::NotImplemented)
    }
}

/// Convert from service layer errors
impl From<ServiceError> for ApiError {
    fn from(error: ServiceError) -> Self {
        match error {
            ServiceError::NotFound { resource, id } => {
                ApiError::new(format!("{} '{}' not found", resource, id))
                    .with_code(ApiErrorCode::NotFound)
            }
            ServiceError::InvalidInput(msg) => {
                ApiError::new(msg).with_code(ApiErrorCode::BadRequest)
            }
            ServiceError::AlreadyExists { resource, id } => {
                ApiError::new(format!("{} '{}' already exists", resource, id))
                    .with_code(ApiErrorCode::Conflict)
            }
            ServiceError::Internal(msg) => ApiError::new(msg).with_code(ApiErrorCode::Internal),
            ServiceError::Forbidden(msg) => ApiError::new(msg).with_code(ApiErrorCode::Forbidden),
            ServiceError::Backend(msg) => ApiError::new(msg).with_code(ApiErrorCode::Internal),
            ServiceError::StateError(msg) => ApiError::new(msg).with_code(ApiErrorCode::Internal),
            ServiceError::InvalidState(msg) => ApiError::new(msg).with_code(ApiErrorCode::Conflict),
            ServiceError::NotImplemented(msg) => {
                ApiError::new(msg).with_code(ApiErrorCode::NotImplemented)
            }
        }
    }
}

/// Implement Axum's IntoResponse for ApiError (only when http feature is enabled)
#[cfg(feature = "http")]
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code =
            StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        (status_code, Json(self)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_creation() {
        let error = ApiError::not_found("User", "123");
        assert_eq!(error.code, 404);
        assert!(error.message.contains("User"));
        assert!(error.message.contains("123"));
    }

    #[test]
    fn test_service_error_conversion() {
        let service_error = ServiceError::NotFound {
            resource: "Cortical Area".to_string(),
            id: "v1".to_string(),
        };

        let api_error: ApiError = service_error.into();
        assert_eq!(api_error.code, 404);
        assert!(api_error.message.contains("Cortical Area"));
    }
}
