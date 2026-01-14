// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Core agent protocol implementation
//!
//! This module provides low-level FEAGI agent protocol functionality:
//! - Agent registration with retry logic
//! - Background heartbeat service
//! - ZMQ transport management
//! - Sensory data transmission
//! - Motor data reception
//!
//! This is the foundational layer used by the SDK and custom agent implementations.

pub mod client;
pub mod config;
pub mod error;
pub mod heartbeat;
pub mod reconnect;
pub mod transport;

// Re-export core types
pub use client::AgentClient;
pub use config::AgentConfig;
pub use error::{Result, SdkError};
pub use transport::{RegistrationResponse, TransportConfig as TransportInfo};

// Re-export types from feagi-io
pub use feagi_io::{
    AgentCapabilities, AgentType, MotorCapability, SensoryCapability, VisionCapability,
};
