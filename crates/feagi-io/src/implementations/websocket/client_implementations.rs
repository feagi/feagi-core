//! WebSocket client implementations for FEAGI network traits.
//!
//! Uses `async-tungstenite` with `async-net` for runtime-agnostic async WebSocket communication.

use async_net::TcpStream;
use async_trait::async_trait;
use async_tungstenite::tungstenite::Message;
use async_tungstenite::{client_async, WebSocketStream};
use futures_util::{SinkExt, StreamExt};

use super::shared_functions::{extract_host_port, normalize_ws_url};
use crate::traits_and_enums::client::client_shared::{
    FeagiClientConnectionState, FeagiClientConnectionStateChange,
};
use crate::traits_and_enums::client::{
    FeagiClient, FeagiClientPusher, FeagiClientRequester, FeagiClientSubscriber,
};
use crate::FeagiNetworkError;

/// Type alias for the client state change callback.
type StateChangeCallback = Box<dyn Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static>;

/// Type alias for the WebSocket stream using async-net's TcpStream.
type WsStream = WebSocketStream<TcpStream>;

//region Subscriber

/// WebSocket client that subscribes to messages from a server.
pub struct FEAGIWebSocketClientSubscriber {
    server_address: String,
    current_state: FeagiClientConnectionState,
    state_change_callback: StateChangeCallback,
    socket: Option<WsStream>,
}

impl FEAGIWebSocketClientSubscriber {
    pub fn new(
        server_address: String,
        state_change_callback: StateChangeCallback,
    ) -> Result<Self, FeagiNetworkError> {
        Ok(Self {
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            state_change_callback,
            socket: None,
        })
    }
}

#[async_trait]
impl FeagiClient for FEAGIWebSocketClientSubscriber {
    async fn connect(&mut self, host: &str) -> Result<(), FeagiNetworkError> {
        let url = normalize_ws_url(host);
        let host_port = extract_host_port(&url)?;

        // Connect TCP stream using async-net (runtime-agnostic)
        let tcp_stream = TcpStream::connect(&host_port)
            .await
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;

        // Perform WebSocket handshake
        let (ws_stream, _response) = client_async(&url, tcp_stream)
            .await
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;

        self.socket = Some(ws_stream);
        self.server_address = host.to_string();
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Connected;
        (self.state_change_callback)(FeagiClientConnectionStateChange::new(
            previous,
            self.current_state,
        ));
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), FeagiNetworkError> {
        if let Some(mut socket) = self.socket.take() {
            let _ = socket.close(None).await;
        }
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Disconnected;
        (self.state_change_callback)(FeagiClientConnectionStateChange::new(
            previous,
            self.current_state,
        ));
        Ok(())
    }

    fn get_current_connection_state(&self) -> FeagiClientConnectionState {
        self.current_state
    }
}

#[async_trait]
impl FeagiClientSubscriber for FEAGIWebSocketClientSubscriber {
    async fn get_subscribed_data(&mut self) -> Result<Vec<u8>, FeagiNetworkError> {
        let socket = self
            .socket
            .as_mut()
            .ok_or_else(|| FeagiNetworkError::ReceiveFailed("Not connected".to_string()))?;

        loop {
            match socket.next().await {
                Some(Ok(Message::Binary(data))) => {
                    return Ok(data);
                }
                Some(Ok(Message::Text(text))) => {
                    return Ok(text.into_bytes());
                }
                Some(Ok(Message::Close(_))) => {
                    let previous = self.current_state;
                    self.current_state = FeagiClientConnectionState::Disconnected;
                    (self.state_change_callback)(FeagiClientConnectionStateChange::new(
                        previous,
                        self.current_state,
                    ));
                    return Err(FeagiNetworkError::ReceiveFailed(
                        "Connection closed".to_string(),
                    ));
                }
                Some(Ok(_)) => {
                    // Ping/Pong/Frame - continue waiting for data
                    continue;
                }
                Some(Err(e)) => {
                    return Err(FeagiNetworkError::ReceiveFailed(e.to_string()));
                }
                None => {
                    let previous = self.current_state;
                    self.current_state = FeagiClientConnectionState::Disconnected;
                    (self.state_change_callback)(FeagiClientConnectionStateChange::new(
                        previous,
                        self.current_state,
                    ));
                    return Err(FeagiNetworkError::ReceiveFailed(
                        "Connection closed".to_string(),
                    ));
                }
            }
        }
    }
}

//endregion

//region Pusher

/// WebSocket client that pushes data to a server.
pub struct FEAGIWebSocketClientPusher {
    server_address: String,
    current_state: FeagiClientConnectionState,
    state_change_callback: StateChangeCallback,
    socket: Option<WsStream>,
}

impl FEAGIWebSocketClientPusher {
    pub fn new(
        server_address: String,
        state_change_callback: StateChangeCallback,
    ) -> Result<Self, FeagiNetworkError> {
        Ok(Self {
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            state_change_callback,
            socket: None,
        })
    }
}

#[async_trait]
impl FeagiClient for FEAGIWebSocketClientPusher {
    async fn connect(&mut self, host: &str) -> Result<(), FeagiNetworkError> {
        let url = normalize_ws_url(host);
        let host_port = extract_host_port(&url)?;

        let tcp_stream = TcpStream::connect(&host_port)
            .await
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;

        let (ws_stream, _response) = client_async(&url, tcp_stream)
            .await
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;

        self.socket = Some(ws_stream);
        self.server_address = host.to_string();
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Connected;
        (self.state_change_callback)(FeagiClientConnectionStateChange::new(
            previous,
            self.current_state,
        ));
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), FeagiNetworkError> {
        if let Some(mut socket) = self.socket.take() {
            let _ = socket.close(None).await;
        }
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Disconnected;
        (self.state_change_callback)(FeagiClientConnectionStateChange::new(
            previous,
            self.current_state,
        ));
        Ok(())
    }

    fn get_current_connection_state(&self) -> FeagiClientConnectionState {
        self.current_state
    }
}

#[async_trait]
impl FeagiClientPusher for FEAGIWebSocketClientPusher {
    async fn push_data(&mut self, data: &[u8]) -> Result<(), FeagiNetworkError> {
        let socket = self
            .socket
            .as_mut()
            .ok_or_else(|| FeagiNetworkError::SendFailed("Not connected".to_string()))?;

        let message = Message::Binary(data.to_vec());
        socket
            .send(message)
            .await
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
    socket: Option<WsStream>,
}

impl FEAGIWebSocketClientRequester {
    pub fn new(
        server_address: String,
        state_change_callback: StateChangeCallback,
    ) -> Result<Self, FeagiNetworkError> {
        Ok(Self {
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            state_change_callback,
            socket: None,
        })
    }
}

#[async_trait]
impl FeagiClient for FEAGIWebSocketClientRequester {
    async fn connect(&mut self, host: &str) -> Result<(), FeagiNetworkError> {
        let url = normalize_ws_url(host);
        let host_port = extract_host_port(&url)?;

        let tcp_stream = TcpStream::connect(&host_port)
            .await
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;

        let (ws_stream, _response) = client_async(&url, tcp_stream)
            .await
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;

        self.socket = Some(ws_stream);
        self.server_address = host.to_string();
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Connected;
        (self.state_change_callback)(FeagiClientConnectionStateChange::new(
            previous,
            self.current_state,
        ));
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), FeagiNetworkError> {
        if let Some(mut socket) = self.socket.take() {
            let _ = socket.close(None).await;
        }
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Disconnected;
        (self.state_change_callback)(FeagiClientConnectionStateChange::new(
            previous,
            self.current_state,
        ));
        Ok(())
    }

    fn get_current_connection_state(&self) -> FeagiClientConnectionState {
        self.current_state
    }
}

#[async_trait]
impl FeagiClientRequester for FEAGIWebSocketClientRequester {
    async fn send_request(&mut self, request: &[u8]) -> Result<(), FeagiNetworkError> {
        let socket = self
            .socket
            .as_mut()
            .ok_or_else(|| FeagiNetworkError::SendFailed("Not connected".to_string()))?;

        let message = Message::Binary(request.to_vec());
        socket
            .send(message)
            .await
            .map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;

        Ok(())
    }

    async fn get_response(&mut self) -> Result<Vec<u8>, FeagiNetworkError> {
        let socket = self
            .socket
            .as_mut()
            .ok_or_else(|| FeagiNetworkError::ReceiveFailed("Not connected".to_string()))?;

        loop {
            match socket.next().await {
                Some(Ok(Message::Binary(data))) => {
                    return Ok(data);
                }
                Some(Ok(Message::Text(text))) => {
                    return Ok(text.into_bytes());
                }
                Some(Ok(Message::Close(_))) => {
                    let previous = self.current_state;
                    self.current_state = FeagiClientConnectionState::Disconnected;
                    (self.state_change_callback)(FeagiClientConnectionStateChange::new(
                        previous,
                        self.current_state,
                    ));
                    return Err(FeagiNetworkError::ReceiveFailed(
                        "Connection closed".to_string(),
                    ));
                }
                Some(Ok(_)) => {
                    // Ping/Pong - continue waiting
                    continue;
                }
                Some(Err(e)) => {
                    return Err(FeagiNetworkError::ReceiveFailed(e.to_string()));
                }
                None => {
                    let previous = self.current_state;
                    self.current_state = FeagiClientConnectionState::Disconnected;
                    (self.state_change_callback)(FeagiClientConnectionStateChange::new(
                        previous,
                        self.current_state,
                    ));
                    return Err(FeagiNetworkError::ReceiveFailed(
                        "Connection closed".to_string(),
                    ));
                }
            }
        }
    }
}

//endregion
