use crate::next::{FeagiNetworkError, FeagiServerBindState};
use crate::next::implementations::zmq::shared_functions::validate_zmq_url;
use crate::next::traits_and_enums::server::{FeagiServer, FeagiServerPublisher, FeagiServerPuller, FeagiServerRouter};
use crate::next::traits_and_enums::server::server_shared::FeagiServerBindStateChange;

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
                FeagiServerBindState::Inactive, FeagiServerBindState::Inactive
            ));
        Ok(())
    }

    fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        self.socket.unbind(&self.server_bind_address).map_err(|e| FeagiNetworkError::CannotUnbind(e.to_string()))?;
        self.current_state = FeagiServerBindState::Inactive;
        Ok(())
    }

    fn get_current_state(&self) -> FeagiServerBindState {
        self.current_state
    }
}

impl<S> FeagiServerPublisher for FEAGIZMQServerPublisher<S>
where S: Fn(FeagiServerBindStateChange) + Send + Sync + 'static {
    fn publish(&mut self, buffered_data_to_send: &[u8]) -> Result<(), FeagiNetworkError> {
        self.socket.send(buffered_data_to_send, 0).map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;
        Ok(())
    }
}
//endregion

//region Puller

pub struct FEAGIZMQServerPuller {
    context_ref: zmq::Context,
    server_bind_address: String,
    current_state: FeagiServerBindState,
    socket: zmq::Socket,
    cached_data: Vec<u8>
}

impl FEAGIZMQServerPuller {
    pub fn new(context: &mut zmq::Context, server_bind_address: String)
        -> Result<Self, FeagiNetworkError>
    {
        validate_zmq_url(&server_bind_address)?;
        let socket = context.socket(zmq::PULL).map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        Ok(Self {
            context_ref: context.clone(),
            server_bind_address,
            current_state: FeagiServerBindState::Inactive,
            socket,
            cached_data: Vec::new()
        })
    }
}

impl FeagiServer for FEAGIZMQServerPuller {
    fn start(&mut self) -> Result<(), FeagiNetworkError> {
        // Set socket to non-blocking mode for try_poll
        self.socket.set_rcvtimeo(0).map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        // NOTE: there is a period of ~200 ms when this needs to activate!
        self.socket.bind(&self.server_bind_address).map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;
        self.current_state = FeagiServerBindState::Active;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        self.socket.unbind(&self.server_bind_address).map_err(|e| FeagiNetworkError::CannotUnbind(e.to_string()))?;
        self.current_state = FeagiServerBindState::Inactive;
        Ok(())
    }

    fn get_current_state(&self) -> FeagiServerBindState {
        self.current_state
    }
}

impl FeagiServerPuller for FEAGIZMQServerPuller {
    fn try_poll(&mut self) -> Result<bool, FeagiNetworkError> {
        match self.socket.recv_bytes(0) {
            Ok(data) => {
                self.cached_data = data; // TODO this does reallocation, we should find a better way of handling this
                Ok(true)
            }
            Err(zmq::Error::EAGAIN) => Ok(false), // No data available (non-blocking)
            Err(e) => Err(FeagiNetworkError::ReceiveFailed(e.to_string()))
        }
    }

    fn get_cached_data(&self) -> &[u8] {
        &self.cached_data
    }
}

//endregion

//region Router

pub struct FEAGIZMQServerRouter {
    context_ref: zmq::Context,
    server_bind_address: String,
    current_state: FeagiServerBindState,
    socket: zmq::Socket,
    cached_client_identity: Vec<u8>,
    cached_request_data: Vec<u8>
}

impl FEAGIZMQServerRouter {
    pub fn new(context: &mut zmq::Context, server_bind_address: String)
        -> Result<Self, FeagiNetworkError>
    {
        validate_zmq_url(&server_bind_address)?;
        let socket = context.socket(zmq::ROUTER).map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        Ok(Self {
            context_ref: context.clone(),
            server_bind_address,
            current_state: FeagiServerBindState::Inactive,
            socket,
            cached_client_identity: Vec::new(),
            cached_request_data: Vec::new()
        })
    }
}

impl FeagiServer for FEAGIZMQServerRouter {
    fn start(&mut self) -> Result<(), FeagiNetworkError> {
        // Set socket to non-blocking mode for try_poll
        self.socket.set_rcvtimeo(0).map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        // NOTE: there is a period of ~200 ms when this needs to activate!
        self.socket.bind(&self.server_bind_address).map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;
        self.current_state = FeagiServerBindState::Active;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        self.socket.unbind(&self.server_bind_address).map_err(|e| FeagiNetworkError::CannotUnbind(e.to_string()))?;
        self.current_state = FeagiServerBindState::Inactive;
        Ok(())
    }

    fn get_current_state(&self) -> FeagiServerBindState {
        self.current_state
    }
}

impl FeagiServerRouter for FEAGIZMQServerRouter {
    fn try_poll(&mut self) -> Result<bool, FeagiNetworkError> {
        // ROUTER receives: [identity, empty_delimiter, request_data]
        // First, try to receive the identity frame
        match self.socket.recv_bytes(0) {
            Ok(identity) => {
                self.cached_client_identity = identity;
            }
            Err(zmq::Error::EAGAIN) => return Ok(false), // No data available
            Err(e) => return Err(FeagiNetworkError::ReceiveFailed(e.to_string()))
        }

        // Receive empty delimiter frame (sent by DEALER)
        if let Err(e) = self.socket.recv_bytes(0) {
            return Err(FeagiNetworkError::ReceiveFailed(format!("Failed to receive delimiter: {}", e)));
        }

        // Receive actual request data
        match self.socket.recv_bytes(0) {
            Ok(data) => {
                self.cached_request_data = data;
                Ok(true)
            }
            Err(e) => Err(FeagiNetworkError::ReceiveFailed(e.to_string()))
        }
    }

    fn get_request_data(&self) -> &[u8] {
        &self.cached_request_data
    }

    fn send_response(&mut self, response: &[u8]) -> Result<(), FeagiNetworkError> {
        // Send response: [identity, empty_delimiter, response_data]
        self.socket.send(&self.cached_client_identity, zmq::SNDMORE)
            .map_err(|e| FeagiNetworkError::SendFailed(format!("Failed to send identity: {}", e)))?;
        self.socket.send(zmq::Message::new(), zmq::SNDMORE)
            .map_err(|e| FeagiNetworkError::SendFailed(format!("Failed to send delimiter: {}", e)))?;
        self.socket.send(response, 0)
            .map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;
        Ok(())
    }
}

//endregion