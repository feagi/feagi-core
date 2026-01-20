//! WebSocket server implementations for FEAGI network traits.

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tungstenite::{accept, Message, WebSocket};
use tungstenite::protocol::Role;

use crate::next::FeagiNetworkError;
use crate::next::state_enums::FeagiServerBindState;
use crate::next::traits::server::{FeagiServer, FeagiServerPublisher, FeagiServerPuller, FeagiServerRouter};

//region Publisher

/// WebSocket server that broadcasts messages to all connected clients.
pub struct FEAGIWebSocketServerPublisher {
    bind_address: String,
    current_state: FeagiServerBindState,
    listener: Option<TcpListener>,
    clients: Vec<WebSocket<TcpStream>>,
}

impl FEAGIWebSocketServerPublisher {
    pub fn new(bind_address: String) -> Result<Self, FeagiNetworkError> {
        Ok(Self {
            bind_address,
            current_state: FeagiServerBindState::Inactive,
            listener: None,
            clients: Vec::new(),
        })
    }

    /// Accept any pending connections (non-blocking).
    /// Call this periodically to accept new clients.
    pub fn accept_pending_connections(&mut self) -> Result<usize, FeagiNetworkError> {
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
        Ok(())
    }

    fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        // Close all client connections
        for mut client in self.clients.drain(..) {
            let _ = client.close(None);
        }
        self.listener = None;
        self.current_state = FeagiServerBindState::Inactive;
        Ok(())
    }

    fn get_current_state(&self) -> FeagiServerBindState {
        self.current_state
    }
}

impl FeagiServerPublisher for FEAGIWebSocketServerPublisher {
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
    listener: Option<TcpListener>,
    clients: Vec<WebSocket<TcpStream>>,
    cached_data: Vec<u8>,
}

impl FEAGIWebSocketServerPuller {
    pub fn new(bind_address: String) -> Result<Self, FeagiNetworkError> {
        Ok(Self {
            bind_address,
            current_state: FeagiServerBindState::Inactive,
            listener: None,
            clients: Vec::new(),
            cached_data: Vec::new(),
        })
    }

    /// Accept any pending connections (non-blocking).
    pub fn accept_pending_connections(&mut self) -> Result<usize, FeagiNetworkError> {
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
        Ok(())
    }

    fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        for mut client in self.clients.drain(..) {
            let _ = client.close(None);
        }
        self.listener = None;
        self.current_state = FeagiServerBindState::Inactive;
        Ok(())
    }

    fn get_current_state(&self) -> FeagiServerBindState {
        self.current_state
    }
}

impl FeagiServerPuller for FEAGIWebSocketServerPuller {
    fn try_poll(&mut self) -> Result<bool, FeagiNetworkError> {
        // Check all clients for incoming data
        let mut failed_indices = Vec::new();
        
        for (i, client) in self.clients.iter_mut().enumerate() {
            match client.read() {
                Ok(Message::Binary(data)) => {
                    self.cached_data = data;
                    return Ok(true);
                }
                Ok(Message::Text(text)) => {
                    self.cached_data = text.into_bytes();
                    return Ok(true);
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
        
        Ok(false)
    }

    fn get_cached_data(&self) -> &[u8] {
        &self.cached_data
    }
}

//endregion

//region Router

/// WebSocket server that handles request-response communication.
pub struct FEAGIWebSocketServerRouter {
    bind_address: String,
    current_state: FeagiServerBindState,
    listener: Option<TcpListener>,
    clients: Vec<WebSocket<TcpStream>>,
    cached_request_data: Vec<u8>,
    last_request_client_index: Option<usize>,
}

impl FEAGIWebSocketServerRouter {
    pub fn new(bind_address: String) -> Result<Self, FeagiNetworkError> {
        Ok(Self {
            bind_address,
            current_state: FeagiServerBindState::Inactive,
            listener: None,
            clients: Vec::new(),
            cached_request_data: Vec::new(),
            last_request_client_index: None,
        })
    }

    /// Accept any pending connections (non-blocking).
    pub fn accept_pending_connections(&mut self) -> Result<usize, FeagiNetworkError> {
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

impl FeagiServer for FEAGIWebSocketServerRouter {
    fn start(&mut self) -> Result<(), FeagiNetworkError> {
        let listener = TcpListener::bind(&self.bind_address)
            .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;
        listener.set_nonblocking(true)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        self.listener = Some(listener);
        self.current_state = FeagiServerBindState::Active;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        for mut client in self.clients.drain(..) {
            let _ = client.close(None);
        }
        self.listener = None;
        self.current_state = FeagiServerBindState::Inactive;
        Ok(())
    }

    fn get_current_state(&self) -> FeagiServerBindState {
        self.current_state
    }
}

impl FeagiServerRouter for FEAGIWebSocketServerRouter {
    fn try_poll(&mut self) -> Result<bool, FeagiNetworkError> {
        let mut failed_indices = Vec::new();
        
        for (i, client) in self.clients.iter_mut().enumerate() {
            match client.read() {
                Ok(Message::Binary(data)) => {
                    self.cached_request_data = data;
                    self.last_request_client_index = Some(i);
                    return Ok(true);
                }
                Ok(Message::Text(text)) => {
                    self.cached_request_data = text.into_bytes();
                    self.last_request_client_index = Some(i);
                    return Ok(true);
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
        
        for i in failed_indices.into_iter().rev() {
            self.clients.remove(i);
            // Adjust last_request_client_index if needed
            if let Some(idx) = self.last_request_client_index {
                if i < idx {
                    self.last_request_client_index = Some(idx - 1);
                } else if i == idx {
                    self.last_request_client_index = None;
                }
            }
        }
        
        Ok(false)
    }

    fn get_request_data(&self) -> &[u8] {
        &self.cached_request_data
    }

    fn send_response(&mut self, response: &[u8]) -> Result<(), FeagiNetworkError> {
        let client_index = self.last_request_client_index
            .ok_or_else(|| FeagiNetworkError::SendFailed("No client to respond to".to_string()))?;
        
        if client_index >= self.clients.len() {
            return Err(FeagiNetworkError::SendFailed("Client disconnected".to_string()));
        }
        
        let message = Message::Binary(response.to_vec());
        self.clients[client_index].send(message)
            .map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;
        
        Ok(())
    }
}

//endregion
