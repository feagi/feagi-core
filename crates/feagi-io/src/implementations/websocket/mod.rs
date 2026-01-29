//! WebSocket implementations for FEAGI network traits.
//!
//! Uses `async-tungstenite` with tokio for async WebSocket communication
//! that matches the async trait signatures.

#[cfg(feature = "websocket-transport")]
mod client_implementations;
#[cfg(feature = "websocket-transport")]
mod server_implementations;

#[cfg(feature = "websocket-transport")]
pub use client_implementations::{
    FEAGIWebSocketClientPusher, FEAGIWebSocketClientRequester, FEAGIWebSocketClientSubscriber,
};
#[cfg(feature = "websocket-transport")]
pub use server_implementations::{
    FEAGIWebSocketServerPublisher, FEAGIWebSocketServerPuller, FEAGIWebSocketServerRouter,
};
