//! WebSocket server implementations using the poll-based trait design.
//!
//! Uses `tungstenite` with non-blocking `std::net::TcpStream` for poll-based
//! WebSocket communication that works with any async runtime or synchronously.

use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::{TcpListener, TcpStream};

use tungstenite::{accept, Message, WebSocket};

use crate::{AgentID, FeagiNetworkError};
use crate::protocol_implementations::websocket::shared::{validate_bind_address, WebSocketUrl};
use crate::traits_and_enums::shared::{FeagiEndpointState, TransportProtocolEndpoint, TransportProtocolImplementation};
use crate::traits_and_enums::server::{
    FeagiServer, FeagiServerPublisher, FeagiServerPublisherProperties,
    FeagiServerPuller, FeagiServerPullerProperties,
    FeagiServerRouter, FeagiServerRouterProperties,
};
use feagi_serialization::FeagiByteContainer;

/// Type alias for WebSocket over TcpStream
type WsStream = WebSocket<TcpStream>;

/// State of a WebSocket connection during handshake
enum HandshakeState {
    /// TCP accepted, handshake in progress
    Handshaking(TcpStream),
    /// Handshake complete, WebSocket ready
    Ready(WsStream),
    /// Handshake failed
    Failed,
}

// ============================================================================
// Publisher
// ============================================================================

//region Publisher Properties

/// Configuration properties for creating a WebSocket publisher server.
#[derive(Debug, Clone, PartialEq)]
pub struct FeagiWebSocketServerPublisherProperties {
    bind_address: String,
}

impl FeagiWebSocketServerPublisherProperties {
    /// Creates new publisher properties with the given bind address.
    ///
    /// # Arguments
    ///
    /// * `bind_address` - The address to bind to (e.g., "127.0.0.1:8080" or "0.0.0.0:8080").
    pub fn new(bind_address: &str) -> Result<Self, FeagiNetworkError> {
        validate_bind_address(bind_address)?;
        Ok(Self {
            bind_address: bind_address.to_string(),
        })
    }
}

impl FeagiServerPublisherProperties for FeagiWebSocketServerPublisherProperties {
    fn as_boxed_server_publisher(&self) -> Box<dyn FeagiServerPublisher> {
        Box::new(FeagiWebSocketServerPublisher {
            bind_address: self.bind_address.clone(),
            current_state: FeagiEndpointState::Inactive,
            listener: None,
            clients: Vec::new(),
        })
    }

    fn get_protocol(&self) -> TransportProtocolImplementation {
        TransportProtocolImplementation::WebSocket
    }

    fn get_endpoint(&self) -> TransportProtocolEndpoint {
        // Create a WebSocketUrl for the endpoint (using ws:// scheme for bind address)
        TransportProtocolEndpoint::WebSocket(
            WebSocketUrl::new(&self.bind_address).expect("bind_address already validated")
        )
    }
}

//endregion

//region Publisher Implementation

/// A WebSocket server that broadcasts data to all connected clients.
pub struct FeagiWebSocketServerPublisher {
    bind_address: String,
    current_state: FeagiEndpointState,
    listener: Option<TcpListener>,
    clients: Vec<WsStream>,
}

impl FeagiWebSocketServerPublisher {
    /// Accept any pending connections (non-blocking).
    fn accept_pending_connections(&mut self) {
        let listener = match &self.listener {
            Some(l) => l,
            None => return,
        };

        // Try to accept connections in non-blocking mode
        loop {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    // Perform handshake first on blocking stream.
                    // Setting non-blocking before handshake can cause immediate failures.
                    match accept(stream) {
                        Ok(mut ws) => {
                            // After handshake, switch the socket to non-blocking mode for poll-based operation.
                            let _ = ws.get_mut().set_nonblocking(true);
                            self.clients.push(ws);
                        }
                        Err(_e) => {
                            // Handshake failed, skip this connection
                        }
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // No more pending connections
                    break;
                }
                Err(_) => {
                    break;
                }
            }
        }
    }

    /// Get the number of connected clients.
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }
}

impl FeagiServer for FeagiWebSocketServerPublisher {
    fn poll(&mut self) -> &FeagiEndpointState {
        if matches!(self.current_state, FeagiEndpointState::ActiveWaiting) {
            self.accept_pending_connections();
        }
        &self.current_state
    }

    fn request_start(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::Inactive => {
                let listener = TcpListener::bind(&self.bind_address)
                    .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;

                // Set listener to non-blocking for poll-based accept
                listener
                    .set_nonblocking(true)
                    .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;

                self.listener = Some(listener);
                self.current_state = FeagiEndpointState::ActiveWaiting;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot start: server is not in Inactive state".to_string(),
            )),
        }
    }

    fn request_stop(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                // Close all client connections
                for mut client in self.clients.drain(..) {
                    let _ = client.close(None);
                }
                self.listener = None;
                self.current_state = FeagiEndpointState::Inactive;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot stop: server is not in Active state".to_string(),
            )),
        }
    }

    fn confirm_error_and_close(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::Errored(_) => {
                for mut client in self.clients.drain(..) {
                    let _ = client.close(None);
                }
                self.listener = None;
                self.current_state = FeagiEndpointState::Inactive;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot confirm error: server is not in Errored state".to_string(),
            )),
        }
    }

    fn get_protocol(&self) -> TransportProtocolImplementation {
        TransportProtocolImplementation::WebSocket
    }

    fn get_endpoint(&self) -> TransportProtocolEndpoint {
        TransportProtocolEndpoint::WebSocket(
            WebSocketUrl::new(&self.bind_address).expect("bind_address already validated")
        )
    }
}

impl FeagiServerPublisher for FeagiWebSocketServerPublisher {
    fn publish_data(&mut self, data: &[u8]) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                let message = Message::Binary(data.to_vec());

                // Send to all clients, tracking which ones fail
                let mut failed_indices = Vec::new();
                for (i, client) in self.clients.iter_mut().enumerate() {
                    // Try to send, handling WouldBlock gracefully
                    match client.send(message.clone()) {
                        Ok(()) => {}
                        Err(tungstenite::Error::Io(ref e)) if e.kind() == ErrorKind::WouldBlock => {
                            // Socket buffer full - could buffer, but for now skip
                        }
                        Err(_) => {
                            failed_indices.push(i);
                        }
                    }
                    // Flush to actually send
                    let _ = client.flush();
                }

                // Remove failed clients (in reverse order to preserve indices)
                for i in failed_indices.into_iter().rev() {
                    let _ = self.clients.remove(i).close(None);
                }

                Ok(())
            }
            _ => Err(FeagiNetworkError::SendFailed(
                "Cannot publish: server is not in Active state".to_string(),
            )),
        }
    }

    fn as_boxed_publisher_properties(&self) -> Box<dyn FeagiServerPublisherProperties> {
        Box::new(FeagiWebSocketServerPublisherProperties {
            bind_address: self.bind_address.clone(),
        })
    }
}

//endregion

// ============================================================================
// Puller
// ============================================================================

//region Puller Properties

/// Configuration properties for creating a WebSocket puller server.
#[derive(Debug, Clone, PartialEq)]
pub struct FeagiWebSocketServerPullerProperties {
    bind_address: String,
}

impl FeagiWebSocketServerPullerProperties {
    /// Creates new puller properties with the given bind address.
    pub fn new(bind_address: &str) -> Result<Self, FeagiNetworkError> {
        validate_bind_address(bind_address)?;
        Ok(Self {
            bind_address: bind_address.to_string(),
        })
    }
}

impl FeagiServerPullerProperties for FeagiWebSocketServerPullerProperties {
    fn as_boxed_server_puller(&self) -> Box<dyn FeagiServerPuller> {
        Box::new(FeagiWebSocketServerPuller {
            bind_address: self.bind_address.clone(),
            current_state: FeagiEndpointState::Inactive,
            listener: None,
            clients: Vec::new(),
            receive_buffer: None,
            has_data: false,
        })
    }

    fn get_protocol(&self) -> TransportProtocolImplementation {
        TransportProtocolImplementation::WebSocket
    }

    fn get_endpoint(&self) -> TransportProtocolEndpoint {
        TransportProtocolEndpoint::WebSocket(
            WebSocketUrl::new(&self.bind_address).expect("bind_address already validated")
        )
    }
}

//endregion

//region Puller Implementation

/// A WebSocket server that receives pushed data from clients.
pub struct FeagiWebSocketServerPuller {
    bind_address: String,
    current_state: FeagiEndpointState,
    listener: Option<TcpListener>,
    clients: Vec<WsStream>,
    /// Buffer for received data
    receive_buffer: Option<Vec<u8>>,
    has_data: bool,
}

impl FeagiWebSocketServerPuller {
    fn accept_pending_connections(&mut self) {
        let listener = match &self.listener {
            Some(l) => l,
            None => return,
        };

        loop {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    if stream.set_nonblocking(true).is_err() {
                        continue;
                    }
                    match accept(stream) {
                        Ok(ws) => {
                            self.clients.push(ws);
                        }
                        Err(_) => {}
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }
    }

    /// Try to receive data from any client (non-blocking).
    fn try_receive(&mut self) -> bool {
        let mut failed_indices = Vec::new();

        for (i, client) in self.clients.iter_mut().enumerate() {
            match client.read() {
                Ok(Message::Binary(data)) => {
                    self.receive_buffer = Some(data);
                    // Remove failed clients before returning
                    for idx in failed_indices.into_iter().rev() {
                        let _ = self.clients.remove(idx).close(None);
                    }
                    return true;
                }
                Ok(Message::Text(text)) => {
                    self.receive_buffer = Some(text.into_bytes());
                    for idx in failed_indices.into_iter().rev() {
                        let _ = self.clients.remove(idx).close(None);
                    }
                    return true;
                }
                Ok(Message::Close(_)) => {
                    failed_indices.push(i);
                }
                Ok(_) => {
                    // Ping/Pong/Frame - continue
                }
                Err(tungstenite::Error::Io(ref e)) if e.kind() == ErrorKind::WouldBlock => {
                    // No data from this client
                }
                Err(_) => {
                    failed_indices.push(i);
                }
            }
        }

        // Remove failed clients
        for i in failed_indices.into_iter().rev() {
            let _ = self.clients.remove(i).close(None);
        }

        false
    }
}

impl FeagiServer for FeagiWebSocketServerPuller {
    fn poll(&mut self) -> &FeagiEndpointState {
        if matches!(self.current_state, FeagiEndpointState::ActiveWaiting) && !self.has_data {
            self.accept_pending_connections();

            if self.try_receive() {
                self.has_data = true;
                self.current_state = FeagiEndpointState::ActiveHasData;
            }
        }
        &self.current_state
    }

    fn request_start(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::Inactive => {
                let listener = TcpListener::bind(&self.bind_address)
                    .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;
                listener
                    .set_nonblocking(true)
                    .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;

                self.listener = Some(listener);
                self.current_state = FeagiEndpointState::ActiveWaiting;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot start: server is not in Inactive state".to_string(),
            )),
        }
    }

    fn request_stop(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                for mut client in self.clients.drain(..) {
                    let _ = client.close(None);
                }
                self.listener = None;
                self.receive_buffer = None;
                self.has_data = false;
                self.current_state = FeagiEndpointState::Inactive;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot stop: server is not in Active state".to_string(),
            )),
        }
    }

    fn confirm_error_and_close(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::Errored(_) => {
                for mut client in self.clients.drain(..) {
                    let _ = client.close(None);
                }
                self.listener = None;
                self.receive_buffer = None;
                self.has_data = false;
                self.current_state = FeagiEndpointState::Inactive;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot confirm error: server is not in Errored state".to_string(),
            )),
        }
    }

    fn get_protocol(&self) -> TransportProtocolImplementation {
        TransportProtocolImplementation::WebSocket
    }

    fn get_endpoint(&self) -> TransportProtocolEndpoint {
        TransportProtocolEndpoint::WebSocket(
            WebSocketUrl::new(&self.bind_address).expect("bind_address already validated")
        )
    }
}

impl FeagiServerPuller for FeagiWebSocketServerPuller {
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

    fn as_boxed_puller_properties(&self) -> Box<dyn FeagiServerPullerProperties> {
        Box::new(FeagiWebSocketServerPullerProperties {
            bind_address: self.bind_address.clone(),
        })
    }
}

//endregion

// ============================================================================
// Router
// ============================================================================

//region Router Properties

/// Configuration properties for creating a WebSocket router server.
#[derive(Debug, Clone, PartialEq)]
pub struct FeagiWebSocketServerRouterProperties {
    bind_address: String,
}

impl FeagiWebSocketServerRouterProperties {
    /// Creates new router properties with the given bind address.
    pub fn new(bind_address: &str) -> Result<Self, FeagiNetworkError> {
        validate_bind_address(bind_address)?;
        Ok(Self {
            bind_address: bind_address.to_string(),
        })
    }
}

impl FeagiServerRouterProperties for FeagiWebSocketServerRouterProperties {
    fn as_boxed_server_router(&self) -> Box<dyn FeagiServerRouter> {
        Box::new(FeagiWebSocketServerRouter {
            bind_address: self.bind_address.clone(),
            current_state: FeagiEndpointState::Inactive,
            listener: None,
            clients: Vec::new(),
            receive_buffer: None,
            current_session: None,
            has_data: false,
            index_to_session: HashMap::new(),
            session_to_index: HashMap::new(),
        })
    }
}

//endregion

//region Router Implementation

/// A WebSocket server that handles request-response communication with multiple clients.
pub struct FeagiWebSocketServerRouter {
    bind_address: String,
    current_state: FeagiEndpointState,
    listener: Option<TcpListener>,
    /// Connections in various states (handshaking or ready)
    clients: Vec<HandshakeState>,
    /// Buffer for received request
    receive_buffer: Option<Vec<u8>>,
    /// Session ID of the client that sent the current request
    current_session: Option<AgentID>,
    has_data: bool,
    /// Client index to AgentID mapping
    index_to_session: HashMap<usize, AgentID>,
    /// AgentID to client index mapping
    session_to_index: HashMap<AgentID, usize>,
}

impl FeagiWebSocketServerRouter {
    fn try_extract_agent_id_from_payload(payload: &[u8]) -> Option<AgentID> {
        let start = FeagiByteContainer::GLOBAL_BYTE_HEADER_BYTE_COUNT;
        let end = start + FeagiByteContainer::AGENT_ID_BYTE_COUNT;
        if payload.len() < end {
            return None;
        }

        let mut id_bytes = [0u8; AgentID::NUMBER_BYTES];
        id_bytes.copy_from_slice(&payload[start..end]);
        let parsed_id = AgentID::new(id_bytes);
        if parsed_id.is_blank() {
            return None;
        }
        Some(parsed_id)
    }

    fn remap_client_session(&mut self, client_index: usize, new_session_id: AgentID) {
        let previous_session = self.index_to_session.insert(client_index, new_session_id);
        if let Some(old_session_id) = previous_session {
            self.session_to_index.remove(&old_session_id);
        }

        if let Some(previous_index) = self.session_to_index.insert(new_session_id, client_index) {
            if previous_index != client_index {
                self.index_to_session.remove(&previous_index);
            }
        }
    }

    fn align_session_with_payload_agent_id(&mut self, client_index: usize, payload: &[u8]) {
        if let Some(payload_agent_id) = Self::try_extract_agent_id_from_payload(payload) {
            self.remap_client_session(client_index, payload_agent_id);
        }
    }

    fn accept_pending_connections(&mut self) {
        let listener = match &self.listener {
            Some(l) => l,
            None => return,
        };

        // Accept new TCP connections and start handshake
        loop {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    // Keep stream blocking for handshake (tungstenite requires it)
                    // We'll switch to non-blocking after handshake completes
                    self.clients.push(HandshakeState::Handshaking(stream));
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }
    }
    
    fn process_handshakes(&mut self) {
        let mut indices_to_remove = Vec::new();
        
        for (i, state) in self.clients.iter_mut().enumerate() {
            match state {
                HandshakeState::Handshaking(stream) => {
                    // Try to complete handshake (this may block briefly but should be fast)
                    match stream.try_clone() {
                        Ok(cloned_stream) => {
                            match accept(cloned_stream) {
                                Ok(ws) => {
                                    // Handshake successful - generate AgentID
                                    let session_id = AgentID::new_random();
                                    
                                    // Set underlying stream to non-blocking for polling
                                    if let Ok(tcp_stream) = ws.get_ref().try_clone() {
                                        let _ = tcp_stream.set_nonblocking(true);
                                    }
                                    
                                    *state = HandshakeState::Ready(ws);
                                    self.index_to_session.insert(i, session_id);
                                    self.session_to_index.insert(session_id, i);
                                }
                                Err(_e) => {
                                    // Handshake failed or not ready - mark for removal on next cycle
                                    // Don't immediately remove to give handshake time to complete
                                }
                            }
                        }
                        Err(_) => {
                            indices_to_remove.push(i);
                        }
                    }
                }
                HandshakeState::Failed => {
                    indices_to_remove.push(i);
                }
                HandshakeState::Ready(_) => {
                    // Already connected, nothing to do
                }
            }
        }
        
        // Remove failed handshakes
        for &i in indices_to_remove.iter().rev() {
            if i < self.clients.len() {
                self.remove_client(i);
            }
        }
    }

    fn try_receive(&mut self) -> bool {
        let mut failed_indices = Vec::new();

        for i in 0..self.clients.len() {
            let read_result = {
                let client = match self.clients.get_mut(i) {
                    Some(HandshakeState::Ready(ws)) => ws,
                    _ => continue, // Skip handshaking/failed connections
                };
                client.read()
            };

            match read_result {
                Ok(Message::Binary(data)) => {
                    self.align_session_with_payload_agent_id(i, &data);
                    if let Some(&session_id) = self.index_to_session.get(&i) {
                        self.receive_buffer = Some(data);
                        self.current_session = Some(session_id);
                        // Remove failed clients before returning
                        for idx in failed_indices.into_iter().rev() {
                            self.remove_client(idx);
                        }
                        return true;
                    }
                }
                Ok(Message::Text(text)) => {
                    let text_bytes = text.into_bytes();
                    self.align_session_with_payload_agent_id(i, &text_bytes);
                    if let Some(&session_id) = self.index_to_session.get(&i) {
                        self.receive_buffer = Some(text_bytes);
                        self.current_session = Some(session_id);
                        for idx in failed_indices.into_iter().rev() {
                            self.remove_client(idx);
                        }
                        return true;
                    }
                }
                Ok(Message::Close(_)) => {
                    failed_indices.push(i);
                }
                Ok(_) => {}
                Err(tungstenite::Error::Io(ref e)) if e.kind() == ErrorKind::WouldBlock => {}
                Err(_) => {
                    failed_indices.push(i);
                }
            }
        }

        for i in failed_indices.into_iter().rev() {
            self.remove_client(i);
        }

        false
    }

    fn remove_client(&mut self, index: usize) {
        if index >= self.clients.len() {
            return;
        }

        // Remove and close the client
        let removed_state = self.clients.remove(index);
        if let HandshakeState::Ready(mut ws) = removed_state {
            let _ = ws.close(None);
        }

        // Remove from mappings
        if let Some(session_id) = self.index_to_session.remove(&index) {
            self.session_to_index.remove(&session_id);
        }

        // Update indices for all clients after the removed one
        let mut new_index_to_session = HashMap::new();
        let mut new_session_to_index = HashMap::new();

        for (old_idx, session_id) in self.index_to_session.drain() {
            let new_idx = if old_idx > index { old_idx - 1 } else { old_idx };
            new_index_to_session.insert(new_idx, session_id);
            new_session_to_index.insert(session_id, new_idx);
        }

        self.index_to_session = new_index_to_session;
        self.session_to_index = new_session_to_index;
    }
}

impl FeagiServer for FeagiWebSocketServerRouter {
    fn poll(&mut self) -> &FeagiEndpointState {
        if matches!(self.current_state, FeagiEndpointState::ActiveWaiting) && !self.has_data {
            self.accept_pending_connections();
            self.process_handshakes(); // Process pending handshakes

            if self.try_receive() {
                self.has_data = true;
                self.current_state = FeagiEndpointState::ActiveHasData;
            }
        }
        &self.current_state
    }

    fn request_start(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::Inactive => {
                let listener = TcpListener::bind(&self.bind_address)
                    .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;
                listener
                    .set_nonblocking(true)
                    .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;

                self.listener = Some(listener);
                self.current_state = FeagiEndpointState::ActiveWaiting;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot start: server is not in Inactive state".to_string(),
            )),
        }
    }

    fn request_stop(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                for state in self.clients.drain(..) {
                    if let HandshakeState::Ready(mut ws) = state {
                        let _ = ws.close(None);
                    }
                }
                self.listener = None;
                self.receive_buffer = None;
                self.current_session = None;
                self.has_data = false;
                self.index_to_session.clear();
                self.session_to_index.clear();
                self.current_state = FeagiEndpointState::Inactive;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot stop: server is not in Active state".to_string(),
            )),
        }
    }

    fn confirm_error_and_close(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::Errored(_) => {
                for state in self.clients.drain(..) {
                    if let HandshakeState::Ready(mut ws) = state {
                        let _ = ws.close(None);
                    }
                }
                self.listener = None;
                self.receive_buffer = None;
                self.current_session = None;
                self.has_data = false;
                self.index_to_session.clear();
                self.session_to_index.clear();
                self.current_state = FeagiEndpointState::Inactive;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot confirm error: server is not in Errored state".to_string(),
            )),
        }
    }

    fn get_protocol(&self) -> TransportProtocolImplementation {
        TransportProtocolImplementation::WebSocket
    }

    fn get_endpoint(&self) -> TransportProtocolEndpoint {
        TransportProtocolEndpoint::WebSocket(
            WebSocketUrl::new(&self.bind_address).expect("bind_address already validated")
        )
    }
}

impl FeagiServerRouter for FeagiWebSocketServerRouter {
    fn consume_retrieved_request(&mut self) -> Result<(AgentID, &[u8]), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveHasData => {
                if self.has_data {
                    if let (Some(ref data), Some(session_id)) =
                        (&self.receive_buffer, self.current_session)
                    {
                        self.has_data = false;
                        self.current_session = None;
                        self.current_state = FeagiEndpointState::ActiveWaiting;
                        Ok((session_id, data.as_slice()))
                    } else {
                        Err(FeagiNetworkError::ReceiveFailed(
                            "No data or session in buffer".to_string(),
                        ))
                    }
                } else {
                    Err(FeagiNetworkError::ReceiveFailed(
                        "No data available".to_string(),
                    ))
                }
            }
            _ => Err(FeagiNetworkError::ReceiveFailed(
                "Cannot consume: no request available".to_string(),
            )),
        }
    }

    fn publish_response(
        &mut self,
        session_id: AgentID,
        message: &[u8],
    ) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                let client_index = *self.session_to_index.get(&session_id).ok_or_else(|| {
                    FeagiNetworkError::SendFailed(format!("Unknown session: {:?}", session_id))
                })?;

                if client_index >= self.clients.len() {
                    return Err(FeagiNetworkError::SendFailed(
                        "Client disconnected".to_string(),
                    ));
                }

                let ws_message = Message::Binary(message.to_vec());
                
                let client = match &mut self.clients[client_index] {
                    HandshakeState::Ready(ws) => ws,
                    _ => return Err(FeagiNetworkError::SendFailed("Client not ready".to_string())),
                };
                
                client
                    .send(ws_message)
                    .map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;
                client
                    .flush()
                    .map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;

                Ok(())
            }
            _ => Err(FeagiNetworkError::SendFailed(
                "Cannot send response: server is not in Active state".to_string(),
            )),
        }
    }

    fn as_boxed_router_properties(&self) -> Box<dyn FeagiServerRouterProperties> {
        Box::new(FeagiWebSocketServerRouterProperties {
            bind_address: self.bind_address.clone(),
        })
    }
}

//endregion