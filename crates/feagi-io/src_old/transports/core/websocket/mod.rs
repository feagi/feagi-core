// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WebSocket transport implementation for FEAGI
//!
//! Provides WebSocket-based transport for all FEAGI communication patterns:
//! - Publisher/Subscriber (for visualization, motor commands)
//! - Push/Pull (for sensory data)
//! - Router/Dealer (for control commands)
//!
//! ## Features
//!
//! - Async-first design using tokio and tokio-tungstenite
//! - Support for both server and client roles
//! - Compatible with web browsers and native clients
//! - JSON and binary message formats
//! - Per-agent routing and isolation
//!
//! ## Example: WebSocket Publisher
//!
//! ```no_run
//! use feagi_transports::websocket::server::WsPub;
//! use feagi_transports::traits::{Transport, Publisher};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut publisher = WsPub::with_address("127.0.0.1:9050").await?;
//!     publisher.start().await?;
//!
//!     loop {
//!         publisher.publish(b"topic", b"data").await?;
//!         tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
//!     }
//! }
//! ```

#[cfg(feature = "websocket-transport")]
pub mod server;

#[cfg(feature = "websocket-transport")]
pub mod client;

pub mod common;

pub use common::*;
