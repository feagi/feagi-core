use std::collections::HashMap;

use crate::io_api::{FeagiNetworkError, FeagiServerBindState};
use crate::io_api::implementations::zmq::shared_functions::validate_zmq_url;
use crate::io_api::traits_and_enums::server::{
    FeagiServer, FeagiServerPublisher, FeagiServerPuller, FeagiServerRouter,
    FeagiServerPublisherProperties, FeagiServerPullerProperties, FeagiServerRouterProperties
};
use crate::io_api::traits_and_enums::server::server_shared::{ClientId, FeagiServerBindStateChange};

/// Type alias for the server state change callback.
type StateChangeCallback = Box<dyn Fn(FeagiServerBindStateChange) + Send + Sync + 'static>;

//region Publisher
pub struct FEAGIZMQServerPublisher {
    server_bind_address: String,
    current_state: FeagiServerBindState,
    socket: zmq::Socket,
    state_change_callback: StateChangeCallback,
    // Configuration options (applied on start)
    linger: i32,
    sndhwm: i32,
}

impl FEAGIZMQServerPublisher {
    const DEFAULT_LINGER: i32 = 0;
    const DEFAULT_SNDHWM: i32 = 1000;
    
    pub fn new(server_bind_address: String, state_change_callback: StateChangeCallback) -> Result<Self, FeagiNetworkError> {
        validate_zmq_url(&server_bind_address)?;
        let context = zmq::Context::new();
        let socket = context.socket(zmq::PUB)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        Ok(Self {
            server_bind_address,
            current_state: FeagiServerBindState::Inactive,
            socket,
            state_change_callback,
            linger: Self::DEFAULT_LINGER,
            sndhwm: Self::DEFAULT_SNDHWM,
        })
    }
    
    /// Set the linger period for socket shutdown (milliseconds).
    /// Returns error if socket is already running.
    pub fn set_linger(&mut self, linger: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiServerBindState::Active {
            return Err(FeagiNetworkError::GeneralFailure("Cannot change configuration while socket is active".to_string()));
        }
        self.linger = linger;
        Ok(())
    }
    
    /// Set the send high water mark (message queue size).
    /// Returns error if socket is already running.
    pub fn set_sndhwm(&mut self, sndhwm: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiServerBindState::Active {
            return Err(FeagiNetworkError::GeneralFailure("Cannot change configuration while socket is active".to_string()));
        }
        self.sndhwm = sndhwm;
        Ok(())
    }
}

impl FeagiServer for FEAGIZMQServerPublisher {
    fn start(&mut self) -> Result<(), FeagiNetworkError> {
        // Apply configuration options
        self.socket.set_linger(self.linger)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        self.socket.set_sndhwm(self.sndhwm)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        // NOTE: there is a period of ~200 ms when this needs to activate!
        self.socket.bind(&self.server_bind_address)
            .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;
        self.current_state = FeagiServerBindState::Active;
        (self.state_change_callback)(
            FeagiServerBindStateChange::new(
                FeagiServerBindState::Inactive, FeagiServerBindState::Active
            ));
        Ok(())
    }

    fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        self.socket.unbind(&self.server_bind_address)
            .map_err(|e| FeagiNetworkError::CannotUnbind(e.to_string()))?;
        self.current_state = FeagiServerBindState::Inactive;
        (self.state_change_callback)(
            FeagiServerBindStateChange::new(
                FeagiServerBindState::Active, FeagiServerBindState::Inactive
            ));
        Ok(())
    }

    fn get_current_state(&self) -> FeagiServerBindState {
        self.current_state
    }
}

impl FeagiServerPublisher for FEAGIZMQServerPublisher {
    fn poll(&mut self) -> Result<(), FeagiNetworkError> {
        // ZMQ handles connections internally - no-op
        Ok(())
    }

    fn publish(&mut self, buffered_data_to_send: &[u8]) -> Result<(), FeagiNetworkError> {
        self.socket.send(buffered_data_to_send, 0)
            .map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;
        Ok(())
    }
}
//endregion

//region Puller

pub struct FEAGIZMQServerPuller {
    server_bind_address: String,
    current_state: FeagiServerBindState,
    socket: zmq::Socket,
    state_change_callback: StateChangeCallback,
    cached_data: Vec<u8>,
    // Configuration options (applied on start)
    linger: i32,
    rcvhwm: i32,
    immediate: bool,
}

impl FEAGIZMQServerPuller {
    const DEFAULT_LINGER: i32 = 0;
    const DEFAULT_RCVHWM: i32 = 1000;
    const DEFAULT_IMMEDIATE: bool = false;
    
    pub fn new(server_bind_address: String, state_change_callback: StateChangeCallback)
        -> Result<Self, FeagiNetworkError>
    {
        validate_zmq_url(&server_bind_address)?;
        let context = zmq::Context::new();
        let socket = context.socket(zmq::PULL)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        Ok(Self {
            server_bind_address,
            current_state: FeagiServerBindState::Inactive,
            socket,
            state_change_callback,
            cached_data: Vec::new(),
            linger: Self::DEFAULT_LINGER,
            rcvhwm: Self::DEFAULT_RCVHWM,
            immediate: Self::DEFAULT_IMMEDIATE,
        })
    }
    
    /// Set the linger period for socket shutdown (milliseconds).
    /// Returns error if socket is already running.
    pub fn set_linger(&mut self, linger: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiServerBindState::Active {
            return Err(FeagiNetworkError::GeneralFailure("Cannot change configuration while socket is active".to_string()));
        }
        self.linger = linger;
        Ok(())
    }
    
    /// Set the receive high water mark (message queue size).
    /// Returns error if socket is already running.
    pub fn set_rcvhwm(&mut self, rcvhwm: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiServerBindState::Active {
            return Err(FeagiNetworkError::GeneralFailure("Cannot change configuration while socket is active".to_string()));
        }
        self.rcvhwm = rcvhwm;
        Ok(())
    }
    
    /// Set immediate mode (only queue messages to completed connections).
    /// Returns error if socket is already running.
    pub fn set_immediate(&mut self, immediate: bool) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiServerBindState::Active {
            return Err(FeagiNetworkError::GeneralFailure("Cannot change configuration while socket is active".to_string()));
        }
        self.immediate = immediate;
        Ok(())
    }
}

impl FeagiServer for FEAGIZMQServerPuller {
    fn start(&mut self) -> Result<(), FeagiNetworkError> {
        // Apply configuration options
        self.socket.set_linger(self.linger)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        self.socket.set_rcvhwm(self.rcvhwm)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        self.socket.set_immediate(self.immediate)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        // Set socket to non-blocking mode for try_poll
        self.socket.set_rcvtimeo(0)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        // NOTE: there is a period of ~200 ms when this needs to activate!
        self.socket.bind(&self.server_bind_address)
            .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;
        self.current_state = FeagiServerBindState::Active;
        (self.state_change_callback)(
            FeagiServerBindStateChange::new(
                FeagiServerBindState::Inactive, FeagiServerBindState::Active
            ));
        Ok(())
    }

    fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        self.socket.unbind(&self.server_bind_address)
            .map_err(|e| FeagiNetworkError::CannotUnbind(e.to_string()))?;
        self.current_state = FeagiServerBindState::Inactive;
        (self.state_change_callback)(
            FeagiServerBindStateChange::new(
                FeagiServerBindState::Active, FeagiServerBindState::Inactive
            ));
        Ok(())
    }

    fn get_current_state(&self) -> FeagiServerBindState {
        self.current_state
    }
}

impl FeagiServerPuller for FEAGIZMQServerPuller {
    fn try_poll_receive(&mut self) -> Result<Option<&[u8]>, FeagiNetworkError> {
        match self.socket.recv_bytes(0) {
            Ok(data) => {
                self.cached_data = data;
                Ok(Some(&self.cached_data))
            }
            Err(zmq::Error::EAGAIN) => Ok(None), // No data available (non-blocking)
            Err(e) => Err(FeagiNetworkError::ReceiveFailed(e.to_string()))
        }
    }
}

//endregion

//region Router

pub struct FEAGIZMQServerRouter {
    server_bind_address: String,
    current_state: FeagiServerBindState,
    socket: zmq::Socket,
    state_change_callback: StateChangeCallback,
    // Bidirectional mapping between ClientId and ZMQ identity
    next_client_id: u64,
    identity_to_id: HashMap<Vec<u8>, u64>,
    id_to_identity: HashMap<u64, Vec<u8>>,
    // Cached request data
    cached_request_data: Vec<u8>,
    last_client_id: Option<ClientId>,
    // Configuration options (applied on start)
    linger: i32,
    rcvhwm: i32,
    sndhwm: i32,
    immediate: bool,
}

impl FEAGIZMQServerRouter {
    const DEFAULT_LINGER: i32 = 0;
    const DEFAULT_RCVHWM: i32 = 1000;
    const DEFAULT_SNDHWM: i32 = 1000;
    const DEFAULT_IMMEDIATE: bool = false;
    
    pub fn new(server_bind_address: String, state_change_callback: StateChangeCallback)
        -> Result<Self, FeagiNetworkError>
    {
        validate_zmq_url(&server_bind_address)?;
        let context = zmq::Context::new();
        let socket = context.socket(zmq::ROUTER)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        Ok(Self {
            server_bind_address,
            current_state: FeagiServerBindState::Inactive,
            socket,
            state_change_callback,
            next_client_id: 1, // Start from 1, reserve 0 for "no client"
            identity_to_id: HashMap::new(),
            id_to_identity: HashMap::new(),
            cached_request_data: Vec::new(),
            last_client_id: None,
            linger: Self::DEFAULT_LINGER,
            rcvhwm: Self::DEFAULT_RCVHWM,
            sndhwm: Self::DEFAULT_SNDHWM,
            immediate: Self::DEFAULT_IMMEDIATE,
        })
    }
    
    /// Set the linger period for socket shutdown (milliseconds).
    /// Returns error if socket is already running.
    pub fn set_linger(&mut self, linger: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiServerBindState::Active {
            return Err(FeagiNetworkError::GeneralFailure("Cannot change configuration while socket is active".to_string()));
        }
        self.linger = linger;
        Ok(())
    }
    
    /// Set the receive high water mark (message queue size).
    /// Returns error if socket is already running.
    pub fn set_rcvhwm(&mut self, rcvhwm: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiServerBindState::Active {
            return Err(FeagiNetworkError::GeneralFailure("Cannot change configuration while socket is active".to_string()));
        }
        self.rcvhwm = rcvhwm;
        Ok(())
    }
    
    /// Set the send high water mark (message queue size).
    /// Returns error if socket is already running.
    pub fn set_sndhwm(&mut self, sndhwm: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiServerBindState::Active {
            return Err(FeagiNetworkError::GeneralFailure("Cannot change configuration while socket is active".to_string()));
        }
        self.sndhwm = sndhwm;
        Ok(())
    }
    
    /// Set immediate mode (only queue messages to completed connections).
    /// Returns error if socket is already running.
    pub fn set_immediate(&mut self, immediate: bool) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiServerBindState::Active {
            return Err(FeagiNetworkError::GeneralFailure("Cannot change configuration while socket is active".to_string()));
        }
        self.immediate = immediate;
        Ok(())
    }

    /// Get or create a ClientId for the given ZMQ identity
    fn get_or_create_client_id(&mut self, identity: Vec<u8>) -> ClientId {
        if let Some(&id) = self.identity_to_id.get(&identity) {
            ClientId(id)
        } else {
            let id = self.next_client_id;
            self.next_client_id += 1;
            self.identity_to_id.insert(identity.clone(), id);
            self.id_to_identity.insert(id, identity);
            ClientId(id)
        }
    }
}

impl FeagiServer for FEAGIZMQServerRouter {
    fn start(&mut self) -> Result<(), FeagiNetworkError> {
        // Apply configuration options
        self.socket.set_linger(self.linger)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        self.socket.set_rcvhwm(self.rcvhwm)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        self.socket.set_sndhwm(self.sndhwm)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        self.socket.set_immediate(self.immediate)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        // Set socket to non-blocking mode for try_poll
        self.socket.set_rcvtimeo(0)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        // NOTE: there is a period of ~200 ms when this needs to activate!
        self.socket.bind(&self.server_bind_address)
            .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;
        self.current_state = FeagiServerBindState::Active;
        (self.state_change_callback)(
            FeagiServerBindStateChange::new(
                FeagiServerBindState::Inactive, FeagiServerBindState::Active
            ));
        Ok(())
    }

    fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        self.socket.unbind(&self.server_bind_address)
            .map_err(|e| FeagiNetworkError::CannotUnbind(e.to_string()))?;
        // Clear client mappings when stopping
        self.identity_to_id.clear();
        self.id_to_identity.clear();
        self.next_client_id = 1;
        self.last_client_id = None;
        
        self.current_state = FeagiServerBindState::Inactive;
        (self.state_change_callback)(
            FeagiServerBindStateChange::new(
                FeagiServerBindState::Active, FeagiServerBindState::Inactive
            ));

        Ok(())
    }

    fn get_current_state(&self) -> FeagiServerBindState {
        self.current_state
    }
}

impl FeagiServerRouter for FEAGIZMQServerRouter {
    fn try_poll_receive(&mut self) -> Result<Option<(ClientId, &[u8])>, FeagiNetworkError> {
        // ROUTER receives: [identity, empty_delimiter, request_data]
        // First, try to receive the identity frame
        let identity = match self.socket.recv_bytes(0) {
            Ok(identity) => identity,
            Err(zmq::Error::EAGAIN) => return Ok(None), // No data available
            Err(e) => return Err(FeagiNetworkError::ReceiveFailed(e.to_string()))
        };

        // Receive empty delimiter frame (sent by DEALER)
        if let Err(e) = self.socket.recv_bytes(0) {
            return Err(FeagiNetworkError::ReceiveFailed(format!("Failed to receive delimiter: {}", e)));
        }

        // Receive actual request data
        match self.socket.recv_bytes(0) {
            Ok(data) => {
                let client_id = self.get_or_create_client_id(identity);
                self.last_client_id = Some(client_id);
                self.cached_request_data = data;
                Ok(Some((client_id, &self.cached_request_data)))
            }
            Err(e) => Err(FeagiNetworkError::ReceiveFailed(e.to_string()))
        }
    }

    fn send_response(&mut self, client: ClientId, response: &[u8]) -> Result<(), FeagiNetworkError> {
        // Look up the ZMQ identity for this ClientId
        let identity = self.id_to_identity.get(&client.0)
            .ok_or_else(|| FeagiNetworkError::SendFailed(format!("Unknown client ID: {:?}", client)))?;

        // Send response: [identity, empty_delimiter, response_data]
        self.socket.send(identity.as_slice(), zmq::SNDMORE)
            .map_err(|e| FeagiNetworkError::SendFailed(format!("Failed to send identity: {}", e)))?;
        self.socket.send(zmq::Message::new(), zmq::SNDMORE)
            .map_err(|e| FeagiNetworkError::SendFailed(format!("Failed to send delimiter: {}", e)))?;
        self.socket.send(response, 0)
            .map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;
        Ok(())
    }
}

//endregion

//region Properties

//region Publisher Properties

/// Properties for configuring and building a ZMQ Server Publisher.
pub struct FEAGIZMQServerPublisherProperties {
    server_bind_address: String,
    linger: i32,
    sndhwm: i32,
}

impl FEAGIZMQServerPublisherProperties {
    const DEFAULT_LINGER: i32 = 0;
    const DEFAULT_SNDHWM: i32 = 1000;
    
    /// Create new properties with the given bind address.
    pub fn new(server_bind_address: String) -> Self {
        Self {
            server_bind_address,
            linger: Self::DEFAULT_LINGER,
            sndhwm: Self::DEFAULT_SNDHWM,
        }
    }
    
    /// Set the linger period for socket shutdown (milliseconds).
    pub fn set_linger(&mut self, linger: i32) -> &mut Self {
        self.linger = linger;
        self
    }
    
    /// Set the send high water mark (message queue size).
    pub fn set_sndhwm(&mut self, sndhwm: i32) -> &mut Self {
        self.sndhwm = sndhwm;
        self
    }
}

impl FeagiServerPublisherProperties for FEAGIZMQServerPublisherProperties {
    fn build(self: Box<Self>, state_change_callback: StateChangeCallback) -> Box<dyn FeagiServerPublisher> {
        let mut publisher = FEAGIZMQServerPublisher::new(
            self.server_bind_address,
            state_change_callback,
        ).expect("Failed to create ZMQ publisher");
        
        let _ = publisher.set_linger(self.linger);
        let _ = publisher.set_sndhwm(self.sndhwm);
        
        Box::new(publisher)
    }
}

//endregion

//region Puller Properties

/// Properties for configuring and building a ZMQ Server Puller.
pub struct FEAGIZMQServerPullerProperties {
    server_bind_address: String,
    linger: i32,
    rcvhwm: i32,
    immediate: bool,
}

impl FEAGIZMQServerPullerProperties {
    const DEFAULT_LINGER: i32 = 0;
    const DEFAULT_RCVHWM: i32 = 1000;
    const DEFAULT_IMMEDIATE: bool = false;
    
    /// Create new properties with the given bind address.
    pub fn new(server_bind_address: String) -> Self {
        Self {
            server_bind_address,
            linger: Self::DEFAULT_LINGER,
            rcvhwm: Self::DEFAULT_RCVHWM,
            immediate: Self::DEFAULT_IMMEDIATE,
        }
    }
    
    /// Set the linger period for socket shutdown (milliseconds).
    pub fn set_linger(&mut self, linger: i32) -> &mut Self {
        self.linger = linger;
        self
    }
    
    /// Set the receive high water mark (message queue size).
    pub fn set_rcvhwm(&mut self, rcvhwm: i32) -> &mut Self {
        self.rcvhwm = rcvhwm;
        self
    }
    
    /// Set immediate mode (only queue messages to completed connections).
    pub fn set_immediate(&mut self, immediate: bool) -> &mut Self {
        self.immediate = immediate;
        self
    }
}

impl FeagiServerPullerProperties for FEAGIZMQServerPullerProperties {
    fn build(self: Box<Self>, state_change_callback: StateChangeCallback) -> Box<dyn FeagiServerPuller> {
        let mut puller = FEAGIZMQServerPuller::new(
            self.server_bind_address,
            state_change_callback,
        ).expect("Failed to create ZMQ puller");
        
        let _ = puller.set_linger(self.linger);
        let _ = puller.set_rcvhwm(self.rcvhwm);
        let _ = puller.set_immediate(self.immediate);
        
        Box::new(puller)
    }
}

//endregion

//region Router Properties

/// Properties for configuring and building a ZMQ Server Router.
pub struct FEAGIZMQServerRouterProperties {
    server_bind_address: String,
    linger: i32,
    rcvhwm: i32,
    sndhwm: i32,
    immediate: bool,
}

impl FEAGIZMQServerRouterProperties {
    const DEFAULT_LINGER: i32 = 0;
    const DEFAULT_RCVHWM: i32 = 1000;
    const DEFAULT_SNDHWM: i32 = 1000;
    const DEFAULT_IMMEDIATE: bool = false;
    
    /// Create new properties with the given bind address.
    pub fn new(server_bind_address: String) -> Self {
        Self {
            server_bind_address,
            linger: Self::DEFAULT_LINGER,
            rcvhwm: Self::DEFAULT_RCVHWM,
            sndhwm: Self::DEFAULT_SNDHWM,
            immediate: Self::DEFAULT_IMMEDIATE,
        }
    }
    
    /// Set the linger period for socket shutdown (milliseconds).
    pub fn set_linger(&mut self, linger: i32) -> &mut Self {
        self.linger = linger;
        self
    }
    
    /// Set the receive high water mark (message queue size).
    pub fn set_rcvhwm(&mut self, rcvhwm: i32) -> &mut Self {
        self.rcvhwm = rcvhwm;
        self
    }
    
    /// Set the send high water mark (message queue size).
    pub fn set_sndhwm(&mut self, sndhwm: i32) -> &mut Self {
        self.sndhwm = sndhwm;
        self
    }
    
    /// Set immediate mode (only queue messages to completed connections).
    pub fn set_immediate(&mut self, immediate: bool) -> &mut Self {
        self.immediate = immediate;
        self
    }
}

impl FeagiServerRouterProperties for FEAGIZMQServerRouterProperties {
    fn build(self: Box<Self>, state_change_callback: StateChangeCallback) -> Box<dyn FeagiServerRouter> {
        let mut router = FEAGIZMQServerRouter::new(
            self.server_bind_address,
            state_change_callback,
        ).expect("Failed to create ZMQ router");
        
        let _ = router.set_linger(self.linger);
        let _ = router.set_rcvhwm(self.rcvhwm);
        let _ = router.set_sndhwm(self.sndhwm);
        let _ = router.set_immediate(self.immediate);
        
        Box::new(router)
    }
}

//endregion

//endregion
