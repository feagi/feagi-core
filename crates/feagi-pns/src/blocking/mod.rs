//! Blocking I/O infrastructure for thread-based transports
//!
//! This module provides reusable infrastructure for transports that use
//! blocking I/O operations with dedicated worker threads:
//! - BlockingTransport trait
//! - Worker thread patterns
//! - Bounded channels for backpressure
//! - LZ4 compression utilities

pub mod channels;
pub mod compression;
pub mod transport;
pub mod worker;

// Re-export main trait
pub use transport::BlockingTransport;



