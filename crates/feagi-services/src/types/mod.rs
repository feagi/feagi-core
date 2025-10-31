/*!
Transport-agnostic types for the service layer.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

pub mod dtos;
pub mod errors;

// Re-export for convenience
pub use dtos::*;
pub use errors::{ServiceError, ServiceResult};




