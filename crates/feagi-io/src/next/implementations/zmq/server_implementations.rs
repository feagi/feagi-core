use std::collections::HashMap;

use crate::next::{FeagiNetworkError, FeagiServerBindState};
use crate::next::implementations::zmq::shared_functions::validate_zmq_url;
use crate::next::traits_and_enums::server::{FeagiServer, FeagiServerPublisher, FeagiServerPuller, FeagiServerRouter};
use crate::next::traits_and_enums::server::server_shared::{ClientId, FeagiServerBindStateChange};

//region Publisher
pub struct FEAGIZMQServerPublisher<S>
where S: Fn(FeagiServerBindStateChange) + Send + Sync + 'static
{
    context_ref: zmq::Context,
    server_bind_address: String,
    current_state: FeagiServerBindState,
    socket: zmq::Socket,
    state_change_callback: S
}


impl<S> FEAGIZMQServerPublisher<S>
where S: Fn(FeagiServerBindStateChange) + Send + Sync + 'static {
    pub fn new(context: &mut zmq::Context, server_bind_address: String, state_change_callback: S) -> Result<Self, FeagiNetworkError> {
        validate_zmq_url(&server_bind_address)?;
        let socket = context.socket(zmq::PUB).map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        Ok(Self {
            context_ref: context.clone(),
            server_bind_address,
            current_state: FeagiServerBindState::Inactive,
            socket,
            state_change_callback
        })
    }
}

impl<S> FeagiServer for FEAGIZMQServerPublisher<S>
where S: Fn(FeagiServerBindStateChange) + Send + Sync + 'static {
    fn start(&mut self) -> Result<(), FeagiNetworkError> {
        // NOTE: there is a period of ~200 ms when this needs to activate!
        self.socket.bind(&self.server_bind_address).map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;
        self.current_state = FeagiServerBindState::Active;
        (self.state_change_callback)(
            FeagiServerBindStateChange::new(
                FeagiServerBindState::Inactive, FeagiServerBindState::Active
            ));
        Ok(())
    }

    fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        self.socket.unbind(&self.server_bind_address).map_err(|e| FeagiNetworkError::CannotUnbind(e.to_string()))?;
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

impl<S> FeagiServerPublisher for FEAGIZMQServerPublisher<S>
where S: Fn(FeagiServerBindStateChange) + Send + Sync + 'static {
    fn poll(&mut self) -> Result<(), FeagiNetworkError> {
        // ZMQ handles connections internally - no-op
        Ok(())
    }

    fn publish(&mut self, buffered_data_to_send: &[u8]) -> Result<(), FeagiNetworkError> {
        self.socket.send(buffered_data_to_send, 0).map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;
        Ok(())
    }
}
//endregion

//region Puller

pub struct FEAGIZMQServerPuller<S>
where S: Fn(FeagiServerBindStateChange) + Send + Sync + 'static
{
    context_ref: zmq::Context,
    server_bind_address: String,
    current_state: FeagiServerBindState,
    socket: zmq::Socket,
    state_change_callback: S,
    cached_data: Vec<u8>,
}

impl<S> FEAGIZMQServerPuller<S>
where S: Fn(FeagiServerBindStateChange) + Send + Sync + 'static
{
    pub fn new(context: &mut zmq::Context, server_bind_address: String, state_change_callback: S)
        -> Result<Self, FeagiNetworkError>
    {
        validate_zmq_url(&server_bind_address)?;
        let socket = context.socket(zmq::PULL).map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        Ok(Self {
            context_ref: context.clone(),
            server_bind_address,
            current_state: FeagiServerBindState::Inactive,
            socket,
            state_change_callback,
            cached_data: Vec::new(),
        })
    }
}

impl<S> FeagiServer for FEAGIZMQServerPuller<S>
where S: Fn(FeagiServerBindStateChange) + Send + Sync + 'static
{
    fn start(&mut self) -> Result<(), FeagiNetworkError> {
        // Set socket to non-blocking mode for try_poll
        self.socket.set_rcvtimeo(0).map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        // NOTE: there is a period of ~200 ms when this needs to activate!
        self.socket.bind(&self.server_bind_address).map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;
        self.current_state = FeagiServerBindState::Active;
        (self.state_change_callback)(
            FeagiServerBindStateChange::new(
                FeagiServerBindState::Inactive, FeagiServerBindState::Active
            ));
        Ok(())
    }

    fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        self.socket.unbind(&self.server_bind_address).map_err(|e| FeagiNetworkError::CannotUnbind(e.to_string()))?;
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

impl<S> FeagiServerPuller for FEAGIZMQServerPuller<S>
where S: Fn(FeagiServerBindStateChange) + Send + Sync + 'static
{
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

pub struct FEAGIZMQServerRouter<S>
where S: Fn(FeagiServerBindStateChange) + Send + Sync + 'static
{
    context_ref: zmq::Context,
    server_bind_address: String,
    current_state: FeagiServerBindState,
    socket: zmq::Socket,
    state_change_callback: S,
    // Bidirectional mapping between ClientId and ZMQ identity
    next_client_id: u64,
    identity_to_id: HashMap<Vec<u8>, u64>,
    id_to_identity: HashMap<u64, Vec<u8>>,
    // Cached request data
    cached_request_data: Vec<u8>,
    last_client_id: Option<ClientId>,
}

impl<S> FEAGIZMQServerRouter<S>
where S: Fn(FeagiServerBindStateChange) + Send + Sync + 'static
{
    pub fn new(context: &mut zmq::Context, server_bind_address: String, state_change_callback: S)
        -> Result<Self, FeagiNetworkError>
    {
        validate_zmq_url(&server_bind_address)?;
        let socket = context.socket(zmq::ROUTER).map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        Ok(Self {
            context_ref: context.clone(),
            server_bind_address,
            current_state: FeagiServerBindState::Inactive,
            socket,
            state_change_callback,
            next_client_id: 1, // Start from 1, reserve 0 for "no client"
            identity_to_id: HashMap::new(),
            id_to_identity: HashMap::new(),
            cached_request_data: Vec::new(),
            last_client_id: None,
        })
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

impl<S> FeagiServer for FEAGIZMQServerRouter<S>
where S: Fn(FeagiServerBindStateChange) + Send + Sync + 'static
{
    fn start(&mut self) -> Result<(), FeagiNetworkError> {
        // Set socket to non-blocking mode for try_poll
        self.socket.set_rcvtimeo(0).map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        // NOTE: there is a period of ~200 ms when this needs to activate!
        self.socket.bind(&self.server_bind_address).map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;
        self.current_state = FeagiServerBindState::Active;
        (self.state_change_callback)(
            FeagiServerBindStateChange::new(
                FeagiServerBindState::Inactive, FeagiServerBindState::Active
            ));
        Ok(())
    }

    fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        self.socket.unbind(&self.server_bind_address).map_err(|e| FeagiNetworkError::CannotUnbind(e.to_string()))?;
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

impl<S> FeagiServerRouter for FEAGIZMQServerRouter<S>
where S: Fn(FeagiServerBindStateChange) + Send + Sync + 'static
{
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