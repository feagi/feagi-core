// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! ZMQ transport implementations
//!
//! This module provides both client and server implementations of common ZMQ socket patterns:
//! - **Request-Reply**: ROUTER (server) ↔ DEALER (client)
//! - **Publish-Subscribe**: PUB (server) ↔ SUB (client)
//! - **Push-Pull**: PULL (server) ↔ PUSH (client)
//!
//! ## Feature Flags
//!
//! - `zmq-server`: Enable server-side sockets (ROUTER, PUB, PULL)
//! - `zmq-client`: Enable client-side sockets (DEALER, SUB, PUSH)
//! - `zmq`: Enable both client and server
//!
//! ## Example (Server)
//!
//! ```no_run
//! use feagi_transports::zmq::server::ZmqRouter;
//! use feagi_transports::traits::{Transport, RequestReplyServer};
//!
//! # fn process_request(request: &[u8]) -> Vec<u8> { b"response".to_vec() }
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut server = ZmqRouter::with_address("tcp://*:5555")?;
//! server.start()?;
//!
//! loop {
//!     let (request, reply_handle) = server.receive()?;
//!     let response = process_request(&request);
//!     reply_handle.send(&response)?;
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Example (Client)
//!
//! ```no_run
//! use feagi_transports::zmq::client::ZmqDealer;
//! use feagi_transports::traits::{Transport, RequestReplyClient};
//!
//! let mut client = ZmqDealer::with_address("tcp://localhost:5555")?;
//! client.start()?;
//!
//! let response = client.request(b"Hello, FEAGI!")?;
//! println!("Response: {:?}", response);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#[cfg(feature = "zmq-server")]
pub mod server;

#[cfg(feature = "zmq-client")]
pub mod client;

#[cfg(feature = "zmq-server")]
pub use server::{ZmqPub, ZmqPull, ZmqRouter};

#[cfg(feature = "zmq-client")]
pub use client::{ZmqDealer, ZmqPush, ZmqSub};

