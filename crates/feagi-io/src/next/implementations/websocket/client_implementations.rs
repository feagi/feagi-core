//! WebSocket client implementations for FEAGI network traits.

use std::net::TcpStream;

use tokio_tungstenite::tungstenite::{self, connect, Message, WebSocket, stream::MaybeTlsStream};

use crate::next::{FeagiClientConnectionState, FeagiNetworkError};
use crate::next::traits_and_enums::client::client_shared::FeagiClientConnectionStateChange;
use crate::next::traits_and_enums::client::{
    FeagiClient, FeagiClientSubscriber, FeagiClientPusher, FeagiClientRequester,
    FeagiClientSubscriberProperties, FeagiClientPusherProperties, FeagiClientRequesterProperties
};

/// Type alias for the client state change callback.
type StateChangeCallback = Box<dyn Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static>;

//region Subscriber

/// WebSocket client that subscribes to messages from a server.
pub struct FEAGIWebSocketClientSubscriber {
    server_address: String,
    current_state: FeagiClientConnectionState,
    state_change_callback: StateChangeCallback,
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    cached_data: Vec<u8>,
}

impl FEAGIWebSocketClientSubscriber {
    pub fn new(server_address: String, state_change_callback: StateChangeCallback) -> Result<Self, FeagiNetworkError> {
        Ok(Self {
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            state_change_callback,
            socket: None,
            cached_data: Vec::new(),
        })
    }

    /// Non-blocking poll for incoming data.
    pub fn try_poll_receive(&mut self) -> Result<Option<&[u8]>, FeagiNetworkError> {
        let socket = match &mut self.socket {
            Some(s) => s,
            None => return Err(FeagiNetworkError::ReceiveFailed("Not connected".to_string())),
        };

        match socket.read() {
            Ok(Message::Binary(data)) => {
                self.cached_data = data;
                Ok(Some(&self.cached_data))
            }
            Ok(Message::Text(text)) => {
                self.cached_data = text.into_bytes();
                Ok(Some(&self.cached_data))
            }
            Ok(Message::Close(_)) => {
                let previous = self.current_state;
                self.current_state = FeagiClientConnectionState::Disconnected;
                (self.state_change_callback)(
                    FeagiClientConnectionStateChange::new(previous, self.current_state)
                );
                Err(FeagiNetworkError::ReceiveFailed("Connection closed".to_string()))
            }
            Ok(_) => Ok(None), // Ping/Pong
            Err(tungstenite::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                Ok(None)
            }
            Err(e) => Err(FeagiNetworkError::ReceiveFailed(e.to_string())),
        }
    }
}

impl FeagiClient for FEAGIWebSocketClientSubscriber {
    fn connect(&mut self, host: &str) -> Result<(), FeagiNetworkError> {
        let url = if host.starts_with("ws://") || host.starts_with("wss://") {
            host.to_string()
        } else {
            format!("ws://{}", host)
        };

        let (socket, _response) = connect(&url)
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;
        
        // Set non-blocking for polling
        if let MaybeTlsStream::Plain(ref stream) = socket.get_ref() {
            stream.set_nonblocking(true)
                .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        }
        
        self.socket = Some(socket);
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Connected;
        (self.state_change_callback)(
            FeagiClientConnectionStateChange::new(previous, self.current_state)
        );
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), FeagiNetworkError> {
        if let Some(mut socket) = self.socket.take() {
            let _ = socket.close(None);
        }
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Disconnected;
        (self.state_change_callback)(
            FeagiClientConnectionStateChange::new(previous, self.current_state)
        );
        Ok(())
    }

    fn get_current_connection_state(&self) -> FeagiClientConnectionState {
        self.current_state
    }
}

impl FeagiClientSubscriber for FEAGIWebSocketClientSubscriber {
    // Polling method is on the impl block directly since trait is empty
}

//endregion

//region Pusher

/// WebSocket client that pushes data to a server.
pub struct FEAGIWebSocketClientPusher {
    server_address: String,
    current_state: FeagiClientConnectionState,
    state_change_callback: StateChangeCallback,
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
}

impl FEAGIWebSocketClientPusher {
    pub fn new(server_address: String, state_change_callback: StateChangeCallback) -> Result<Self, FeagiNetworkError> {
        Ok(Self {
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            state_change_callback,
            socket: None,
        })
    }
}

impl FeagiClient for FEAGIWebSocketClientPusher {
    fn connect(&mut self, host: &str) -> Result<(), FeagiNetworkError> {
        let url = if host.starts_with("ws://") || host.starts_with("wss://") {
            host.to_string()
        } else {
            format!("ws://{}", host)
        };

        let (socket, _response) = connect(&url)
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;
        
        self.socket = Some(socket);
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Connected;
        (self.state_change_callback)(
            FeagiClientConnectionStateChange::new(previous, self.current_state)
        );
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), FeagiNetworkError> {
        if let Some(mut socket) = self.socket.take() {
            let _ = socket.close(None);
        }
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Disconnected;
        (self.state_change_callback)(
            FeagiClientConnectionStateChange::new(previous, self.current_state)
        );
        Ok(())
    }

    fn get_current_connection_state(&self) -> FeagiClientConnectionState {
        self.current_state
    }
}

impl FeagiClientPusher for FEAGIWebSocketClientPusher {
    fn push_data(&self, _data: &[u8]) {
        // Note: Can't mutate with &self - this is a limitation of the trait
        // Use push_data_mut below for actual functionality
    }
}

impl FEAGIWebSocketClientPusher {
    /// Push data to the server (mutable version needed for WebSocket).
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
    state_change_callback: StateChangeCallback,
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    cached_response_data: Vec<u8>,
}

impl FEAGIWebSocketClientRequester {
    pub fn new(server_address: String, state_change_callback: StateChangeCallback) -> Result<Self, FeagiNetworkError> {
        Ok(Self {
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            state_change_callback,
            socket: None,
            cached_response_data: Vec::new(),
        })
    }
}

impl FeagiClient for FEAGIWebSocketClientRequester {
    fn connect(&mut self, host: &str) -> Result<(), FeagiNetworkError> {
        let url = if host.starts_with("ws://") || host.starts_with("wss://") {
            host.to_string()
        } else {
            format!("ws://{}", host)
        };

        let (socket, _response) = connect(&url)
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;
        
        // Set non-blocking for polling
        if let MaybeTlsStream::Plain(ref stream) = socket.get_ref() {
            stream.set_nonblocking(true)
                .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        }
        
        self.socket = Some(socket);
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Connected;
        (self.state_change_callback)(
            FeagiClientConnectionStateChange::new(previous, self.current_state)
        );
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), FeagiNetworkError> {
        if let Some(mut socket) = self.socket.take() {
            let _ = socket.close(None);
        }
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Disconnected;
        (self.state_change_callback)(
            FeagiClientConnectionStateChange::new(previous, self.current_state)
        );
        Ok(())
    }

    fn get_current_connection_state(&self) -> FeagiClientConnectionState {
        self.current_state
    }
}

impl FeagiClientRequester for FEAGIWebSocketClientRequester {
    fn send_request(&self, _request: &[u8]) -> Result<(), FeagiNetworkError> {
        // Note: Can't mutate with &self - use send_request_mut below
        Err(FeagiNetworkError::SendFailed("Use send_request_mut instead for WebSocket".to_string()))
    }

    fn try_poll_receive(&mut self) -> Result<Option<&[u8]>, FeagiNetworkError> {
        let socket = match &mut self.socket {
            Some(s) => s,
            None => return Err(FeagiNetworkError::ReceiveFailed("Not connected".to_string())),
        };

        match socket.read() {
            Ok(Message::Binary(data)) => {
                self.cached_response_data = data;
                Ok(Some(&self.cached_response_data))
            }
            Ok(Message::Text(text)) => {
                self.cached_response_data = text.into_bytes();
                Ok(Some(&self.cached_response_data))
            }
            Ok(Message::Close(_)) => {
                let previous = self.current_state;
                self.current_state = FeagiClientConnectionState::Disconnected;
                (self.state_change_callback)(
                    FeagiClientConnectionStateChange::new(previous, self.current_state)
                );
                Err(FeagiNetworkError::ReceiveFailed("Connection closed".to_string()))
            }
            Ok(_) => Ok(None),
            Err(tungstenite::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                Ok(None)
            }
            Err(e) => Err(FeagiNetworkError::ReceiveFailed(e.to_string())),
        }
    }
}

impl FEAGIWebSocketClientRequester {
    /// Send a request to the server (mutable version needed for WebSocket).
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

//region Properties

//region Subscriber Properties

/// Properties for configuring and building a WebSocket Client Subscriber.
pub struct FEAGIWebSocketClientSubscriberProperties {
    server_address: String,
}

impl FEAGIWebSocketClientSubscriberProperties {
    /// Create new properties with the given server address.
    pub fn new(server_address: String) -> Self {
        Self {
            server_address,
        }
    }
}

impl FeagiClientSubscriberProperties for FEAGIWebSocketClientSubscriberProperties {
    fn build(self: Box<Self>, state_change_callback: StateChangeCallback) -> Box<dyn FeagiClientSubscriber> {
        let subscriber = FEAGIWebSocketClientSubscriber::new(
            self.server_address,
            state_change_callback,
        ).expect("Failed to create WebSocket subscriber");
        
        Box::new(subscriber)
    }
}

//endregion

//region Pusher Properties

/// Properties for configuring and building a WebSocket Client Pusher.
pub struct FEAGIWebSocketClientPusherProperties {
    server_address: String,
}

impl FEAGIWebSocketClientPusherProperties {
    /// Create new properties with the given server address.
    pub fn new(server_address: String) -> Self {
        Self {
            server_address,
        }
    }
}

impl FeagiClientPusherProperties for FEAGIWebSocketClientPusherProperties {
    fn build(self: Box<Self>, state_change_callback: StateChangeCallback) -> Box<dyn FeagiClientPusher> {
        let pusher = FEAGIWebSocketClientPusher::new(
            self.server_address,
            state_change_callback,
        ).expect("Failed to create WebSocket pusher");
        
        Box::new(pusher)
    }
}

//endregion

//region Requester Properties

/// Properties for configuring and building a WebSocket Client Requester.
pub struct FEAGIWebSocketClientRequesterProperties {
    server_address: String,
}

impl FEAGIWebSocketClientRequesterProperties {
    /// Create new properties with the given server address.
    pub fn new(server_address: String) -> Self {
        Self {
            server_address,
        }
    }
}

impl FeagiClientRequesterProperties for FEAGIWebSocketClientRequesterProperties {
    fn build(self: Box<Self>, state_change_callback: StateChangeCallback) -> Box<dyn FeagiClientRequester> {
        let requester = FEAGIWebSocketClientRequester::new(
            self.server_address,
            state_change_callback,
        ).expect("Failed to create WebSocket requester");
        
        Box::new(requester)
    }
}

//endregion

//endregion
