use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use utoipa::ToSchema;

/// API error codes
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiErrorCode {
    NotFound,
    InvalidInput,
    AlreadyExists,
    Internal,
    NotImplemented,
    ServiceUnavailable,
    Unauthorized,
    Forbidden,
}

/// API error type (compatible with Python FastAPI error format)
#[derive(Debug, Error, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiError {
    /// Error detail message (Python FastAPI compatibility: uses "detail" field)
    pub detail: String,
    
    /// Error code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<ApiErrorCode>,
    
    /// Additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.detail)
    }
}

impl ApiError {
    pub fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
            code: None,
            details: None,
        }
    }

    pub fn with_code(mut self, code: ApiErrorCode) -> Self {
        self.code = Some(code);
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn not_found(resource: impl Into<String>, id: impl Into<String>) -> Self {
        let resource = resource.into();
        let id = id.into();
        Self::new(format!("{} '{}' not found", resource, id))
            .with_code(ApiErrorCode::NotFound)
            .with_details(serde_json::json!({
                "resource": resource,
                "id": id
            }))
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(message).with_code(ApiErrorCode::InvalidInput)
    }

    pub fn already_exists(resource: impl Into<String>, id: impl Into<String>) -> Self {
        let resource = resource.into();
        let id = id.into();
        Self::new(format!("{} '{}' already exists", resource, id))
            .with_code(ApiErrorCode::AlreadyExists)
            .with_details(serde_json::json!({
                "resource": resource,
                "id": id
            }))
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(message).with_code(ApiErrorCode::Internal)
    }

    pub fn not_implemented(feature: impl Into<String>) -> Self {
        Self::new(format!("Feature not implemented: {}", feature.into()))
            .with_code(ApiErrorCode::NotImplemented)
    }

    pub fn service_unavailable(service: impl Into<String>) -> Self {
        Self::new(format!("Service unavailable: {}", service.into()))
            .with_code(ApiErrorCode::ServiceUnavailable)
    }
}

/// Convert service layer errors to API errors
impl From<feagi_services::ServiceError> for ApiError {
    fn from(err: feagi_services::ServiceError) -> Self {
        use feagi_services::ServiceError;
        
        match err {
            ServiceError::NotFound { resource, id } => {
                ApiError::new(format!("{} '{}' not found", resource, id))
                    .with_code(ApiErrorCode::NotFound)
                    .with_details(serde_json::json!({"resource": resource, "id": id}))
            },
            ServiceError::InvalidInput(msg) => ApiError::new(msg).with_code(ApiErrorCode::InvalidInput),
            ServiceError::AlreadyExists { resource, id } => {
                ApiError::new(format!("{} '{}' already exists", resource, id))
                    .with_code(ApiErrorCode::AlreadyExists)
                    .with_details(serde_json::json!({"resource": resource, "id": id}))
            },
            ServiceError::Internal(msg) => ApiError::new(msg).with_code(ApiErrorCode::Internal),
            ServiceError::Forbidden(msg) => ApiError::new(msg).with_code(ApiErrorCode::Forbidden),
            ServiceError::Backend(msg) => ApiError::new(msg).with_code(ApiErrorCode::Internal),
            ServiceError::StateError(msg) => ApiError::new(msg).with_code(ApiErrorCode::Internal),
        }
    }
}


