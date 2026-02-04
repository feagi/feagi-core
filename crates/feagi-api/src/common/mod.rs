// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// Common types used across all transports

pub mod error;
pub mod agent_registration;
pub mod request;
pub mod response;
pub mod types;
pub use types::{ApiState, Json, Path, Query, State};

pub use error::{ApiError, ApiErrorCode};
pub use request::ApiRequest;
pub use response::{ApiResponse, EmptyResponse};

/// Result type for API operations
pub type ApiResult<T> = Result<T, ApiError>;
