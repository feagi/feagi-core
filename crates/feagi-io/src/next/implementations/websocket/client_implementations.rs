//! WebSocket client implementations for FEAGI network traits.

use std::net::TcpStream;

use tungstenite::{connect, Message, WebSocket, stream::MaybeTlsStream};

use crate::next::{FeagiClientConnectionState, FeagiNetworkError};
use crate::next::traits_and_enums::client::client_shared::FeagiClientConnectionStateChange;
use crate::next::traits_and_enums::client::{
    FeagiClient, FeagiClientSubscriber, FeagiClientPusher, FeagiClientRequester,
    FeagiClientSubscriberProperties, FeagiClientPusherProperties, FeagiClientRequesterProperties
};

//region Subscriber

/// WebSocket client that subscribes to messages from a server.
pub struct FEAGIWebSocketClientSubscriber<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    server_address: String,
    current_state: FeagiClientConnectionState,
    state_change_callback: S,
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    cached_data: Vec<u8>,
}

impl<S> FEAGIWebSocketClientSubscriber<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    pub fn new(server_address: String, state_change_callback: S) -> Result<Self, FeagiNetworkError> {
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

impl<S> FeagiClient for FEAGIWebSocketClientSubscriber<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
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

impl<S> FeagiClientSubscriber for FEAGIWebSocketClientSubscriber<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    // Polling method is on the impl block directly since trait is empty
}

//endregion

//region Pusher

/// WebSocket client that pushes data to a server.
pub struct FEAGIWebSocketClientPusher<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    server_address: String,
    current_state: FeagiClientConnectionState,
    state_change_callback: S,
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
}

impl<S> FEAGIWebSocketClientPusher<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    pub fn new(server_address: String, state_change_callback: S) -> Result<Self, FeagiNetworkError> {
        Ok(Self {
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            state_change_callback,
            socket: None,
        })
    }
}

impl<S> FeagiClient for FEAGIWebSocketClientPusher<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
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

impl<S> FeagiClientPusher for FEAGIWebSocketClientPusher<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    fn push_data(&self, _data: &[u8]) {
        // Note: Can't mutate with &self - this is a limitation of the trait
        // Use push_data_mut below for actual functionality
    }
}

impl<S> FEAGIWebSocketClientPusher<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
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
pub struct FEAGIWebSocketClientRequester<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    server_address: String,
    current_state: FeagiClientConnectionState,
    state_change_callback: S,
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
    cached_response_data: Vec<u8>,
}

impl<S> FEAGIWebSocketClientRequester<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    pub fn new(server_address: String, state_change_callback: S) -> Result<Self, FeagiNetworkError> {
        Ok(Self {
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            state_change_callback,
            socket: None,
            cached_response_data: Vec::new(),
        })
    }
}

impl<S> FeagiClient for FEAGIWebSocketClientRequester<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
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

impl<S> FeagiClientRequester for FEAGIWebSocketClientRequester<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
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

impl<S> FEAGIWebSocketClientRequester<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
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
    fn build<F>(self, state_change_callback: F) -> Box<dyn FeagiClientSubscriber>
    where F: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
    {
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
    fn build<F>(self, state_change_callback: F) -> Box<dyn FeagiClientPusher>
    where F: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
    {
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
    fn build<F>(self, state_change_callback: F) -> Box<dyn FeagiClientRequester>
    where F: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
    {
        let requester = FEAGIWebSocketClientRequester::new(
            self.server_address,
            state_change_callback,
        ).expect("Failed to create WebSocket requester");
        
        Box::new(requester)
    }
}

//endregion

//endregion
