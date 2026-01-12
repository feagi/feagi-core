// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Controller trait and base implementations

use crate::sdk::error::Result;
use feagi_structures::genomic::cortical_area::CorticalID;

/// Core controller trait
///
/// Implement this trait to create custom controllers that follow FEAGI conventions.
///
/// # Example
/// ```ignore
/// use feagi_agent::sdk::base::Controller;
///
/// struct MyController {
///     // ... fields
/// }
///
/// #[async_trait::async_trait]
/// impl Controller for MyController {
///     fn controller_type(&self) -> &str { "my-custom" }
///     fn agent_id(&self) -> &str { &self.agent_id }
///     async fn start(&mut self) -> Result<()> { /* ... */ }
///     async fn stop(&mut self) -> Result<()> { /* ... */ }
///     fn is_running(&self) -> bool { /* ... */ }
///     fn cortical_ids(&self) -> &[CorticalID] { /* ... */ }
/// }
/// ```
#[async_trait::async_trait]
pub trait Controller: Send + Sync {
    /// Controller type identifier (e.g., "video", "text", "audio")
    fn controller_type(&self) -> &str;

    /// Get the agent ID this controller manages
    fn agent_id(&self) -> &str;

    /// Start the controller (connect to FEAGI, begin operation)
    async fn start(&mut self) -> Result<()>;

    /// Stop the controller gracefully
    async fn stop(&mut self) -> Result<()>;

    /// Check if controller is currently running
    fn is_running(&self) -> bool;

    /// Get cortical IDs this controller produces/consumes
    fn cortical_ids(&self) -> &[CorticalID];
}
