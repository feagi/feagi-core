// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WebSocket transport for PNS
//!
//! Provides WebSocket server implementations for all FEAGI streams:
//! - Sensory stream (agents → FEAGI)
//! - Motor stream (FEAGI → agents)
//! - Visualization stream (FEAGI → clients)
//! - Registration/control stream (bidirectional)
//!
//! Uses the feagi-transports WebSocket implementation under the hood.

#[cfg(feature = "websocket-transport")]
pub mod streams;

#[cfg(feature = "websocket-transport")]
pub use streams::WebSocketStreams;
