//! WebSocket server implementations for FEAGI network traits.
//!
//! Uses `async-tungstenite` with tokio for async WebSocket communication.

use std::collections::HashMap;

use async_tungstenite::tokio::accept_async;
use async_tungstenite::tungstenite::Message;
use async_tungstenite::WebSocketStream;
use feagi_serialization::SessionID;
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};

use crate::traits_and_enums::server::server_shared::{
    FeagiServerBindState, FeagiServerBindStateChange,
};
use crate::traits_and_enums::server::{
    FeagiServer, FeagiServerPublisher, FeagiServerPuller, FeagiServerRouter,
};
use crate::FeagiNetworkError;

/// Type alias for the server state change callback.
type StateChangeCallback = Box<dyn Fn(FeagiServerBindStateChange) + Send + Sync + 'static>;

//region Publisher

/// WebSocket server that broadcasts messages to all connected clients.
pub struct FEAGIWebSocketServerPublisher {
    bind_address: String,
    current_state: FeagiServerBindState,
    state_change_callback: StateChangeCallback,
    listener: Option<TcpListener>,
    clients: Vec<WebSocketStream<TcpStream>>,
}

impl FEAGIWebSocketServerPublisher {
    pub fn new(
        bind_address: String,
        state_change_callback: StateChangeCallback,
    ) -> Result<Self, FeagiNetworkError> {
        Ok(Self {
            bind_address,
            current_state: FeagiServerBindState::Inactive,
            state_change_callback,
            listener: None,
            clients: Vec::new(),
        })
    }

    /// Get the number of connected clients.
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Accept any pending connections (non-blocking via try_accept).
    async fn accept_pending_connections(&mut self) -> Result<usize, FeagiNetworkError> {
        let listener = match &self.listener {
            Some(l) => l,
            None => return Ok(0),
        };

        let mut accepted = 0;
        loop {
            match listener.try_accept() {
                Ok((stream, _addr)) => match accept_async(stream).await {
                    Ok(ws) => {
                        self.clients.push(ws);
                        accepted += 1;
                    }
                    Err(e) => {
                        eprintln!("WebSocket handshake failed: {}", e);
                    }
                },
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(_) => {
                    break;
                }
            }
        }
        Ok(accepted)
    }
}

impl FeagiServer for FEAGIWebSocketServerPublisher {
    async fn start(&mut self) -> Result<(), FeagiNetworkError> {
        let listener = TcpListener::bind(&self.bind_address)
            .await
            .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;

        self.listener = Some(listener);
        let previous = self.current_state;
        self.current_state = FeagiServerBindState::Active;
        (self.state_change_callback)(FeagiServerBindStateChange::new(
            previous,
            FeagiServerBindState::Active,
        ));
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        // Close all client connections
        for mut client in self.clients.drain(..) {
            let _ = client.close(None).await;
        }
        self.listener = None;
        let previous = self.current_state;
        self.current_state = FeagiServerBindState::Inactive;
        (self.state_change_callback)(FeagiServerBindStateChange::new(
            previous,
            FeagiServerBindState::Inactive,
        ));
        Ok(())
    }

    fn get_current_state(&self) -> FeagiServerBindState {
        self.current_state
    }
}

impl FeagiServerPublisher for FEAGIWebSocketServerPublisher {
    async fn poll(&mut self) -> Result<(), FeagiNetworkError> {
        self.accept_pending_connections().await?;
        Ok(())
    }

    async fn publish(&mut self, buffered_data_to_send: &[u8]) -> Result<(), FeagiNetworkError> {
        let message = Message::Binary(buffered_data_to_send.to_vec());

        // Send to all clients, tracking which ones fail
        let mut failed_indices = Vec::new();
        for (i, client) in self.clients.iter_mut().enumerate() {
            if client.send(message.clone()).await.is_err() {
                failed_indices.push(i);
            }
        }

        // Remove failed clients (in reverse order to preserve indices)
        for i in failed_indices.into_iter().rev() {
            self.clients.remove(i);
        }

        Ok(())
    }
}

//endregion

//region Puller

/// WebSocket server that receives pushed data from clients.
pub struct FEAGIWebSocketServerPuller {
    bind_address: String,
    current_state: FeagiServerBindState,
    state_change_callback: StateChangeCallback,
    listener: Option<TcpListener>,
    clients: Vec<WebSocketStream<TcpStream>>,
    cached_data: Vec<u8>,
    /// Index of the client we're currently reading from (round-robin style)
    current_client_index: usize,
}

impl FEAGIWebSocketServerPuller {
    pub fn new(
        bind_address: String,
        state_change_callback: StateChangeCallback,
    ) -> Result<Self, FeagiNetworkError> {
        Ok(Self {
            bind_address,
            current_state: FeagiServerBindState::Inactive,
            state_change_callback,
            listener: None,
            clients: Vec::new(),
            cached_data: Vec::new(),
            current_client_index: 0,
        })
    }

    /// Accept any pending connections (non-blocking via try_accept).
    async fn accept_pending_connections(&mut self) -> Result<usize, FeagiNetworkError> {
        let listener = match &self.listener {
            Some(l) => l,
            None => return Ok(0),
        };

        let mut accepted = 0;
        loop {
            match listener.try_accept() {
                Ok((stream, _addr)) => match accept_async(stream).await {
                    Ok(ws) => {
                        self.clients.push(ws);
                        accepted += 1;
                    }
                    Err(e) => {
                        eprintln!("WebSocket handshake failed: {}", e);
                    }
                },
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(_) => {
                    break;
                }
            }
        }
        Ok(accepted)
    }
}

impl FeagiServer for FEAGIWebSocketServerPuller {
    async fn start(&mut self) -> Result<(), FeagiNetworkError> {
        let listener = TcpListener::bind(&self.bind_address)
            .await
            .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;

        self.listener = Some(listener);
        let previous = self.current_state;
        self.current_state = FeagiServerBindState::Active;
        (self.state_change_callback)(FeagiServerBindStateChange::new(
            previous,
            FeagiServerBindState::Active,
        ));
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        for mut client in self.clients.drain(..) {
            let _ = client.close(None).await;
        }
        self.listener = None;
        let previous = self.current_state;
        self.current_state = FeagiServerBindState::Inactive;
        (self.state_change_callback)(FeagiServerBindStateChange::new(
            previous,
            FeagiServerBindState::Inactive,
        ));
        Ok(())
    }

    fn get_current_state(&self) -> FeagiServerBindState {
        self.current_state
    }
}

impl FeagiServerPuller for FEAGIWebSocketServerPuller {
    async fn try_poll_receive(&mut self) -> Result<&[u8], FeagiNetworkError> {
        // Accept any pending connections first
        self.accept_pending_connections().await?;

        if self.clients.is_empty() {
            return Err(FeagiNetworkError::ReceiveFailed(
                "No clients connected".to_string(),
            ));
        }

        // Use select to wait for data from any client
        // For simplicity, we'll iterate through clients and await on each
        // A more sophisticated approach would use futures::select_all
        
        let mut failed_indices = Vec::new();

        for (i, client) in self.clients.iter_mut().enumerate() {
            loop {
                match client.next().await {
                    Some(Ok(Message::Binary(data))) => {
                        self.cached_data = data;
                        // Remove failed clients before returning
                        for idx in failed_indices.into_iter().rev() {
                            self.clients.remove(idx);
                        }
                        return Ok(&self.cached_data);
                    }
                    Some(Ok(Message::Text(text))) => {
                        self.cached_data = text.into_bytes();
                        for idx in failed_indices.into_iter().rev() {
                            self.clients.remove(idx);
                        }
                        return Ok(&self.cached_data);
                    }
                    Some(Ok(Message::Close(_))) => {
                        failed_indices.push(i);
                        break;
                    }
                    Some(Ok(_)) => {
                        // Ping/Pong - continue reading from this client
                        continue;
                    }
                    Some(Err(_)) => {
                        failed_indices.push(i);
                        break;
                    }
                    None => {
                        failed_indices.push(i);
                        break;
                    }
                }
            }
        }

        // Remove disconnected clients
        for i in failed_indices.into_iter().rev() {
            self.clients.remove(i);
        }

        Err(FeagiNetworkError::ReceiveFailed(
            "All clients disconnected".to_string(),
        ))
    }
}

//endregion

//region Router

/// WebSocket server that handles request-response communication.
pub struct FEAGIWebSocketServerRouter {
    bind_address: String,
    current_state: FeagiServerBindState,
    state_change_callback: StateChangeCallback,
    listener: Option<TcpListener>,
    clients: Vec<WebSocketStream<TcpStream>>,
    // Client ID tracking - map client index to SessionID
    next_client_id: u64,
    index_to_session: HashMap<usize, SessionID>,
    session_to_index: HashMap<SessionID, usize>,
    // Cached request data
    cached_request_data: Vec<u8>,
}

impl FEAGIWebSocketServerRouter {
    pub fn new(
        bind_address: String,
        state_change_callback: StateChangeCallback,
    ) -> Result<Self, FeagiNetworkError> {
        Ok(Self {
            bind_address,
            current_state: FeagiServerBindState::Inactive,
            state_change_callback,
            listener: None,
            clients: Vec::new(),
            next_client_id: 1,
            index_to_session: HashMap::new(),
            session_to_index: HashMap::new(),
            cached_request_data: Vec::new(),
        })
    }

    /// Generate a new SessionID from the counter.
    fn generate_session_id(&mut self) -> SessionID {
        let id = self.next_client_id;
        self.next_client_id += 1;
        SessionID::new(id.to_le_bytes())
    }

    /// Accept any pending connections (non-blocking via try_accept).
    async fn accept_pending_connections(&mut self) -> Result<usize, FeagiNetworkError> {
        let listener = match &self.listener {
            Some(l) => l,
            None => return Ok(0),
        };

        let mut accepted = 0;
        loop {
            match listener.try_accept() {
                Ok((stream, _addr)) => match accept_async(stream).await {
                    Ok(ws) => {
                        let index = self.clients.len();
                        let session_id = self.generate_session_id();

                        self.clients.push(ws);
                        self.index_to_session.insert(index, session_id);
                        self.session_to_index.insert(session_id, index);
                        accepted += 1;
                    }
                    Err(e) => {
                        eprintln!("WebSocket handshake failed: {}", e);
                    }
                },
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(_) => {
                    break;
                }
            }
        }
        Ok(accepted)
    }

    /// Remove a client and update the mappings.
    fn remove_client(&mut self, index: usize) {
        if let Some(session_id) = self.index_to_session.remove(&index) {
            self.session_to_index.remove(&session_id);
        }
        self.clients.remove(index);

        // Update indices for all clients after the removed one
        let mut new_index_to_session = HashMap::new();
        let mut new_session_to_index = HashMap::new();

        for (old_idx, session_id) in self.index_to_session.drain() {
            let new_idx = if old_idx > index {
                old_idx - 1
            } else {
                old_idx
            };
            new_index_to_session.insert(new_idx, session_id);
            new_session_to_index.insert(session_id, new_idx);
        }

        self.index_to_session = new_index_to_session;
        self.session_to_index = new_session_to_index;
    }
}

impl FeagiServer for FEAGIWebSocketServerRouter {
    async fn start(&mut self) -> Result<(), FeagiNetworkError> {
        let listener = TcpListener::bind(&self.bind_address)
            .await
            .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;

        self.listener = Some(listener);
        let previous = self.current_state;
        self.current_state = FeagiServerBindState::Active;
        (self.state_change_callback)(FeagiServerBindStateChange::new(
            previous,
            FeagiServerBindState::Active,
        ));
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        for mut client in self.clients.drain(..) {
            let _ = client.close(None).await;
        }
        self.index_to_session.clear();
        self.session_to_index.clear();
        self.listener = None;
        let previous = self.current_state;
        self.current_state = FeagiServerBindState::Inactive;
        (self.state_change_callback)(FeagiServerBindStateChange::new(
            previous,
            FeagiServerBindState::Inactive,
        ));
        Ok(())
    }

    fn get_current_state(&self) -> FeagiServerBindState {
        self.current_state
    }
}

impl FeagiServerRouter for FEAGIWebSocketServerRouter {
    async fn try_poll_receive(&mut self) -> Result<(SessionID, &[u8]), FeagiNetworkError> {
        // Accept any pending connections first
        self.accept_pending_connections().await?;

        if self.clients.is_empty() {
            return Err(FeagiNetworkError::ReceiveFailed(
                "No clients connected".to_string(),
            ));
        }

        let mut failed_indices = Vec::new();

        for (i, client) in self.clients.iter_mut().enumerate() {
            loop {
                match client.next().await {
                    Some(Ok(Message::Binary(data))) => {
                        self.cached_request_data = data;
                        if let Some(&session_id) = self.index_to_session.get(&i) {
                            // Remove failed clients before returning
                            for idx in failed_indices.into_iter().rev() {
                                self.remove_client(idx);
                            }
                            return Ok((session_id, &self.cached_request_data));
                        }
                    }
                    Some(Ok(Message::Text(text))) => {
                        self.cached_request_data = text.into_bytes();
                        if let Some(&session_id) = self.index_to_session.get(&i) {
                            for idx in failed_indices.into_iter().rev() {
                                self.remove_client(idx);
                            }
                            return Ok((session_id, &self.cached_request_data));
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        failed_indices.push(i);
                        break;
                    }
                    Some(Ok(_)) => {
                        // Ping/Pong - continue reading
                        continue;
                    }
                    Some(Err(_)) => {
                        failed_indices.push(i);
                        break;
                    }
                    None => {
                        failed_indices.push(i);
                        break;
                    }
                }
            }
        }

        // Remove failed clients (in reverse order)
        for i in failed_indices.into_iter().rev() {
            self.remove_client(i);
        }

        Err(FeagiNetworkError::ReceiveFailed(
            "All clients disconnected".to_string(),
        ))
    }

    async fn send_response(
        &mut self,
        client: SessionID,
        response: &[u8],
    ) -> Result<(), FeagiNetworkError> {
        let client_index = self.session_to_index.get(&client).ok_or_else(|| {
            FeagiNetworkError::SendFailed(format!("Unknown client session: {:?}", client))
        })?;

        if *client_index >= self.clients.len() {
            return Err(FeagiNetworkError::SendFailed(
                "Client disconnected".to_string(),
            ));
        }

        let message = Message::Binary(response.to_vec());
        self.clients[*client_index]
            .send(message)
            .await
            .map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;

        Ok(())
    }
}

//endregion
