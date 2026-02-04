// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Core types and utilities shared across all transports
//!
//! This module contains transport-agnostic components:
//! - SharedFBC (Arc<FeagiByteContainer>) type alias
//! - Error types
//! - Agent registry and management
//! - Registration handling
//! - Heartbeat monitoring
//! - I/O system configuration

pub mod agent_registry;
pub mod config;
pub mod events;
pub mod heartbeat;
pub mod registration;
pub mod type_validation;
pub mod types;

// Re-export commonly used types (all from feagi-services now)
pub use agent_registry::{
    AgentCapabilities, AgentInfo, AgentRegistry, AgentTransport, AgentType, MotorCapability,
    MotorUnit, MotorUnitSpec, SensoryCapability, SensoryUnit, VisionCapability,
    VisualizationCapability,
};
pub use config::{IOConfig, TransportMode, WebSocketConfig};
pub use events::{
    AgentDisconnectedEvent, AgentRegisteredEvent, MotorCommandEvent, SensoryDataEvent,
    VisualizationReadyEvent,
};
pub use heartbeat::HeartbeatTracker;
pub use registration::{
    RegistrationHandler, RegistrationRequest, RegistrationResponse, TransportConfig,
};
pub use type_validation::{
    get_recommended_buffer_size, should_use_compression, validate_motor_compatibility,
    validate_sensory_compatibility, ValidationResult,
};
pub use types::{IOError, Result, SharedFBC, StreamType};
