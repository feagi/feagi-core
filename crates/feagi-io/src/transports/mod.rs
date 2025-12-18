// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Transport implementations
//!
//! Each transport module implements either BlockingTransport or NonBlockingTransport
//! (or both for dual-mode transports like ZMQ).
//!
//! Available transports:
//! - zmq: ZeroMQ (blocking, TCP-based, reliable) [feature: zmq-transport]
//! - udp: User Datagram Protocol (nonblocking, best-effort, high-throughput) [feature: udp-transport]
//! - shm: Shared Memory (blocking only, future) [feature: shm-transport]
//! - websocket: WebSocket (nonblocking only, future) [feature: websocket-transport]
//! - rtos: Embedded/RTOS (special no_std, future) [build flag: --target]

#[cfg(feature = "zmq-transport")]
pub mod zmq;

#[cfg(feature = "udp-transport")]
pub mod udp;

#[cfg(feature = "websocket-transport")]
pub mod websocket;

// Future transports (placeholder modules will be added as needed):
// #[cfg(feature = "shm-transport")]
// pub mod shm;
// #[cfg(feature = "rtos-transport")]
// pub mod rtos;
