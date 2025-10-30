//! # feagi-transports
//!
//! Transport abstraction layer for FEAGI, providing a unified interface for multiple
//! transport protocols including ZMQ, UDP, and shared memory.
//!
//! ## Features
//!
//! This crate supports both client and server implementations for various transport protocols:
//!
//! ### ZMQ (ZeroMQ)
//! - **Server**: ROUTER (request-reply), PUB (publish), PULL (receive)
//! - **Client**: DEALER (request-reply), SUB (subscribe), PUSH (send)
//!
//! ### UDP
//! - **Server**: UDP socket server for datagram reception
//! - **Client**: UDP socket client for datagram transmission
//!
//! ### Shared Memory (SHM)
//! - **Server**: Shared memory server for high-performance IPC
//! - **Client**: Shared memory client for high-performance IPC
//!
//! ## Feature Flags
//!
//! Control which transports and roles are compiled:
//!
//! ```toml
//! # Server-side only (FEAGI core)
//! [dependencies]
//! feagi-transports = { version = "2.0", features = ["server"] }
//!
//! # Client-side only (Rust agents)
//! [dependencies]
//! feagi-transports = { version = "2.0", features = ["client"] }
//!
//! # Everything
//! [dependencies]
//! feagi-transports = { version = "2.0", features = ["all"] }
//! ```
//!
//! Available feature flags:
//! - `zmq-server`: ZMQ server patterns
//! - `zmq-client`: ZMQ client patterns
//! - `udp-server`: UDP server
//! - `udp-client`: UDP client
//! - `shm-server`: Shared memory server
//! - `shm-client`: Shared memory client
//! - `server`: All server implementations
//! - `client`: All client implementations
//! - `all`: Everything
//!
//! ## Example: ZMQ Request-Reply
//!
//! ### Server
//!
//! ```no_run
//! use feagi_transports::zmq::server::ZmqRouter;
//! use feagi_transports::traits::{Transport, RequestReplyServer};
//!
//! let mut server = ZmqRouter::with_address("tcp://*:5555")?;
//! server.start()?;
//!
//! loop {
//!     let (request, reply_handle) = server.receive()?;
//!     println!("Received: {:?}", request);
//!     reply_handle.send(b"OK")?;
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Client
//!
//! ```no_run
//! use feagi_transports::zmq::client::ZmqDealer;
//! use feagi_transports::traits::{Transport, RequestReplyClient};
//!
//! let mut client = ZmqDealer::with_address("tcp://localhost:5555")?;
//! client.start()?;
//!
//! let response = client.request(b"Hello!")?;
//! println!("Response: {:?}", response);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Example: ZMQ Publish-Subscribe
//!
//! ### Publisher (Server)
//!
//! ```no_run
//! use feagi_transports::zmq::server::ZmqPub;
//! use feagi_transports::traits::{Transport, Publisher};
//!
//! let mut publisher = ZmqPub::with_address("tcp://*:5556")?;
//! publisher.start()?;
//!
//! loop {
//!     publisher.publish(b"topic", b"data")?;
//!     std::thread::sleep(std::time::Duration::from_millis(100));
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ### Subscriber (Client)
//!
//! ```no_run
//! use feagi_transports::zmq::client::ZmqSub;
//! use feagi_transports::traits::{Transport, Subscriber};
//!
//! let mut subscriber = ZmqSub::with_address("tcp://localhost:5556")?;
//! subscriber.start()?;
//! subscriber.subscribe(b"topic")?;
//!
//! loop {
//!     let (topic, data) = subscriber.receive()?;
//!     println!("Received: {:?} - {:?}", topic, data);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Architecture
//!
//! The crate is organized into layers:
//!
//! 1. **Common**: Shared types (errors, configs, messages)
//! 2. **Traits**: Transport-agnostic interfaces
//! 3. **Implementations**: Protocol-specific code (ZMQ, UDP, SHM)
//!
//! This design allows swapping transports without changing application logic.

pub mod common;
pub mod traits;

#[cfg(any(feature = "zmq-server", feature = "zmq-client"))]
pub mod zmq;

#[cfg(any(feature = "udp-server", feature = "udp-client"))]
pub mod udp;

#[cfg(any(feature = "shm-server", feature = "shm-client"))]
pub mod shm;

// Re-export commonly used types
pub use common::{
    ClientConfig, Message, MessageMetadata, MultipartMessage, ReplyHandle, ServerConfig,
    TransportConfig, TransportError, TransportResult,
};

pub use traits::{
    Publisher, Pull, Push, RequestReplyClient, RequestReplyServer, Subscriber, Transport,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::common::*;
    pub use crate::traits::*;

    #[cfg(feature = "zmq-server")]
    pub use crate::zmq::server::*;

    #[cfg(feature = "zmq-client")]
    pub use crate::zmq::client::*;
}
