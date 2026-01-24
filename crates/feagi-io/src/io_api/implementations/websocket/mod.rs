//! WebSocket implementations for FEAGI network traits.
//!
//! Uses `tungstenite` for synchronous WebSocket communication with
//! a polling API that matches the ZMQ implementations.

#[cfg(feature = "websocket-transport")]
mod client_implementations;
#[cfg(feature = "websocket-transport")]
mod server_implementations;

#[cfg(feature = "websocket-transport")]
pub use client_implementations::{
    FEAGIWebSocketClientPusher, FEAGIWebSocketClientPusherProperties,
    FEAGIWebSocketClientRequester, FEAGIWebSocketClientRequesterProperties,
    FEAGIWebSocketClientSubscriber, FEAGIWebSocketClientSubscriberProperties,
};
#[cfg(feature = "websocket-transport")]
pub use server_implementations::{
    FEAGIWebSocketServerPublisher, FEAGIWebSocketServerPublisherProperties,
    FEAGIWebSocketServerPuller, FEAGIWebSocketServerPullerProperties, FEAGIWebSocketServerRouter,
    FEAGIWebSocketServerRouterProperties,
};
