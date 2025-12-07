// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Transport-agnostic types for the service layer.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

pub mod dtos;
pub mod registration;
pub mod errors;
pub mod agent_registry;

// Re-export for convenience
pub use dtos::*;
pub use errors::{ServiceError, ServiceResult};





