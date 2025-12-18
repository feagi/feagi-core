// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! WebSocket server implementations
//!
//! Provides server-side WebSocket transports for FEAGI:
//! - Publisher (PUB): Broadcast to multiple clients
//! - Pull: Receive from multiple clients
//! - Router: Request/reply with routing

pub mod pub_socket;
pub mod pull;
pub mod router;

pub use pub_socket::WsPub;
pub use pull::WsPull;
pub use router::WsRouter;
