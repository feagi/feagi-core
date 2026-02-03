//! Server-side networking traits for FEAGI.
//!
//! This module defines the core abstractions for server-side network communication,
//! supporting multiple messaging patterns:
//!
//! - **Publisher** ([`FeagiServerPublisher`]): One-to-many broadcast pattern
//! - **Puller** ([`FeagiServerPuller`]): Receives pushed data from clients
//! - **Router** ([`FeagiServerRouter`]): Request-response pattern with automatic routing

mod feagi_server;
mod feagi_server_publisher;
mod feagi_server_puller;
mod feagi_server_router;

pub use feagi_server::FeagiServer;
pub use feagi_server_publisher::FeagiServerPublisher;
pub use feagi_server_publisher::FeagiServerPublisherProperties;
pub use feagi_server_puller::FeagiServerPuller;
pub use feagi_server_router::FeagiServerRouter;
