// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Service implementations.

Default implementations of service traits using feagi-bdu, feagi-evo, and feagi-burst-engine.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

pub mod agent_service_impl;
pub mod analytics_service_impl;
pub mod connectome_service_impl;
pub mod genome_service_impl;
pub mod neuron_service_impl;
pub mod runtime_service_impl;
pub mod snapshot_service_impl;
pub mod system_service_impl;

// Re-export for convenience
pub use agent_service_impl::AgentServiceImpl;
pub use analytics_service_impl::AnalyticsServiceImpl;
pub use connectome_service_impl::ConnectomeServiceImpl;
pub use genome_service_impl::GenomeServiceImpl;
pub use neuron_service_impl::NeuronServiceImpl;
pub use runtime_service_impl::RuntimeServiceImpl;
pub use snapshot_service_impl::SnapshotServiceImpl;
pub use system_service_impl::SystemServiceImpl;
