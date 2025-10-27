//! Blocking I/O infrastructure for thread-based transports
//!
//! This module provides reusable infrastructure for transports that use
//! blocking I/O operations with dedicated worker threads:
//! - BlockingTransport trait
//! - Worker thread patterns
//! - Bounded channels for backpressure
//! - LZ4 compression utilities

pub mod transport;
pub mod channels;
pub mod worker;
pub mod compression;

// Re-export main trait
pub use transport::BlockingTransport;

