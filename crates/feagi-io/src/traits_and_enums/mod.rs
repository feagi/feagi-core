//! Networking traits and shared types for FEAGI.
//!
//! This module provides runtime-agnostic trait definitions for network communication.
//! The traits use a poll-based design that works with any async runtime (tokio, embassy, WASM)
//! or can be used synchronously.
//!
//! # Architecture
//!
//! - [`FeagiEndpointState`]: Common state enum for all endpoints
//! - [`client`]: Client-side traits (connect to servers)
//! - [`server`]: Server-side traits (accept connections)
//!
//! # Design Philosophy
//!
//! These traits intentionally avoid `async fn` to remain runtime-agnostic. Implementations
//! are expected to be wrapped by runtime-specific adapters that provide async interfaces
//! where needed.

#[cfg(feature = "feagi-server")]
pub mod server;

#[cfg(feature = "feagi-client")]
pub mod client;
