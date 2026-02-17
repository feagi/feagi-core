//! ZMQ transport implementations.
//!
//! This module provides ZMQ-based implementations of the FEAGI networking traits
//! using the `zmq` crate (C bindings). All operations are non-blocking via
//! `zmq::DONTWAIT`, making them compatible with any async runtime or synchronous usage.
//!
//! # Socket Patterns
//!
//! | Server | Client | Pattern |
//! |--------|--------|---------|
//! | [`FeagiZmqServerPublisher`] | [`FeagiZmqClientSubscriber`] | Pub/Sub (broadcast) |
//! | [`FeagiZmqServerPuller`] | [`FeagiZmqClientPusher`] | Push/Pull (pipeline) |
//! | [`FeagiZmqServerRouter`] | [`FeagiZmqClientRequester`] | Router/Dealer (async req/rep) |
//!
//! # Creating Instances
//!
//! All server and client instances are created through their Properties types:
//!
//! ```ignore
//! // Server example
//! let props = FeagiZmqServerPublisherProperties::new("tcp://*:5555")?;
//! let mut server = props.as_boxed_server_publisher();
//!
//! // Client example
//! let props = FeagiZmqClientSubscriberProperties::new("tcp://localhost:5555")?;
//! let mut client = props.as_boxed_client_subscriber();
//! ```

mod client_implementations;
mod server_implementations;
mod shared;

// Client implementations and properties
pub use client_implementations::{
    FeagiZmqClientPusher, FeagiZmqClientPusherProperties, FeagiZmqClientRequester,
    FeagiZmqClientRequesterProperties, FeagiZmqClientSubscriber,
    FeagiZmqClientSubscriberProperties,
};

// Server implementations and properties
pub use server_implementations::{
    FeagiZmqServerPublisher, FeagiZmqServerPublisherProperties, FeagiZmqServerPuller,
    FeagiZmqServerPullerProperties, FeagiZmqServerRouter, FeagiZmqServerRouterProperties,
};
