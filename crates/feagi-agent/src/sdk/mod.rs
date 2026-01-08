// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! FEAGI Agent SDK
//!
//! High-level tools for building FEAGI controllers.

pub mod base;
#[cfg(feature = "sdk-io")]
pub mod connector;
pub mod error;
pub mod motor;
pub mod sensory;
pub mod registration;
#[cfg(feature = "sdk-io")]
pub mod types;
pub mod util;

// Re-export commonly used types
pub use base::{Controller, CorticalTopology, TopologyCache};
#[cfg(feature = "sdk-io")]
pub use connector::ConnectorAgent;
pub use error::{Result, SdkError};
pub use motor::MotorDecoder;
pub use sensory::SensoryEncoder;

