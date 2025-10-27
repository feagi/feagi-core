//! Core types and utilities shared across all transports
//!
//! This module contains transport-agnostic components:
//! - SharedFBC (Arc<FeagiByteContainer>) type alias
//! - Error types
//! - Agent registry and management
//! - Registration handling
//! - Heartbeat monitoring
//! - PNS configuration

pub mod types;
pub mod agent_registry;
pub mod registration;
pub mod heartbeat;
pub mod config;
pub mod events;

// Re-export commonly used types
pub use types::{SharedFBC, PNSError, StreamType, Result};
pub use agent_registry::{AgentRegistry, AgentInfo, AgentType, AgentCapabilities};
pub use registration::{RegistrationHandler, RegistrationRequest};
pub use heartbeat::HeartbeatTracker;
pub use config::{PNSConfig, TransportMode};
pub use events::{
    AgentDisconnectedEvent, AgentRegisteredEvent, MotorCommandEvent, SensoryDataEvent,
    VisualizationReadyEvent,
};

