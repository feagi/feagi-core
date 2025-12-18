// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Service trait definitions.

These traits define the stable application boundary between
transport adapters and domain logic.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

pub mod agent_service;
pub mod analytics_service;
pub mod connectome_service;
pub mod genome_service;
pub mod neuron_service;
pub mod registration_handler;
pub mod runtime_service;
pub mod snapshot_service;
pub mod system_service;

// Re-export for convenience
pub use agent_service::AgentService;
pub use analytics_service::AnalyticsService;
pub use connectome_service::ConnectomeService;
pub use genome_service::GenomeService;
pub use neuron_service::NeuronService;
pub use runtime_service::RuntimeService;
pub use snapshot_service::*;
pub use system_service::SystemService;
