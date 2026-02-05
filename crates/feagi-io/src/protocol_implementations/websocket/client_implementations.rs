//! WebSocket client implementations using the poll-based trait design.
//!
//! Uses `tungstenite` with non-blocking `std::net::TcpStream` for poll-based
//! WebSocket communication that works with any async runtime or synchronously.

use std::io::ErrorKind;
use std::net::TcpStream;

use tungstenite::{connect, Message, WebSocket};

use crate::FeagiNetworkError;
use crate::protocol_implementations::websocket::shared::{
    extract_host_port, normalize_ws_url,
};
use crate::traits_and_enums::FeagiEndpointState;
use crate::traits_and_enums::client::{
    FeagiClient, FeagiClientPusher, FeagiClientPusherProperties,
    FeagiClientRequester, FeagiClientRequesterProperties,
    FeagiClientSubscriber, FeagiClientSubscriberProperties,
};

/// Type alias for WebSocket over TcpStream
type WsStream = WebSocket<tungstenite::stream::MaybeTlsStream<TcpStream>>;

// ============================================================================
// Subscriber
// ============================================================================

//region Subscriber Properties

/// Configuration properties for creating a WebSocket subscriber client.
#[derive(Debug, Clone, PartialEq)]
pub struct FeagiWebSocketClientSubscriberProperties {
    server_address: String,
}

impl FeagiWebSocketClientSubscriberProperties {
    /// Creates new subscriber properties with the given server address.
    ///
    /// # Arguments
    ///
    /// * `server_address` - The WebSocket URL (e.g., "ws://localhost:8080" or "localhost:8080").
    pub fn new(server_address: &str) -> Result<Self, FeagiNetworkError> {
        let url = normalize_ws_url(server_address);
        let _ = extract_host_port(&url)?;
        Ok(Self {
            server_address: url,
        })
    }
}

impl FeagiClientSubscriberProperties for FeagiWebSocketClientSubscriberProperties {
    fn as_boxed_client_subscriber(&self) -> Box<dyn FeagiClientSubscriber> {
        Box::new(FeagiWebSocketClientSubscriber {
            server_address: self.server_address.clone(),
            current_state: FeagiEndpointState::Inactive,
            socket: None,
            receive_buffer: None,
            has_data: false,
        })
    }
}

//endregion

//region Subscriber Implementation

/// A WebSocket client that subscribes to data from a publisher server.
pub struct FeagiWebSocketClientSubscriber {
    server_address: String,
    current_state: FeagiEndpointState,
    socket: Option<WsStream>,
    receive_buffer: Option<Vec<u8>>,
    has_data: bool,
}

impl FeagiWebSocketClientSubscriber {
    fn try_receive(&mut self) -> bool {
        let socket = match &mut self.socket {
            Some(s) => s,
            None => return false,
        };

        match socket.read() {
            Ok(Message::Binary(data)) => {
                self.receive_buffer = Some(data);
                true
            }
            Ok(Message::Text(text)) => {
                self.receive_buffer = Some(text.into_bytes());
                true
            }
            Ok(Message::Close(_)) => {
                self.current_state = FeagiEndpointState::Errored(
                    FeagiNetworkError::ReceiveFailed("Connection closed".to_string()),
                );
                false
            }
            Ok(_) => false, // Ping/Pong
            Err(tungstenite::Error::Io(ref e)) if e.kind() == ErrorKind::WouldBlock => false,
            Err(e) => {
                self.current_state = FeagiEndpointState::Errored(
                    FeagiNetworkError::ReceiveFailed(e.to_string()),
                );
                false
            }
        }
    }
}

impl FeagiClient for FeagiWebSocketClientSubscriber {
    fn poll(&mut self) -> &FeagiEndpointState {
        if matches!(self.current_state, FeagiEndpointState::ActiveWaiting) && !self.has_data {
            if self.try_receive() {
                self.has_data = true;
                self.current_state = FeagiEndpointState::ActiveHasData;
            }
        }
        &self.current_state
    }

    fn request_connect(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::Inactive => {
                let (socket, _response) = connect(&self.server_address)
                    .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;

                // Set underlying stream to non-blocking
                if let tungstenite::stream::MaybeTlsStream::Plain(ref stream) = socket.get_ref() {
                    stream
                        .set_nonblocking(true)
                        .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;
                }

                self.socket = Some(socket);
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
                if let Some(mut socket) = self.socket.take() {
                    let _ = socket.close(None);
                }
                self.receive_buffer = None;
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
                if let Some(mut socket) = self.socket.take() {
                    let _ = socket.close(None);
                }
                self.receive_buffer = None;
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

impl FeagiClientSubscriber for FeagiWebSocketClientSubscriber {
    fn consume_retrieved_data(&mut self) -> Result<&[u8], FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveHasData => {
                if self.has_data {
                    if let Some(ref data) = self.receive_buffer {
                        self.has_data = false;
                        self.current_state = FeagiEndpointState::ActiveWaiting;
                        Ok(data.as_slice())
                    } else {
                        Err(FeagiNetworkError::ReceiveFailed(
                            "No data in buffer".to_string(),
                        ))
                    }
                } else {
                    Err(FeagiNetworkError::ReceiveFailed(
                        "No data available".to_string(),
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

/// Configuration properties for creating a WebSocket pusher client.
#[derive(Debug, Clone, PartialEq)]
pub struct FeagiWebSocketClientPusherProperties {
    server_address: String,
}

impl FeagiWebSocketClientPusherProperties {
    /// Creates new pusher properties with the given server address.
    pub fn new(server_address: &str) -> Result<Self, FeagiNetworkError> {
        let url = normalize_ws_url(server_address);
        let _ = extract_host_port(&url)?;
        Ok(Self {
            server_address: url,
        })
    }
}

impl FeagiClientPusherProperties for FeagiWebSocketClientPusherProperties {
    fn as_boxed_client_pusher(&self) -> Box<dyn FeagiClientPusher> {
        Box::new(FeagiWebSocketClientPusher {
            server_address: self.server_address.clone(),
            current_state: FeagiEndpointState::Inactive,
            socket: None,
        })
    }
}

//endregion

//region Pusher Implementation

/// A WebSocket client that pushes data to a server.
pub struct FeagiWebSocketClientPusher {
    server_address: String,
    current_state: FeagiEndpointState,
    socket: Option<WsStream>,
}

impl FeagiClient for FeagiWebSocketClientPusher {
    fn poll(&mut self) -> &FeagiEndpointState {
        // Pusher doesn't receive data, just return current state
        &self.current_state
    }

    fn request_connect(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::Inactive => {
                let (socket, _response) = connect(&self.server_address)
                    .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;

                if let tungstenite::stream::MaybeTlsStream::Plain(ref stream) = socket.get_ref() {
                    stream
                        .set_nonblocking(true)
                        .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;
                }

                self.socket = Some(socket);
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
                if let Some(mut socket) = self.socket.take() {
                    let _ = socket.close(None);
                }
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
                if let Some(mut socket) = self.socket.take() {
                    let _ = socket.close(None);
                }
                self.current_state = FeagiEndpointState::Inactive;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot confirm error: client is not in Errored state".to_string(),
            )),
        }
    }
}

impl FeagiClientPusher for FeagiWebSocketClientPusher {
    fn publish_data(&mut self, data: &[u8]) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                let socket = self
                    .socket
                    .as_mut()
                    .ok_or_else(|| FeagiNetworkError::SendFailed("Not connected".to_string()))?;

                let message = Message::Binary(data.to_vec());
                socket
                    .send(message)
                    .map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;
                socket
                    .flush()
                    .map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;

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
// Requester
// ============================================================================

//region Requester Properties

/// Configuration properties for creating a WebSocket requester client.
#[derive(Debug, Clone, PartialEq)]
pub struct FeagiWebSocketClientRequesterProperties {
    server_address: String,
}

impl FeagiWebSocketClientRequesterProperties {
    /// Creates new requester properties with the given server address.
    pub fn new(server_address: &str) -> Result<Self, FeagiNetworkError> {
        let url = normalize_ws_url(server_address);
        let _ = extract_host_port(&url)?;
        Ok(Self {
            server_address: url,
        })
    }
}

impl FeagiClientRequesterProperties for FeagiWebSocketClientRequesterProperties {
    fn as_boxed_client_requester(&self) -> Box<dyn FeagiClientRequester> {
        Box::new(FeagiWebSocketClientRequester {
            server_address: self.server_address.clone(),
            current_state: FeagiEndpointState::Inactive,
            socket: None,
            receive_buffer: None,
            has_data: false,
        })
    }
}

//endregion

//region Requester Implementation

/// A WebSocket client that sends requests and receives responses.
pub struct FeagiWebSocketClientRequester {
    server_address: String,
    current_state: FeagiEndpointState,
    socket: Option<WsStream>,
    receive_buffer: Option<Vec<u8>>,
    has_data: bool,
}

impl FeagiWebSocketClientRequester {
    fn try_receive(&mut self) -> bool {
        let socket = match &mut self.socket {
            Some(s) => s,
            None => return false,
        };

        match socket.read() {
            Ok(Message::Binary(data)) => {
                self.receive_buffer = Some(data);
                true
            }
            Ok(Message::Text(text)) => {
                self.receive_buffer = Some(text.into_bytes());
                true
            }
            Ok(Message::Close(_)) => {
                self.current_state = FeagiEndpointState::Errored(
                    FeagiNetworkError::ReceiveFailed("Connection closed".to_string()),
                );
                false
            }
            Ok(_) => false,
            Err(tungstenite::Error::Io(ref e)) if e.kind() == ErrorKind::WouldBlock => false,
            Err(e) => {
                self.current_state = FeagiEndpointState::Errored(
                    FeagiNetworkError::ReceiveFailed(e.to_string()),
                );
                false
            }
        }
    }
}

impl FeagiClient for FeagiWebSocketClientRequester {
    fn poll(&mut self) -> &FeagiEndpointState {
        if matches!(self.current_state, FeagiEndpointState::ActiveWaiting) && !self.has_data {
            if self.try_receive() {
                self.has_data = true;
                self.current_state = FeagiEndpointState::ActiveHasData;
            }
        }
        &self.current_state
    }

    fn request_connect(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::Inactive => {
                let (socket, _response) = connect(&self.server_address)
                    .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;

                if let tungstenite::stream::MaybeTlsStream::Plain(ref stream) = socket.get_ref() {
                    stream
                        .set_nonblocking(true)
                        .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;
                }

                self.socket = Some(socket);
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
                if let Some(mut socket) = self.socket.take() {
                    let _ = socket.close(None);
                }
                self.receive_buffer = None;
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
                if let Some(mut socket) = self.socket.take() {
                    let _ = socket.close(None);
                }
                self.receive_buffer = None;
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

impl FeagiClientRequester for FeagiWebSocketClientRequester {
    fn publish_request(&mut self, request: &[u8]) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                let socket = self
                    .socket
                    .as_mut()
                    .ok_or_else(|| FeagiNetworkError::SendFailed("Not connected".to_string()))?;

                let message = Message::Binary(request.to_vec());
                socket
                    .send(message)
                    .map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;
                socket
                    .flush()
                    .map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;

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
                    if let Some(ref data) = self.receive_buffer {
                        self.has_data = false;
                        self.current_state = FeagiEndpointState::ActiveWaiting;
                        Ok(data.as_slice())
                    } else {
                        Err(FeagiNetworkError::ReceiveFailed(
                            "No data in buffer".to_string(),
                        ))
                    }
                } else {
                    Err(FeagiNetworkError::ReceiveFailed(
                        "No response available".to_string(),
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
