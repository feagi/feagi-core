// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Transport-agnostic types for the service layer.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

pub mod agent_registry;
pub mod connectome_snapshot;
pub mod dtos;
pub mod errors;
pub mod memory_stats;
pub mod registration;

// Re-export for convenience
pub use connectome_snapshot::{ConnectomeMetadata, ConnectomeSnapshot, ConnectomeStatistics, SerializableNeuronArray, SerializableSynapseArray};
pub use dtos::*;
pub use errors::{ServiceError, ServiceResult};
pub use memory_stats::{MemoryAreaStats, MemoryStatsCache};
