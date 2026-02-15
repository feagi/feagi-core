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

    pub fn as_transport_protocol_implementation(&self) -> TransportProtocolImplementation {
        self.clone().into()
    }
}


// ========================================================================
    // Server Properties Factory Methods
    // ========================================================================

pub fn create_default_boxed_server_publisher_properties(server_bind: TransportProtocolEndpoint, agent_remote: TransportProtocolEndpoint) -> Result<Box<dyn FeagiServerPublisherProperties>, FeagiNetworkError> {

    match server_bind {
        #[cfg(any(feature = "websocket-transport-std", feature = "websocket-transport-wasm"))]
        TransportProtocolEndpoint::WebSocket(server_bind) => {
            match agent_remote {
                TransportProtocolEndpoint::WebSocket(agent_remote) => {
                    Ok(Box::new(FeagiWebSocketServerPublisherProperties::new(server_bind.as_str(), agent_remote.as_str()).unwrap()))
                }
                _ => Err(FeagiNetworkError::InvalidSocketProperties("Server bind and Agent remote cannot use different protocols!".to_string()))
            }
        }
        #[cfg(feature = "zmq-transport")]
        TransportProtocolEndpoint::Zmq(server_bind) => {
            match agent_remote {
                TransportProtocolEndpoint::Zmq(agent_endpoint) => {
                    Ok(Box::new(FeagiZmqServerPublisherProperties::new(server_bind.as_str(), agent_endpoint.as_str())?))
                }
                _ => Err(FeagiNetworkError::InvalidSocketProperties("Server bind and Agent remote cannot use different protocols!".to_string()))
            }
        }
    }


}

pub fn create_default_boxed_server_puller_properties(server_bind: TransportProtocolEndpoint, agent_remote: TransportProtocolEndpoint) -> Result<Box<dyn FeagiServerPullerProperties>, FeagiNetworkError> {
    match server_bind {
        #[cfg(any(feature = "websocket-transport-std", feature = "websocket-transport-wasm"))]
        TransportProtocolEndpoint::WebSocket(server_bind) => {
            match agent_remote {
                TransportProtocolEndpoint::WebSocket(agent_remote) => {
                    Ok(Box::new(FeagiWebSocketServerPullerProperties::new_with_remote(server_bind.as_str(), agent_remote.as_str()).unwrap()))
                }
                _ => Err(FeagiNetworkError::InvalidSocketProperties("Server bind and Agent remote cannot use different protocols!".to_string()))
            }
        }
        #[cfg(feature = "zmq-transport")]
        TransportProtocolEndpoint::Zmq(server_bind) => {
            match agent_remote {
                TransportProtocolEndpoint::Zmq(agent_endpoint) => {
                    Ok(Box::new(FeagiZmqServerPullerProperties::new(server_bind.as_str(), agent_endpoint.as_str())?))
                }
                _ => Err(FeagiNetworkError::InvalidSocketProperties("Server bind and Agent remote cannot use different protocols!".to_string()))
            }
        }
    }
}

pub fn create_default_boxed_server_router_properties(server_bind: TransportProtocolEndpoint, agent_remote: TransportProtocolEndpoint) -> Result<Box<dyn FeagiServerRouterProperties>, FeagiNetworkError> {
    match server_bind {
        #[cfg(any(feature = "websocket-transport-std", feature = "websocket-transport-wasm"))]
        TransportProtocolEndpoint::WebSocket(server_bind) => {
            match agent_remote {
                TransportProtocolEndpoint::WebSocket(agent_remote) => {
                    Ok(Box::new(FeagiWebSocketServerRouterProperties::new_with_remote(server_bind.as_str(), agent_remote.as_str()).unwrap()))
                }
                _ => Err(FeagiNetworkError::InvalidSocketProperties("Server bind and Agent remote cannot use different protocols!".to_string()))
            }
        }
        #[cfg(feature = "zmq-transport")]
        TransportProtocolEndpoint::Zmq(server_bind) => {
            match agent_remote {
                TransportProtocolEndpoint::Zmq(agent_endpoint) => {
                    Ok(Box::new(FeagiZmqServerRouterProperties::new(server_bind.as_str(), agent_endpoint.as_str())?))
                }
                _ => Err(FeagiNetworkError::InvalidSocketProperties("Server bind and Agent remote cannot use different protocols!".to_string()))
            }
        }
    }
}
