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

// Re-export commonly used types
pub use agent_registry::{
    AgentCapabilities, AgentInfo, AgentRegistry, AgentTransport, AgentType, MotorCapability,
    VisionCapability, VisualizationCapability,
};
pub use config::{PNSConfig, TransportMode};
pub use events::{
    AgentDisconnectedEvent, AgentRegisteredEvent, MotorCommandEvent, SensoryDataEvent,
    VisualizationReadyEvent,
};
pub use heartbeat::HeartbeatTracker;
pub use registration::{RegistrationHandler, RegistrationRequest};
pub use types::{PNSError, Result, SharedFBC, StreamType};
