//! WebSocket server implementations for FEAGI network traits.

use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};

use tungstenite::{accept, Message, WebSocket};

use crate::next::FeagiNetworkError;
use crate::next::traits_and_enums::server::server_shared::{ClientId, FeagiServerBindState, FeagiServerBindStateChange};
use crate::next::traits_and_enums::server::{
    FeagiServer, FeagiServerPublisher, FeagiServerPuller, FeagiServerRouter,
    FeagiServerPublisherProperties, FeagiServerPullerProperties, FeagiServerRouterProperties
};

/// Type alias for the server state change callback.
type StateChangeCallback = Box<dyn Fn(FeagiServerBindStateChange) + Send + Sync + 'static>;

//region Publisher

/// WebSocket server that broadcasts messages to all connected clients.
pub struct FEAGIWebSocketServerPublisher {
    bind_address: String,
    current_state: FeagiServerBindState,
    state_change_callback: StateChangeCallback,
    listener: Option<TcpListener>,
    clients: Vec<WebSocket<TcpStream>>,
}

impl FEAGIWebSocketServerPublisher {
    pub fn new(bind_address: String, state_change_callback: StateChangeCallback) -> Result<Self, FeagiNetworkError> {
        Ok(Self {
            bind_address,
            current_state: FeagiServerBindState::Inactive,
            state_change_callback,
            listener: None,
            clients: Vec::new(),
        })
    }

    /// Accept any pending connections (non-blocking).
    fn accept_pending_connections(&mut self) -> Result<usize, FeagiNetworkError> {
        let listener = match &self.listener {
            Some(l) => l,
            None => return Ok(0),
        };

        let mut accepted = 0;
        loop {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    stream.set_nonblocking(true)
                        .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
                    match accept(stream) {
                        Ok(ws) => {
                            self.clients.push(ws);
                            accepted += 1;
                        }
                        Err(e) => {
                            // Log but don't fail - client may have disconnected
                            eprintln!("WebSocket handshake failed: {}", e);
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break; // No more pending connections
                }
                Err(e) => {
                    return Err(FeagiNetworkError::ReceiveFailed(e.to_string()));
                }
            }
        }
        Ok(accepted)
    }

    /// Get the number of connected clients.
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }
}

impl FeagiServer for FEAGIWebSocketServerPublisher {
    fn start(&mut self) -> Result<(), FeagiNetworkError> {
        let listener = TcpListener::bind(&self.bind_address)
            .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;
        listener.set_nonblocking(true)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        self.listener = Some(listener);
        self.current_state = FeagiServerBindState::Active;
        (self.state_change_callback)(
            FeagiServerBindStateChange::new(FeagiServerBindState::Inactive, FeagiServerBindState::Active)
        );
        Ok(())
    }

    fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        // Close all client connections
        for mut client in self.clients.drain(..) {
            let _ = client.close(None);
        }
        self.listener = None;
        self.current_state = FeagiServerBindState::Inactive;
        (self.state_change_callback)(
            FeagiServerBindStateChange::new(FeagiServerBindState::Active, FeagiServerBindState::Inactive)
        );
        Ok(())
    }

    fn get_current_state(&self) -> FeagiServerBindState {
        self.current_state
    }
}

impl FeagiServerPublisher for FEAGIWebSocketServerPublisher {
    fn poll(&mut self) -> Result<(), FeagiNetworkError> {
        // Accept any pending connections
        self.accept_pending_connections()?;
        Ok(())
    }

    fn publish(&mut self, buffered_data_to_send: &[u8]) -> Result<(), FeagiNetworkError> {
        let message = Message::Binary(buffered_data_to_send.to_vec());
        
        // Send to all clients, removing any that fail
        let mut failed_indices = Vec::new();
        for (i, client) in self.clients.iter_mut().enumerate() {
            if client.send(message.clone()).is_err() {
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
    clients: Vec<WebSocket<TcpStream>>,
    cached_data: Vec<u8>,
}

impl FEAGIWebSocketServerPuller {
    pub fn new(bind_address: String, state_change_callback: StateChangeCallback) -> Result<Self, FeagiNetworkError> {
        Ok(Self {
            bind_address,
            current_state: FeagiServerBindState::Inactive,
            state_change_callback,
            listener: None,
            clients: Vec::new(),
            cached_data: Vec::new(),
        })
    }

    /// Accept any pending connections (non-blocking).
    fn accept_pending_connections(&mut self) -> Result<usize, FeagiNetworkError> {
        let listener = match &self.listener {
            Some(l) => l,
            None => return Ok(0),
        };

        let mut accepted = 0;
        loop {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    stream.set_nonblocking(true)
                        .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
                    match accept(stream) {
                        Ok(ws) => {
                            self.clients.push(ws);
                            accepted += 1;
                        }
                        Err(e) => {
                            eprintln!("WebSocket handshake failed: {}", e);
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => {
                    return Err(FeagiNetworkError::ReceiveFailed(e.to_string()));
                }
            }
        }
        Ok(accepted)
    }
}

impl FeagiServer for FEAGIWebSocketServerPuller {
    fn start(&mut self) -> Result<(), FeagiNetworkError> {
        let listener = TcpListener::bind(&self.bind_address)
            .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;
        listener.set_nonblocking(true)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        self.listener = Some(listener);
        self.current_state = FeagiServerBindState::Active;
        (self.state_change_callback)(
            FeagiServerBindStateChange::new(FeagiServerBindState::Inactive, FeagiServerBindState::Active)
        );
        Ok(())
    }

    fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        for mut client in self.clients.drain(..) {
            let _ = client.close(None);
        }
        self.listener = None;
        self.current_state = FeagiServerBindState::Inactive;
        (self.state_change_callback)(
            FeagiServerBindStateChange::new(FeagiServerBindState::Active, FeagiServerBindState::Inactive)
        );
        Ok(())
    }

    fn get_current_state(&self) -> FeagiServerBindState {
        self.current_state
    }
}

impl FeagiServerPuller for FEAGIWebSocketServerPuller {
    fn try_poll_receive(&mut self) -> Result<Option<&[u8]>, FeagiNetworkError> {
        // Accept any pending connections first
        self.accept_pending_connections()?;
        
        // Check all clients for incoming data
        let mut failed_indices = Vec::new();
        
        for (i, client) in self.clients.iter_mut().enumerate() {
            match client.read() {
                Ok(Message::Binary(data)) => {
                    self.cached_data = data;
                    return Ok(Some(&self.cached_data));
                }
                Ok(Message::Text(text)) => {
                    self.cached_data = text.into_bytes();
                    return Ok(Some(&self.cached_data));
                }
                Ok(Message::Close(_)) => {
                    failed_indices.push(i);
                }
                Ok(_) => {
                    // Ping/Pong/Frame - continue
                }
                Err(tungstenite::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data available on this client
                }
                Err(_) => {
                    failed_indices.push(i);
                }
            }
        }
        
        // Remove disconnected clients
        for i in failed_indices.into_iter().rev() {
            self.clients.remove(i);
        }
        
        Ok(None)
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
    clients: Vec<WebSocket<TcpStream>>,
    // Client ID tracking - map client index to ClientId
    next_client_id: u64,
    index_to_id: HashMap<usize, u64>,
    id_to_index: HashMap<u64, usize>,
    // Cached request data
    cached_request_data: Vec<u8>,
    last_client_id: Option<ClientId>,
}

impl FEAGIWebSocketServerRouter {
    pub fn new(bind_address: String, state_change_callback: StateChangeCallback) -> Result<Self, FeagiNetworkError> {
        Ok(Self {
            bind_address,
            current_state: FeagiServerBindState::Inactive,
            state_change_callback,
            listener: None,
            clients: Vec::new(),
            next_client_id: 1,
            index_to_id: HashMap::new(),
            id_to_index: HashMap::new(),
            cached_request_data: Vec::new(),
            last_client_id: None,
        })
    }

    /// Accept any pending connections (non-blocking).
    fn accept_pending_connections(&mut self) -> Result<usize, FeagiNetworkError> {
        let listener = match &self.listener {
            Some(l) => l,
            None => return Ok(0),
        };

        let mut accepted = 0;
        loop {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    stream.set_nonblocking(true)
                        .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
                    match accept(stream) {
                        Ok(ws) => {
                            let index = self.clients.len();
                            let client_id = self.next_client_id;
                            self.next_client_id += 1;
                            
                            self.clients.push(ws);
                            self.index_to_id.insert(index, client_id);
                            self.id_to_index.insert(client_id, index);
                            accepted += 1;
                        }
                        Err(e) => {
                            eprintln!("WebSocket handshake failed: {}", e);
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => {
                    return Err(FeagiNetworkError::ReceiveFailed(e.to_string()));
                }
            }
        }
        Ok(accepted)
    }

    /// Remove a client and update the mappings.
    fn remove_client(&mut self, index: usize) {
        if let Some(client_id) = self.index_to_id.remove(&index) {
            self.id_to_index.remove(&client_id);
        }
        self.clients.remove(index);
        
        // Update indices for all clients after the removed one
        let mut new_index_to_id = HashMap::new();
        let mut new_id_to_index = HashMap::new();
        
        for (old_idx, client_id) in self.index_to_id.drain() {
            let new_idx = if old_idx > index { old_idx - 1 } else { old_idx };
            new_index_to_id.insert(new_idx, client_id);
            new_id_to_index.insert(client_id, new_idx);
        }
        
        self.index_to_id = new_index_to_id;
        self.id_to_index = new_id_to_index;
    }
}

impl FeagiServer for FEAGIWebSocketServerRouter {
    fn start(&mut self) -> Result<(), FeagiNetworkError> {
        let listener = TcpListener::bind(&self.bind_address)
            .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;
        listener.set_nonblocking(true)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        self.listener = Some(listener);
        self.current_state = FeagiServerBindState::Active;
        (self.state_change_callback)(
            FeagiServerBindStateChange::new(FeagiServerBindState::Inactive, FeagiServerBindState::Active)
        );
        Ok(())
    }

    fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        for mut client in self.clients.drain(..) {
            let _ = client.close(None);
        }
        self.index_to_id.clear();
        self.id_to_index.clear();
        self.listener = None;
        self.current_state = FeagiServerBindState::Inactive;
        (self.state_change_callback)(
            FeagiServerBindStateChange::new(FeagiServerBindState::Active, FeagiServerBindState::Inactive)
        );
        Ok(())
    }

    fn get_current_state(&self) -> FeagiServerBindState {
        self.current_state
    }
}

impl FeagiServerRouter for FEAGIWebSocketServerRouter {
    fn try_poll_receive(&mut self) -> Result<Option<(ClientId, &[u8])>, FeagiNetworkError> {
        // Accept any pending connections first
        self.accept_pending_connections()?;
        
        let mut failed_indices = Vec::new();
        
        for (i, client) in self.clients.iter_mut().enumerate() {
            match client.read() {
                Ok(Message::Binary(data)) => {
                    self.cached_request_data = data;
                    if let Some(&client_id) = self.index_to_id.get(&i) {
                        self.last_client_id = Some(ClientId(client_id));
                        return Ok(Some((ClientId(client_id), &self.cached_request_data)));
                    }
                }
                Ok(Message::Text(text)) => {
                    self.cached_request_data = text.into_bytes();
                    if let Some(&client_id) = self.index_to_id.get(&i) {
                        self.last_client_id = Some(ClientId(client_id));
                        return Ok(Some((ClientId(client_id), &self.cached_request_data)));
                    }
                }
                Ok(Message::Close(_)) => {
                    failed_indices.push(i);
                }
                Ok(_) => {}
                Err(tungstenite::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_) => {
                    failed_indices.push(i);
                }
            }
        }
        
        // Remove failed clients (in reverse order)
        for i in failed_indices.into_iter().rev() {
            self.remove_client(i);
        }
        
        Ok(None)
    }

    fn send_response(&mut self, client: ClientId, response: &[u8]) -> Result<(), FeagiNetworkError> {
        let client_index = self.id_to_index.get(&client.0)
            .ok_or_else(|| FeagiNetworkError::SendFailed(format!("Unknown client ID: {:?}", client)))?;
        
        if *client_index >= self.clients.len() {
            return Err(FeagiNetworkError::SendFailed("Client disconnected".to_string()));
        }
        
        let message = Message::Binary(response.to_vec());
        self.clients[*client_index].send(message)
            .map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;
        
        Ok(())
    }
}

//endregion

//region Properties

//region Publisher Properties

/// Properties for configuring and building a WebSocket Server Publisher.
pub struct FEAGIWebSocketServerPublisherProperties {
    bind_address: String,
}

impl FEAGIWebSocketServerPublisherProperties {
    /// Create new properties with the given bind address.
    pub fn new(bind_address: String) -> Self {
        Self {
            bind_address,
        }
    }
}

impl FeagiServerPublisherProperties for FEAGIWebSocketServerPublisherProperties {
    fn build(self: Box<Self>, state_change_callback: StateChangeCallback) -> Box<dyn FeagiServerPublisher> {
        let publisher = FEAGIWebSocketServerPublisher::new(
            self.bind_address,
            state_change_callback,
        ).expect("Failed to create WebSocket publisher");
        
        Box::new(publisher)
    }
}

//endregion

//region Puller Properties

/// Properties for configuring and building a WebSocket Server Puller.
pub struct FEAGIWebSocketServerPullerProperties {
    bind_address: String,
}

impl FEAGIWebSocketServerPullerProperties {
    /// Create new properties with the given bind address.
    pub fn new(bind_address: String) -> Self {
        Self {
            bind_address,
        }
    }
}

impl FeagiServerPullerProperties for FEAGIWebSocketServerPullerProperties {
    fn build(self: Box<Self>, state_change_callback: StateChangeCallback) -> Box<dyn FeagiServerPuller> {
        let puller = FEAGIWebSocketServerPuller::new(
            self.bind_address,
            state_change_callback,
        ).expect("Failed to create WebSocket puller");
        
        Box::new(puller)
    }
}

//endregion

//region Router Properties

/// Properties for configuring and building a WebSocket Server Router.
pub struct FEAGIWebSocketServerRouterProperties {
    bind_address: String,
}

impl FEAGIWebSocketServerRouterProperties {
    /// Create new properties with the given bind address.
    pub fn new(bind_address: String) -> Self {
        Self {
            bind_address,
        }
    }
}

impl FeagiServerRouterProperties for FEAGIWebSocketServerRouterProperties {
    fn build(self: Box<Self>, state_change_callback: StateChangeCallback) -> Box<dyn FeagiServerRouter> {
        let router = FEAGIWebSocketServerRouter::new(
            self.bind_address,
            state_change_callback,
        ).expect("Failed to create WebSocket router");
        
        Box::new(router)
    }
}

//endregion

//endregion
