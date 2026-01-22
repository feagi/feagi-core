//! WebSocket implementations for FEAGI network traits.
//!
//! Uses `tungstenite` for synchronous WebSocket communication with
//! a polling API that matches the ZMQ implementations.

#[cfg(feature = "websocket-transport")]
mod server_implementations;
#[cfg(feature = "websocket-transport")]
mod client_implementations;

#[cfg(feature = "websocket-transport")]
pub use server_implementations::{
    FEAGIWebSocketServerPublisher, FEAGIWebSocketServerPuller, FEAGIWebSocketServerRouter,
    FEAGIWebSocketServerPublisherProperties, FEAGIWebSocketServerPullerProperties, FEAGIWebSocketServerRouterProperties
};
#[cfg(feature = "websocket-transport")]
pub use client_implementations::{
    FEAGIWebSocketClientSubscriber, FEAGIWebSocketClientPusher, FEAGIWebSocketClientRequester,
    FEAGIWebSocketClientSubscriberProperties, FEAGIWebSocketClientPusherProperties, FEAGIWebSocketClientRequesterProperties
};
