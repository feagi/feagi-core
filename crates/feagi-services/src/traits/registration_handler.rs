// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Trait for registration handling - breaks circular dependency

use crate::types::registration::{RegistrationRequest, RegistrationResponse};

/// Trait for handling agent registration
/// Implemented by feagi-io::RegistrationHandler
pub trait RegistrationHandlerTrait: Send + Sync {
    /// Process a registration request
    fn process_registration(
        &self,
        request: RegistrationRequest,
    ) -> Result<RegistrationResponse, String>;
}
