//! WebSocket transport implementations.
//!
//! This module provides WebSocket-based implementations of the FEAGI networking traits
//! using the `tungstenite` crate with non-blocking sockets. All operations use
//! `set_nonblocking(true)` and check for `WouldBlock`, making them compatible
//! with any async runtime or synchronous usage.
//!
//! # Socket Patterns
//!
//! | Server | Client | Pattern |
//! |--------|--------|---------|
//! | [`FeagiWebSocketServerPublisher`] | [`FeagiWebSocketClientSubscriber`] | Pub/Sub (broadcast) |
//! | [`FeagiWebSocketServerPuller`] | [`FeagiWebSocketClientPusher`] | Push/Pull (pipeline) |
//! | [`FeagiWebSocketServerRouter`] | [`FeagiWebSocketClientRequester`] | Router/Dealer (req/rep) |
//!
//! # Creating Instances
//!
//! All server and client instances are created through their Properties types:
//!
//! ```ignore
//! // Server example
//! let props = FeagiWebSocketServerPublisherProperties::new("127.0.0.1:8080")?;
//! let mut server = props.as_boxed_server_publisher();
//!
//! // Client example
//! let props = FeagiWebSocketClientSubscriberProperties::new("ws://localhost:8080")?;
//! let mut client = props.as_boxed_client_subscriber();
//! ```
//!
#[cfg(feature = "feagi-server")]
mod server_implementations;

#[cfg(feature = "feagi-client")]
mod client_implementations;

mod shared;

#[cfg(feature = "feagi-server")]
// Server implementations and properties
pub use server_implementations::{
    FeagiWebSocketServerPublisher,
    FeagiWebSocketServerPublisherProperties,
    FeagiWebSocketServerPuller,
    FeagiWebSocketServerPullerProperties,
    FeagiWebSocketServerRouter,
    FeagiWebSocketServerRouterProperties,
};

#[cfg(feature = "feagi-client")]
// Client implementations and properties
pub use client_implementations::{
    FeagiWebSocketClientPusher,
    FeagiWebSocketClientPusherProperties,
    FeagiWebSocketClientRequester,
    FeagiWebSocketClientRequesterProperties,
    FeagiWebSocketClientSubscriber,
    FeagiWebSocketClientSubscriberProperties,
};


