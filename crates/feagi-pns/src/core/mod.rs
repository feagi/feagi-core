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
//! - PNS configuration

pub mod agent_registry;
pub mod config;
pub mod events;
pub mod heartbeat;
pub mod registration;
pub mod types;
pub mod type_validation;

// Re-export commonly used types (all from feagi-services now)
pub use agent_registry::{
    AgentRegistry, AgentCapabilities, AgentInfo, AgentTransport, AgentType, MotorCapability,
    SensoryCapability, VisionCapability, VisualizationCapability,
};
pub use config::{PNSConfig, TransportMode, WebSocketConfig};
pub use events::{
    AgentDisconnectedEvent, AgentRegisteredEvent, MotorCommandEvent, SensoryDataEvent,
    VisualizationReadyEvent,
};
pub use heartbeat::HeartbeatTracker;
pub use registration::{RegistrationHandler, RegistrationRequest, RegistrationResponse, TransportConfig};
pub use types::{PNSError, Result, SharedFBC, StreamType};
pub use type_validation::{
    validate_sensory_compatibility, validate_motor_compatibility,
    get_recommended_buffer_size, should_use_compression, ValidationResult,
};
