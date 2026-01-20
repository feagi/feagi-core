//! WebSocket client implementations for FEAGI network traits.

use std::net::TcpStream;

use tungstenite::{connect, Message, WebSocket, stream::MaybeTlsStream};

use crate::next::FeagiNetworkError;
use crate::next::state_enums::FeagiClientConnectionState;
use crate::next::traits::client::{FeagiClient, FeagiClientSubscriber, FeagiClientPusher, FeagiClientRequester};

//region Subscriber

/// WebSocket client that subscribes to messages from a server.
pub struct FEAGIWebSocketClientSubscriber {
    server_address: String,
    current_state: FeagiClientConnectionState,
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    cached_data: Vec<u8>,
}

impl FEAGIWebSocketClientSubscriber {
    pub fn new(server_address: String) -> Result<Self, FeagiNetworkError> {
        Ok(Self {
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            socket: None,
            cached_data: Vec::new(),
        })
    }
}

impl FeagiClient for FEAGIWebSocketClientSubscriber {
    fn connect(&self, host: String) {
        // Note: This is a bit awkward with &self - in practice, use connect_mut
        // The trait signature may need adjustment for WebSocket
    }

    fn disconnect(&self) -> Result<(), FeagiNetworkError> {
        // Can't mutate with &self - see disconnect_mut below
        Ok(())
    }

    fn get_current_connection_state(&self) -> FeagiClientConnectionState {
        self.current_state
    }

    fn register_connection_state_changes<F>(&self, _on_state_change: F)
    where
        F: Fn((FeagiClientConnectionState, FeagiClientConnectionState)) + Send + Sync + 'static
    {
        // TODO: Implement state change notifications
    }
}

impl FEAGIWebSocketClientSubscriber {
    /// Connect to the WebSocket server (mutable version).
    pub fn connect_mut(&mut self) -> Result<(), FeagiNetworkError> {
        let url = if self.server_address.starts_with("ws://") || self.server_address.starts_with("wss://") {
            self.server_address.clone()
        } else {
            format!("ws://{}", self.server_address)
        };

        let (socket, _response) = connect(&url)
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;
        
        // Set non-blocking for polling
        if let MaybeTlsStream::Plain(ref stream) = socket.get_ref() {
            stream.set_nonblocking(true)
                .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        }
        
        self.socket = Some(socket);
        self.current_state = FeagiClientConnectionState::Connected;
        Ok(())
    }

    /// Disconnect from the server (mutable version).
    pub fn disconnect_mut(&mut self) -> Result<(), FeagiNetworkError> {
        if let Some(mut socket) = self.socket.take() {
            let _ = socket.close(None);
        }
        self.current_state = FeagiClientConnectionState::Disconnected;
        Ok(())
    }

    /// Non-blocking poll for incoming data.
    pub fn try_poll(&mut self) -> Result<bool, FeagiNetworkError> {
        let socket = match &mut self.socket {
            Some(s) => s,
            None => return Err(FeagiNetworkError::ReceiveFailed("Not connected".to_string())),
        };

        match socket.read() {
            Ok(Message::Binary(data)) => {
                self.cached_data = data;
                Ok(true)
            }
            Ok(Message::Text(text)) => {
                self.cached_data = text.into_bytes();
                Ok(true)
            }
            Ok(Message::Close(_)) => {
                self.current_state = FeagiClientConnectionState::Disconnected;
                Err(FeagiNetworkError::ReceiveFailed("Connection closed".to_string()))
            }
            Ok(_) => Ok(false), // Ping/Pong
            Err(tungstenite::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                Ok(false)
            }
            Err(e) => Err(FeagiNetworkError::ReceiveFailed(e.to_string())),
        }
    }

    /// Get the last received data.
    pub fn get_cached_data(&self) -> &[u8] {
        &self.cached_data
    }
}

impl FeagiClientSubscriber for FEAGIWebSocketClientSubscriber {
    // No additional trait methods - uses try_poll/get_cached_data above
}

//endregion

//region Pusher

/// WebSocket client that pushes data to a server.
pub struct FEAGIWebSocketClientPusher {
    server_address: String,
    current_state: FeagiClientConnectionState,
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
}

impl FEAGIWebSocketClientPusher {
    pub fn new(server_address: String) -> Result<Self, FeagiNetworkError> {
        Ok(Self {
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            socket: None,
        })
    }

    /// Connect to the WebSocket server.
    pub fn connect_mut(&mut self) -> Result<(), FeagiNetworkError> {
        let url = if self.server_address.starts_with("ws://") || self.server_address.starts_with("wss://") {
            self.server_address.clone()
        } else {
            format!("ws://{}", self.server_address)
        };

        let (socket, _response) = connect(&url)
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;
        
        self.socket = Some(socket);
        self.current_state = FeagiClientConnectionState::Connected;
        Ok(())
    }

    /// Disconnect from the server.
    pub fn disconnect_mut(&mut self) -> Result<(), FeagiNetworkError> {
        if let Some(mut socket) = self.socket.take() {
            let _ = socket.close(None);
        }
        self.current_state = FeagiClientConnectionState::Disconnected;
        Ok(())
    }
}

impl FeagiClient for FEAGIWebSocketClientPusher {
    fn connect(&self, _host: String) {
        // Use connect_mut instead
    }

    fn disconnect(&self) -> Result<(), FeagiNetworkError> {
        Ok(())
    }

    fn get_current_connection_state(&self) -> FeagiClientConnectionState {
        self.current_state
    }

    fn register_connection_state_changes<F>(&self, _on_state_change: F)
    where
        F: Fn((FeagiClientConnectionState, FeagiClientConnectionState)) + Send + Sync + 'static
    {
    }
}

impl FeagiClientPusher for FEAGIWebSocketClientPusher {
    fn push_data(&self, data: &[u8]) {
        // Note: Can't mutate with &self, this won't actually work
        // Need to use push_data_mut below
    }
}

impl FEAGIWebSocketClientPusher {
    /// Push data to the server (mutable version).
    pub fn push_data_mut(&mut self, data: &[u8]) -> Result<(), FeagiNetworkError> {
        let socket = match &mut self.socket {
            Some(s) => s,
            None => return Err(FeagiNetworkError::SendFailed("Not connected".to_string())),
        };

        let message = Message::Binary(data.to_vec());
        socket.send(message)
            .map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;
        
        Ok(())
    }
}

//endregion

//region Requester

/// WebSocket client that sends requests and receives responses.
pub struct FEAGIWebSocketClientRequester {
    server_address: String,
    current_state: FeagiClientConnectionState,
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    cached_response_data: Vec<u8>,
}

impl FEAGIWebSocketClientRequester {
    pub fn new(server_address: String) -> Result<Self, FeagiNetworkError> {
        Ok(Self {
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            socket: None,
            cached_response_data: Vec::new(),
        })
    }

    /// Connect to the WebSocket server.
    pub fn connect_mut(&mut self) -> Result<(), FeagiNetworkError> {
        let url = if self.server_address.starts_with("ws://") || self.server_address.starts_with("wss://") {
            self.server_address.clone()
        } else {
            format!("ws://{}", self.server_address)
        };

        let (socket, _response) = connect(&url)
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;
        
        // Set non-blocking for polling
        if let MaybeTlsStream::Plain(ref stream) = socket.get_ref() {
            stream.set_nonblocking(true)
                .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        }
        
        self.socket = Some(socket);
        self.current_state = FeagiClientConnectionState::Connected;
        Ok(())
    }

    /// Disconnect from the server.
    pub fn disconnect_mut(&mut self) -> Result<(), FeagiNetworkError> {
        if let Some(mut socket) = self.socket.take() {
            let _ = socket.close(None);
        }
        self.current_state = FeagiClientConnectionState::Disconnected;
        Ok(())
    }
}

impl FeagiClient for FEAGIWebSocketClientRequester {
    fn connect(&self, _host: String) {
        // Use connect_mut instead
    }

    fn disconnect(&self) -> Result<(), FeagiNetworkError> {
        Ok(())
    }

    fn get_current_connection_state(&self) -> FeagiClientConnectionState {
        self.current_state
    }

    fn register_connection_state_changes<F>(&self, _on_state_change: F)
    where
        F: Fn((FeagiClientConnectionState, FeagiClientConnectionState)) + Send + Sync + 'static
    {
    }
}

impl FeagiClientRequester for FEAGIWebSocketClientRequester {
    fn send_request(&self, _request: &[u8]) -> Result<(), FeagiNetworkError> {
        // Use send_request_mut below
        Err(FeagiNetworkError::SendFailed("Use send_request_mut instead".to_string()))
    }

    fn try_poll_response(&mut self) -> Result<bool, FeagiNetworkError> {
        let socket = match &mut self.socket {
            Some(s) => s,
            None => return Err(FeagiNetworkError::ReceiveFailed("Not connected".to_string())),
        };

        match socket.read() {
            Ok(Message::Binary(data)) => {
                self.cached_response_data = data;
                Ok(true)
            }
            Ok(Message::Text(text)) => {
                self.cached_response_data = text.into_bytes();
                Ok(true)
            }
            Ok(Message::Close(_)) => {
                self.current_state = FeagiClientConnectionState::Disconnected;
                Err(FeagiNetworkError::ReceiveFailed("Connection closed".to_string()))
            }
            Ok(_) => Ok(false),
            Err(tungstenite::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                Ok(false)
            }
            Err(e) => Err(FeagiNetworkError::ReceiveFailed(e.to_string())),
        }
    }

    fn get_response_data(&self) -> &[u8] {
        &self.cached_response_data
    }
}

impl FEAGIWebSocketClientRequester {
    /// Send a request to the server (mutable version).
    pub fn send_request_mut(&mut self, request: &[u8]) -> Result<(), FeagiNetworkError> {
        let socket = match &mut self.socket {
            Some(s) => s,
            None => return Err(FeagiNetworkError::SendFailed("Not connected".to_string())),
        };

        let message = Message::Binary(request.to_vec());
        socket.send(message)
            .map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;
        
        Ok(())
    }
}

//endregion
