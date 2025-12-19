// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Transport primitives (consolidated from feagi-transports)
//!
//! This module provides low-level transport abstractions for ZMQ, WebSocket, UDP, and SHM.
//! These are used internally by feagi-io's domain-specific transport wrappers.

pub mod common;
pub mod traits;

#[cfg(feature = "zmq-transport")]
pub mod zmq;

#[cfg(feature = "websocket-transport")]
pub mod websocket;

// UDP and SHM modules are placeholders for now
// #[cfg(feature = "udp-transport")]
// pub mod udp;

// #[cfg(feature = "shm-transport")]
// pub mod shm;

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
    pub use crate::transports::core::common::*;
    pub use crate::transports::core::traits::*;

    #[cfg(feature = "zmq-transport")]
    pub use crate::transports::core::zmq::server::{ZmqPub, ZmqPull, ZmqRouter};

    #[cfg(feature = "zmq-transport")]
    pub use crate::transports::core::zmq::client::{ZmqDealer, ZmqPush, ZmqSub};

    #[cfg(feature = "websocket-transport")]
    pub use crate::transports::core::websocket::server::{WsPub, WsPull, WsRouter};

    #[cfg(feature = "websocket-transport")]
    pub use crate::transports::core::websocket::client::{WsDealer, WsPush, WsSub};
}

