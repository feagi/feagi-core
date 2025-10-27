//! Core types used across all PNS modules

use feagi_data_serialization::FeagiByteContainer;
use std::sync::Arc;
use thiserror::Error;

/// Type alias for thread-safe shared reference to FBC
/// NOT a custom type - just Arc wrapper around existing FeagiByteContainer!
///
/// # Purpose
/// - ✅ Uses `FeagiByteContainer` exactly as-is (no modifications)
/// - ✅ Arc wrapper only for thread-safety (Rust requirement for sharing across threads)
/// - ✅ All FBC methods available: `get_byte_ref()`, `overwrite_byte_data_*()`, etc.
/// - ✅ Zero custom code - leverages existing FBC capabilities completely
///
/// # Why Arc?
/// - Arc is required by Rust to share FBC across threads (PNS worker threads, async tasks)
/// - Without Arc, would need to copy FBC each time (expensive!)
/// - Arc provides zero-copy sharing via reference counting
pub type SharedFBC = Arc<FeagiByteContainer>;

/// Stream types for PNS communication
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StreamType {
    /// REST API and registration (reliable, TCP)
    Rest,
    /// Motor commands to agents (reliable, TCP)
    Motor,
    /// Visualization data to Brain Visualizer (high-throughput, configurable transport)
    Visualization,
    /// Sensory data from agents (high-throughput, configurable transport)
    Sensory,
}

/// Errors that can occur in PNS operations
#[derive(Error, Debug)]
pub enum PNSError {
    #[error("ZMQ error: {0}")]
    Zmq(String),
    #[error("SHM error: {0}")]
    Shm(String),
    #[error("Agent error: {0}")]
    Agent(String),
    #[error("Registration error: {0}")]
    Registration(String),
    #[error("Not running: {0}")]
    NotRunning(String),
    #[error("Transport error: {0}")]
    Transport(String),
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Result type for PNS operations
pub type Result<T> = std::result::Result<T, PNSError>;

