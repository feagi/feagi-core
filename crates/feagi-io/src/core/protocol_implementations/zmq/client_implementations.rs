//! ZMQ client implementations using the poll-based trait design.
//!
//! These implementations use the `zmq` crate (C bindings) which provides
//! true non-blocking operations via `zmq::DONTWAIT`, making them compatible
//! with any async runtime or synchronous usage.
//!
//! # Memory Efficiency
//!
//! These implementations reuse `zmq::Message` objects to minimize allocations.
//! ZMQ handles internal memory pooling and zero-copy optimizations.

use zmq::{Context, Message, Socket};

use crate::FeagiNetworkError;
use crate::core::protocol_implementations::zmq::shared::ZmqUrl;
use crate::core::traits_and_enums::FeagiEndpointState;
use crate::core::traits_and_enums::client::{
    FeagiClient, FeagiClientPusher, FeagiClientPusherProperties,
    FeagiClientRequester, FeagiClientRequesterProperties,
    FeagiClientSubscriber, FeagiClientSubscriberProperties,
};

// ============================================================================
// Subscriber
// ============================================================================

//region Subscriber Properties

/// Configuration properties for creating a ZMQ SUB client.
///
/// This allows storing configuration separately from active instances,
/// enabling creation of new subscribers with the same settings.
#[derive(Debug, Clone, PartialEq)]
pub struct FeagiZmqClientSubscriberProperties {
    server_address: ZmqUrl,
}

impl FeagiZmqClientSubscriberProperties {
    /// Creates new subscriber properties with the given server address.
    ///
    /// # Arguments
    ///
    /// * `server_address` - The ZMQ address to connect to (e.g., "tcp://localhost:5555").
    ///
    /// # Errors
    ///
    /// Returns an error if the address is invalid.
    pub fn new(server_address: &str) -> Result<Self, FeagiNetworkError> {
        let zmq_url = ZmqUrl::new(server_address)?;
        Ok(Self { server_address: zmq_url })
    }
}

impl FeagiClientSubscriberProperties for FeagiZmqClientSubscriberProperties {
    fn as_boxed_client_subscriber(&self) -> Box<dyn FeagiClientSubscriber> {
        let context = Context::new();
        let socket = context
            .socket(zmq::SUB)
            .expect("Failed to create ZMQ SUB socket");

        // Subscribe to all messages (empty filter)
        socket
            .set_subscribe(b"")
            .expect("Failed to set ZMQ subscription filter");

        Box::new(FeagiZmqClientSubscriber {
            server_address: self.server_address.clone(),
            current_state: FeagiEndpointState::Inactive,
            context,
            socket,
            recv_msg: Message::new(),
            has_data: false,
        })
    }
}

//endregion

//region Subscriber Implementation

/// A ZMQ SUB client that receives broadcast data from a publisher.
///
/// Uses a reusable `zmq::Message` to minimize memory allocations.
///
/// # Example
///
/// ```ignore
/// let props = FeagiZmqClientSubscriberProperties::new("tcp://localhost:5555")?;
/// let mut subscriber = props.as_boxed_client_subscriber();
///
/// subscriber.request_connect()?;
/// loop {
///     match subscriber.poll() {
///         FeagiEndpointState::ActiveHasData => {
///             let data = subscriber.consume_retrieved_data()?;
///             process(data);
///         }
///         FeagiEndpointState::ActiveWaiting => { /* no data yet */ }
///         _ => break,
///     }
/// }
/// ```
pub struct FeagiZmqClientSubscriber {
    server_address: ZmqUrl,
    current_state: FeagiEndpointState,
    #[allow(dead_code)]
    context: Context,
    socket: Socket,
    /// Reusable message buffer
    recv_msg: Message,
    /// Whether recv_msg contains valid data
    has_data: bool,
}

impl FeagiClient for FeagiZmqClientSubscriber {
    fn poll(&mut self) -> &FeagiEndpointState {
        if matches!(self.current_state, FeagiEndpointState::ActiveWaiting) && !self.has_data {
            match self.socket.recv(&mut self.recv_msg, zmq::DONTWAIT) {
                Ok(()) => {
                    self.has_data = true;
                    self.current_state = FeagiEndpointState::ActiveHasData;
                }
                Err(zmq::Error::EAGAIN) => {
                    // No data available
                }
                Err(e) => {
                    self.current_state = FeagiEndpointState::Errored(
                        FeagiNetworkError::ReceiveFailed(e.to_string()),
                    );
                }
            }
        }
        &self.current_state
    }

    fn request_connect(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::Inactive => {
                self.socket
                    .connect(&self.server_address.to_string())
                    .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;

                self.current_state = FeagiEndpointState::ActiveWaiting;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot connect: client is not in Inactive state".to_string(),
            )),
        }
    }

    fn request_disconnect(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                self.socket
                    .disconnect(&self.server_address.to_string())
                    .map_err(|e| FeagiNetworkError::CannotDisconnect(e.to_string()))?;

                self.has_data = false;
                self.current_state = FeagiEndpointState::Inactive;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot disconnect: client is not in Active state".to_string(),
            )),
        }
    }

    fn confirm_error_and_close(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::Errored(_) => {
                let _ = self.socket.disconnect(&self.server_address.to_string());
                self.has_data = false;
                self.current_state = FeagiEndpointState::Inactive;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot confirm error: client is not in Errored state".to_string(),
            )),
        }
    }
}

impl FeagiClientSubscriber for FeagiZmqClientSubscriber {
    fn consume_retrieved_data(&mut self) -> Result<&[u8], FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveHasData => {
                if self.has_data {
                    self.has_data = false;
                    self.current_state = FeagiEndpointState::ActiveWaiting;
                    Ok(&self.recv_msg)
                } else {
                    Err(FeagiNetworkError::ReceiveFailed(
                        "No data available despite ActiveHasData state".to_string(),
                    ))
                }
            }
            _ => Err(FeagiNetworkError::ReceiveFailed(
                "Cannot consume: no data available".to_string(),
            )),
        }
    }
}

//endregion

// ============================================================================
// Pusher
// ============================================================================

//region Pusher Properties

/// Configuration properties for creating a ZMQ PUSH client.
///
/// This allows storing configuration separately from active instances,
/// enabling creation of new pushers with the same settings.
#[derive(Debug, Clone, PartialEq)]
pub struct FeagiZmqClientPusherProperties {
    server_address: ZmqUrl,
}

impl FeagiZmqClientPusherProperties {
    /// Creates new pusher properties with the given server address.
    ///
    /// # Arguments
    ///
    /// * `server_address` - The ZMQ address to connect to (e.g., "tcp://localhost:5556").
    ///
    /// # Errors
    ///
    /// Returns an error if the address is invalid.
    pub fn new(server_address: &str) -> Result<Self, FeagiNetworkError> {
        let zmq_url = ZmqUrl::new(server_address)?;
        Ok(Self { server_address: zmq_url })
    }
}

impl FeagiClientPusherProperties for FeagiZmqClientPusherProperties {
    fn as_boxed_client_pusher(&self) -> Box<dyn FeagiClientPusher> {
        let context = Context::new();
        let socket = context
            .socket(zmq::PUSH)
            .expect("Failed to create ZMQ PUSH socket");

        Box::new(FeagiZmqClientPusher {
            server_address: self.server_address.clone(),
            current_state: FeagiEndpointState::Inactive,
            context,
            socket,
        })
    }
}

//endregion

//region Pusher Implementation

/// A ZMQ PUSH client that sends data to a PULL server.
///
/// # Example
///
/// ```ignore
/// let props = FeagiZmqClientPusherProperties::new("tcp://localhost:5556")?;
/// let mut pusher = props.as_boxed_client_pusher();
///
/// pusher.request_connect()?;
/// while pusher.poll() != FeagiEndpointState::ActiveWaiting {
///     // wait for connection
/// }
///
/// pusher.publish_data(b"Hello server!")?;
/// ```
pub struct FeagiZmqClientPusher {
    server_address: ZmqUrl,
    current_state: FeagiEndpointState,
    #[allow(dead_code)]
    context: Context,
    socket: Socket,
}

impl FeagiClient for FeagiZmqClientPusher {
    fn poll(&mut self) -> &FeagiEndpointState {
        // PUSH sockets don't receive data, just return current state
        &self.current_state
    }

    fn request_connect(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::Inactive => {
                self.socket
                    .connect(&self.server_address.to_string())
                    .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;

                self.current_state = FeagiEndpointState::ActiveWaiting;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot connect: client is not in Inactive state".to_string(),
            )),
        }
    }

    fn request_disconnect(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                self.socket
                    .disconnect(&self.server_address.to_string())
                    .map_err(|e| FeagiNetworkError::CannotDisconnect(e.to_string()))?;

                self.current_state = FeagiEndpointState::Inactive;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot disconnect: client is not in Active state".to_string(),
            )),
        }
    }

    fn confirm_error_and_close(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::Errored(_) => {
                let _ = self.socket.disconnect(&self.server_address.to_string());
                self.current_state = FeagiEndpointState::Inactive;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot confirm error: client is not in Errored state".to_string(),
            )),
        }
    }
}

impl FeagiClientPusher for FeagiZmqClientPusher {
    fn publish_data(&mut self, data: &[u8]) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                self.socket
                    .send(data, zmq::DONTWAIT)
                    .map_err(|e| {
                        if e == zmq::Error::EAGAIN {
                            FeagiNetworkError::SendFailed("Socket would block".to_string())
                        } else {
                            FeagiNetworkError::SendFailed(e.to_string())
                        }
                    })?;
                Ok(())
            }
            _ => Err(FeagiNetworkError::SendFailed(
                "Cannot publish: client is not in Active state".to_string(),
            )),
        }
    }
}

//endregion

// ============================================================================
// Requester (Dealer)
// ============================================================================

//region Requester Properties

/// Configuration properties for creating a ZMQ DEALER client.
///
/// This allows storing configuration separately from active instances,
/// enabling creation of new requesters with the same settings.
#[derive(Debug, Clone, PartialEq)]
pub struct FeagiZmqClientRequesterProperties {
    server_address: ZmqUrl,
}

impl FeagiZmqClientRequesterProperties {
    /// Creates new requester properties with the given server address.
    ///
    /// # Arguments
    ///
    /// * `server_address` - The ZMQ address to connect to (e.g., "tcp://localhost:5557").
    ///
    /// # Errors
    ///
    /// Returns an error if the address is invalid.
    pub fn new(server_address: &str) -> Result<Self, FeagiNetworkError> {
        let zmq_url = ZmqUrl::new(server_address)?;
        Ok(Self { server_address: zmq_url })
    }
}

impl FeagiClientRequesterProperties for FeagiZmqClientRequesterProperties {
    fn as_boxed_client_requester(&self) -> Box<dyn FeagiClientRequester> {
        let context = Context::new();
        let socket = context
            .socket(zmq::DEALER)
            .expect("Failed to create ZMQ DEALER socket");

        Box::new(FeagiZmqClientRequester {
            server_address: self.server_address.clone(),
            current_state: FeagiEndpointState::Inactive,
            context,
            socket,
            delimiter_msg: Message::new(),
            payload_msg: Message::new(),
            has_data: false,
        })
    }
}

//endregion

//region Requester Implementation

/// A ZMQ DEALER client that sends requests and receives responses.
///
/// Uses DEALER socket (async REQ) to allow non-blocking request/response flow.
/// Uses reusable `zmq::Message` objects to minimize allocations.
///
/// # Example
///
/// ```ignore
/// let props = FeagiZmqClientRequesterProperties::new("tcp://localhost:5557")?;
/// let mut requester = props.as_boxed_client_requester();
///
/// requester.request_connect()?;
/// while requester.poll() != FeagiEndpointState::ActiveWaiting {
///     // wait for connection
/// }
///
/// requester.publish_request(b"Hello?")?;
/// loop {
///     match requester.poll() {
///         FeagiEndpointState::ActiveHasData => {
///             let response = requester.consume_retrieved_response()?;
///             process(response);
///             break;
///         }
///         FeagiEndpointState::ActiveWaiting => { /* still waiting */ }
///         _ => break,
///     }
/// }
/// ```
pub struct FeagiZmqClientRequester {
    server_address: ZmqUrl,
    current_state: FeagiEndpointState,
    #[allow(dead_code)]
    context: Context,
    socket: Socket,
    /// Reusable message for delimiter frame
    delimiter_msg: Message,
    /// Reusable message for payload frame
    payload_msg: Message,
    /// Whether payload_msg contains valid data
    has_data: bool,
}

impl FeagiZmqClientRequester {
    /// Receive a multipart response into reusable buffers.
    /// Response format: [empty delimiter, payload]
    /// Returns true if a complete message was received.
    fn try_recv_response(&mut self) -> Result<bool, zmq::Error> {
        // Receive first frame
        self.socket.recv(&mut self.delimiter_msg, zmq::DONTWAIT)?;

        // Check if there's a second frame
        if self.socket.get_rcvmore()? {
            // First frame was delimiter, receive payload
            self.socket.recv(&mut self.payload_msg, 0)?;
            
            // Drain any extra frames
            while self.socket.get_rcvmore()? {
                let mut discard = Message::new();
                self.socket.recv(&mut discard, 0)?;
            }
        } else {
            // Single frame - it's the payload (no delimiter from some servers)
            std::mem::swap(&mut self.delimiter_msg, &mut self.payload_msg);
        }

        Ok(true)
    }
}

impl FeagiClient for FeagiZmqClientRequester {
    fn poll(&mut self) -> &FeagiEndpointState {
        if matches!(self.current_state, FeagiEndpointState::ActiveWaiting) && !self.has_data {
            match self.try_recv_response() {
                Ok(true) => {
                    self.has_data = true;
                    self.current_state = FeagiEndpointState::ActiveHasData;
                }
                Ok(false) => {
                    // Incomplete message
                }
                Err(zmq::Error::EAGAIN) => {
                    // No response yet
                }
                Err(e) => {
                    self.current_state = FeagiEndpointState::Errored(
                        FeagiNetworkError::ReceiveFailed(e.to_string()),
                    );
                }
            }
        }
        &self.current_state
    }

    fn request_connect(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::Inactive => {
                self.socket
                    .connect(&self.server_address.to_string())
                    .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;

                self.current_state = FeagiEndpointState::ActiveWaiting;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot connect: client is not in Inactive state".to_string(),
            )),
        }
    }

    fn request_disconnect(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                self.socket
                    .disconnect(&self.server_address.to_string())
                    .map_err(|e| FeagiNetworkError::CannotDisconnect(e.to_string()))?;

                self.has_data = false;
                self.current_state = FeagiEndpointState::Inactive;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot disconnect: client is not in Active state".to_string(),
            )),
        }
    }

    fn confirm_error_and_close(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::Errored(_) => {
                let _ = self.socket.disconnect(&self.server_address.to_string());
                self.has_data = false;
                self.current_state = FeagiEndpointState::Inactive;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot confirm error: client is not in Errored state".to_string(),
            )),
        }
    }
}

impl FeagiClientRequester for FeagiZmqClientRequester {
    fn publish_request(&mut self, request: &[u8]) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                // DEALER sends: [empty delimiter, payload]
                let frames: &[&[u8]] = &[&[], request];
                self.socket
                    .send_multipart(frames, zmq::DONTWAIT)
                    .map_err(|e| {
                        if e == zmq::Error::EAGAIN {
                            FeagiNetworkError::SendFailed("Socket would block".to_string())
                        } else {
                            FeagiNetworkError::SendFailed(e.to_string())
                        }
                    })?;
                Ok(())
            }
            _ => Err(FeagiNetworkError::SendFailed(
                "Cannot send request: client is not in Active state".to_string(),
            )),
        }
    }

    fn consume_retrieved_response(&mut self) -> Result<&[u8], FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveHasData => {
                if self.has_data {
                    self.has_data = false;
                    self.current_state = FeagiEndpointState::ActiveWaiting;
                    Ok(&self.payload_msg)
                } else {
                    Err(FeagiNetworkError::ReceiveFailed(
                        "No data available despite ActiveHasData state".to_string(),
                    ))
                }
            }
            _ => Err(FeagiNetworkError::ReceiveFailed(
                "Cannot consume: no response available".to_string(),
            )),
        }
    }
}

//endregion
