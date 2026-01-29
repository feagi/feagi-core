//! WebSocket implementations for FEAGI network traits.
//!
//! Uses `async-tungstenite` with tokio for async WebSocket communication
//! that matches the async trait signatures.

// NOTE: This module needs to be imported by feature from the implementations module!
mod client_implementations;
mod server_implementations;
mod shared_functions;

pub use client_implementations::{
    FEAGIWebSocketClientPusher, FEAGIWebSocketClientRequester, FEAGIWebSocketClientSubscriber,
};
pub use server_implementations::{
    FEAGIWebSocketServerPublisher, FEAGIWebSocketServerPuller, FEAGIWebSocketServerRouter,
};
