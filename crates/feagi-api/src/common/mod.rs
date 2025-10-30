// Common types used across all transports

pub mod error;
pub mod request;
pub mod response;

pub use error::{ApiError, ApiErrorCode};
pub use request::ApiRequest;
pub use response::{ApiResponse, EmptyResponse};

/// Result type for API operations
pub type ApiResult<T> = Result<T, ApiError>;

