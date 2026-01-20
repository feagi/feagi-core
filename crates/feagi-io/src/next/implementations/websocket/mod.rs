//! WebSocket implementations for FEAGI network traits.
//!
//! Uses `tokio-tungstenite` for async WebSocket communication with
//! a synchronous polling API that matches the ZMQ implementations.

#[cfg(feature = "websocket-transport")]
mod server_implementations;
#[cfg(feature = "websocket-transport")]
mod client_implementations;

#[cfg(feature = "websocket-transport")]
pub use server_implementations::{FEAGIWebSocketServerPublisher, FEAGIWebSocketServerPuller, FEAGIWebSocketServerRouter};
#[cfg(feature = "websocket-transport")]
pub use client_implementations::{FEAGIWebSocketClientSubscriber, FEAGIWebSocketClientPusher, FEAGIWebSocketClientRequester};
