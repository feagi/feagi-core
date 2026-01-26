// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Blocking transport trait for thread-based I/O
//!
//! Transports implementing this trait use blocking I/O operations with dedicated
//! worker threads for handling network communication.

use crate::core::{Result, SharedFBC};

/// Trait for transports that use blocking I/O with worker threads
///
/// # Design
/// - Operations block the calling thread until complete or timeout
/// - Typically used with dedicated worker threads for async behavior
/// - Good for: ZMQ (blocking API), SHM (file-based), traditional sockets
///
/// # Thread Safety
/// - All methods must be safe to call from multiple threads (`Send + Sync`)
/// - Internal state should use appropriate synchronization primitives
///
/// # Example Implementation
/// ```no_run
/// use feagi_io::blocking::BlockingTransport;
/// use feagi_io::core::{Result, SharedFBC, StreamType};
///
/// struct ZmqTransport {
///     context: zmq::Context,
///     // ... sockets, workers, etc.
/// }
///
/// impl BlockingTransport for ZmqTransport {
///     fn backend_name(&self) -> &str {
///         "zmq-tcp"
///     }
///
///     fn start(&mut self) -> Result<()> {
///         // Start worker threads, bind sockets, etc.
///         Ok(())
///     }
///
///     fn stop(&mut self) -> Result<()> {
///         // Stop workers, close sockets, etc.
///         Ok(())
///     }
///
///     fn publish_visualization(&self, fbc: SharedFBC) -> Result<()> {
///         // Compress and send via worker queue
///         Ok(())
///     }
///
///     fn publish_motor(&self, agent_id: &str, fbc: SharedFBC) -> Result<()> {
///         // Send motor commands to specific agent
///         Ok(())
///     }
/// }
/// ```
pub trait BlockingTransport: Send + Sync {
    /// Get the transport backend name (e.g., "zmq-tcp", "shm", "uart")
    fn backend_name(&self) -> &str;

    /// Start the transport (bind sockets, spawn workers, etc.)
    fn start(&mut self) -> Result<()>;

    /// Stop the transport (shutdown workers, close connections, etc.)
    fn stop(&mut self) -> Result<()>;

    /// Publish visualization data to all subscribers
    ///
    /// # Arguments
    /// - `fbc`: Shared reference to FeagiByteContainer with neural activity data
    ///
    /// # Behavior
    /// - May compress data before sending (transport-specific)
    /// - May queue if immediate send would block
    /// - Should NOT drop frames silently (use backpressure instead)
    ///
    /// # Performance Notes
    /// - Arc clone is cheap (just refcount increment)
    /// - Use `fbc.get_byte_ref()` for zero-copy read access
    /// - Compression should happen in worker thread, not caller
    fn publish_visualization(&self, fbc: SharedFBC) -> Result<()>;

    /// Publish motor commands to a specific agent
    ///
    /// # Arguments
    /// - `agent_id`: Target agent identifier
    /// - `fbc`: Shared reference to FeagiByteContainer with motor command data
    ///
    /// # Behavior
    /// - Must be reliable (no drops)
    /// - May compress data before sending
    /// - Should block on backpressure (motor commands are critical)
    fn publish_motor(&self, agent_id: &str, fbc: SharedFBC) -> Result<()>;
}
