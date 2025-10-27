//! Non-blocking I/O infrastructure for async/await transports
//!
//! This module provides reusable infrastructure for transports that use
//! async/await with tokio runtime:
//! - NonBlockingTransport trait
//! - Tokio runtime helpers
//! - Async channels
//! - Async LZ4 compression

pub mod transport;
pub mod runtime;
pub mod channels;
pub mod compression;

// Re-export main trait
pub use transport::NonBlockingTransport;

