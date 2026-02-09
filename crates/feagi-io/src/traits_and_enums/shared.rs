//! Shared types for FEAGI networking traits.

use serde::{Deserialize, Serialize};
use crate::FeagiNetworkError;

#[cfg(feature = "zmq-transport")]
use crate::protocol_implementations::zmq::ZmqUrl;
#[cfg(any(feature = "websocket-transport-std", feature = "websocket-transport-wasm"))]
use crate::protocol_implementations::websocket::WebSocketUrl;

// Client properties imports
#[cfg(feature = "websocket-transport-std")]
use crate::protocol_implementations::websocket::websocket_std::{
    FeagiWebSocketClientSubscriberProperties,
    FeagiWebSocketClientPusherProperties,
    FeagiWebSocketClientRequesterProperties,
};
#[cfg(feature = "zmq-transport")]
use crate::protocol_implementations::zmq::{
    FeagiZmqClientSubscriberProperties,
    FeagiZmqClientPusherProperties,
    FeagiZmqClientRequesterProperties,
};

// Server properties imports
#[cfg(feature = "websocket-transport-std")]
use crate::protocol_implementations::websocket::websocket_std::{
    FeagiWebSocketServerPublisherProperties,
    FeagiWebSocketServerPullerProperties,
    FeagiWebSocketServerRouterProperties,
};
#[cfg(feature = "zmq-transport")]
use crate::protocol_implementations::zmq::{
    FeagiZmqServerPublisherProperties,
    FeagiZmqServerPullerProperties,
    FeagiZmqServerRouterProperties,
};

// Trait imports
use crate::traits_and_enums::client::{
    FeagiClientSubscriberProperties,
    FeagiClientPusherProperties,
    FeagiClientRequesterProperties,
};
use crate::traits_and_enums::server::{
    FeagiServerPublisherProperties,
    FeagiServerPullerProperties,
    FeagiServerRouterProperties,
};

/// Represents the current state of a FEAGI network endpoint (client or server).
///
/// This enum is returned by `FeagiClient::poll()` and
/// `FeagiServer::poll()` to indicate what operations are valid
/// and whether data is available.
///
/// # State Machine
///
/// ```text
/// ┌──────────┐  request_connect/start   ┌─────────┐
/// │ Inactive │ ───────────────────────► │ Pending │
/// └──────────┘                          └────┬────┘
///       ▲                                    │
///       │ request_disconnect/stop            │ (connection established)
///       │                                    ▼
///       │                          ┌─────────────────┐
///       └────────────────────────  │  ActiveWaiting  │ ◄─────┐
///                                  └────────┬────────┘       │
///                                           │                │
///                                           │ (data arrives) │ (data consumed)
///                                           ▼                │
///                                  ┌─────────────────┐       │
///                                  │  ActiveHasData  │ ──────┘
///                                  └─────────────────┘
///
/// Any state can transition to Errored on failure.
/// Call confirm_error_and_close() to return to Inactive from Errored.
/// ```
///
/// # Usage Contract
///
/// - Always call `poll()` before performing send/receive operations
/// - Data operations (`publish_data`, `consume_*`) are only valid in `Active*` states
/// - Sending is valid in the `ActiveWaiting` state
/// - Consuming data is only meaningful in `ActiveHasData`
#[derive(Debug, Clone, PartialEq)]
pub enum FeagiEndpointState {
    /// The endpoint is not connected/bound and not attempting to connect/bind.
    ///
    /// Valid operations: `request_connect()` / `request_start()`
    Inactive,

    /// A connection/bind operation is in progress.
    ///
    /// Continue calling `poll()` until the state transitions to `ActiveWaiting`
    /// or `Errored`. No data operations are valid in this state.
    Pending,

    /// The endpoint is active and ready for data operations, but no incoming
    /// data is currently available.
    ///
    /// Valid operations:
    /// - Sending: `publish_data()`, `publish_request()`, `publish_response()`
    /// - Lifecycle: `request_disconnect()` / `request_stop()`
    ActiveWaiting,

    /// The endpoint is active and has incoming data available to consume.
    ///
    /// This state is only relevant for endpoints that receive data (subscribers,
    /// pullers, requesters awaiting responses, routers with pending requests).
    ///
    /// Valid operations:
    /// - Receiving: `consume_retrieved_data()`, `consume_retrieved_response()`, `consume_retrieved_request()`
    /// - Sending: `publish_data()`, `publish_request()`, `publish_response()`
    /// - Lifecycle: `request_disconnect()` / `request_stop()`
    ActiveHasData,

    /// The endpoint has encountered an error and is no longer operational.
    ///
    /// The contained error describes what went wrong. Call `confirm_error_and_close()`
    /// to acknowledge the error and return to `Inactive` state.
    ///
    /// This state is for persistent errors that require explicit acknowledgment,
    /// not for transient failures that are returned as `Result::Err` from individual
    /// operations.
    Errored(FeagiNetworkError),
}

/// Defines what type of transport protocol is being used
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum TransportProtocolImplementation {
    WebSocket,
    Zmq,
    BluetoothSerial,
    SharedMemory,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransportProtocolEndpoint {
    #[cfg(any(feature = "websocket-transport-std", feature = "websocket-transport-wasm"))]
    WebSocket(WebSocketUrl),

    #[cfg(feature = "zmq-transport")]
    Zmq(ZmqUrl)

    // TODO other implementations
}

impl From<TransportProtocolEndpoint> for TransportProtocolImplementation {
    fn from(t: TransportProtocolEndpoint) -> Self {
        match t {
            TransportProtocolEndpoint::Zmq(_) => {
                TransportProtocolImplementation::Zmq
            }
            TransportProtocolEndpoint::WebSocket(_) => {
                TransportProtocolImplementation::WebSocket
            }
        }
    }
}

impl TransportProtocolEndpoint {
    // ========================================================================
    // Client Properties Factory Methods
    // ========================================================================

    /// Creates a boxed client subscriber properties from this endpoint.
    pub fn create_boxed_client_subscriber_properties(&self) -> Box<dyn FeagiClientSubscriberProperties> {
        match self {
            #[cfg(any(feature = "websocket-transport-std", feature = "websocket-transport-wasm"))]
            TransportProtocolEndpoint::WebSocket(endpoint) => {
                #[cfg(feature = "websocket-transport-std")]
                return Box::new(FeagiWebSocketClientSubscriberProperties::new(endpoint.as_str()).unwrap());
                #[cfg(not(feature = "websocket-transport-std"))]
                panic!("WebSocket std is not included in this build!")
            }
            #[cfg(feature = "zmq-transport")]
            TransportProtocolEndpoint::Zmq(endpoint) => {
                Box::new(FeagiZmqClientSubscriberProperties::new(endpoint.as_str()).unwrap())
            }
        }
    }

    /// Creates a boxed client pusher properties from this endpoint.
    pub fn create_boxed_client_pusher_properties(&self) -> Box<dyn FeagiClientPusherProperties> {
        match self {
            #[cfg(any(feature = "websocket-transport-std", feature = "websocket-transport-wasm"))]
            TransportProtocolEndpoint::WebSocket(endpoint) => {
                #[cfg(feature = "websocket-transport-std")]
                return Box::new(FeagiWebSocketClientPusherProperties::new(endpoint.as_str()).unwrap());
                #[cfg(not(feature = "websocket-transport-std"))]
                panic!("WebSocket std is not included in this build!")
            }
            #[cfg(feature = "zmq-transport")]
            TransportProtocolEndpoint::Zmq(endpoint) => {
                Box::new(FeagiZmqClientPusherProperties::new(endpoint.as_str()).unwrap())
            }
        }
    }

    /// Creates a boxed client requester properties from this endpoint.
    pub fn create_boxed_client_requester_properties(&self) -> Box<dyn FeagiClientRequesterProperties> {
        match self {
            #[cfg(any(feature = "websocket-transport-std", feature = "websocket-transport-wasm"))]
            TransportProtocolEndpoint::WebSocket(endpoint) => {
                #[cfg(feature = "websocket-transport-std")]
                return Box::new(FeagiWebSocketClientRequesterProperties::new(endpoint.as_str()).unwrap());
                #[cfg(not(feature = "websocket-transport-std"))]
                panic!("WebSocket std is not included in this build!")
            }
            #[cfg(feature = "zmq-transport")]
            TransportProtocolEndpoint::Zmq(endpoint) => {
                Box::new(FeagiZmqClientRequesterProperties::new(endpoint.as_str()).unwrap())
            }
        }
    }

    // ========================================================================
    // Server Properties Factory Methods
    // ========================================================================

    /// Creates a boxed server publisher properties from this endpoint.
    pub fn create_boxed_server_publisher_properties(&self) -> Box<dyn FeagiServerPublisherProperties> {
        match self {
            #[cfg(any(feature = "websocket-transport-std", feature = "websocket-transport-wasm"))]
            TransportProtocolEndpoint::WebSocket(endpoint) => {
                #[cfg(feature = "websocket-transport-std")]
                return Box::new(FeagiWebSocketServerPublisherProperties::new(endpoint.as_str()).unwrap());
                #[cfg(not(feature = "websocket-transport-std"))]
                panic!("WebSocket std is not included in this build!")
            }
            #[cfg(feature = "zmq-transport")]
            TransportProtocolEndpoint::Zmq(endpoint) => {
                Box::new(FeagiZmqServerPublisherProperties::new(endpoint.as_str()).unwrap())
            }
        }
    }

    /// Creates a boxed server puller properties from this endpoint.
    pub fn create_boxed_server_puller_properties(&self) -> Box<dyn FeagiServerPullerProperties> {
        match self {
            #[cfg(any(feature = "websocket-transport-std", feature = "websocket-transport-wasm"))]
            TransportProtocolEndpoint::WebSocket(endpoint) => {
                #[cfg(feature = "websocket-transport-std")]
                return Box::new(FeagiWebSocketServerPullerProperties::new(endpoint.as_str()).unwrap());
                #[cfg(not(feature = "websocket-transport-std"))]
                panic!("WebSocket std is not included in this build!")
            }
            #[cfg(feature = "zmq-transport")]
            TransportProtocolEndpoint::Zmq(endpoint) => {
                Box::new(FeagiZmqServerPullerProperties::new(endpoint.as_str()).unwrap())
            }
        }
    }

    /// Creates a boxed server router properties from this endpoint.
    pub fn create_boxed_server_router_properties(&self) -> Box<dyn FeagiServerRouterProperties> {
        match self {
            #[cfg(any(feature = "websocket-transport-std", feature = "websocket-transport-wasm"))]
            TransportProtocolEndpoint::WebSocket(endpoint) => {
                #[cfg(feature = "websocket-transport-std")]
                return Box::new(FeagiWebSocketServerRouterProperties::new(endpoint.as_str()).unwrap());
                #[cfg(not(feature = "websocket-transport-std"))]
                panic!("WebSocket std is not included in this build!")
            }
            #[cfg(feature = "zmq-transport")]
            TransportProtocolEndpoint::Zmq(endpoint) => {
                Box::new(FeagiZmqServerRouterProperties::new(endpoint.as_str()).unwrap())
            }
        }
    }
}
