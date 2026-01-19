use crate::next::FeagiNetworkError;
use crate::next::implementations::zmq::shared_functions::validate_zmq_url;
use crate::next::state_enums::FeagiServerBindState;
use crate::next::traits::server::{FeagiServer, FeagiServerPublisher, FeagiServerPuller, FeagiServerRouter};

//region Publisher
pub struct FEAGIZMQServerPublisher {
    context_ref: zmq::Context,
    server_bind_address: String,
    current_state: FeagiServerBindState,
    socket: zmq::Socket
}

impl FEAGIZMQServerPublisher {
    fn new(context: &mut zmq::Context, server_bind_address: String) -> Result<Self, FeagiNetworkError> {
        validate_zmq_url(&server_bind_address)?;
        let socket = context.socket(zmq::PUB).map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        Ok(Self {
            context_ref: context.clone(),
            server_bind_address,
            current_state: FeagiServerBindState::Inactive,
            socket
        })
    }
}

impl FeagiServer for FEAGIZMQServerPublisher {
    fn start(&mut self) -> Result<(), FeagiNetworkError> {
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

impl FeagiServerPublisher for FEAGIZMQServerPublisher {
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
    data_received_callback: Fn(&[u8]) + Send + Sync + 'static
}

impl FEAGIZMQServerPuller {
    fn new<F>(context: &mut zmq::Context, server_bind_address: String, data_received_callback: F)
        -> Result<Self, FeagiNetworkError>
    where
        F: Fn(&[u8]) + Send + Sync + 'static {
    validate_zmq_url(&server_bind_address)?;
        let socket = context.socket(zmq::PULL).map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        Ok(Self {
            context_ref: context.clone(),
            server_bind_address,
            current_state: FeagiServerBindState::Inactive,
            socket,
            data_received_callback
        })
    }
}

impl FeagiServer for FEAGIZMQServerPuller {
    fn start(&mut self) -> Result<(), FeagiNetworkError> {
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
    // Nothing
}

//endregion

//region Router

pub struct FEAGIZMQServerRouter {
    context_ref: zmq::Context,
    server_bind_address: String,
    current_state: FeagiServerBindState,
    socket: zmq::Socket,
    data_process_callback: Fn(&[u8], &mut [u8]) -> Result<(), FeagiNetworkError> + Send + Sync + 'static,
    cache_processed_bytes: Vec<u8>
}

impl FEAGIZMQServerRouter {
    fn new<F>(context: &mut zmq::Context, server_bind_address: String, data_process_callback: F)
        -> Result<Self, FeagiNetworkError>
    where
        F: Fn(&[u8], &mut [u8]) -> Result<(), FeagiNetworkError> + Send + Sync + 'static
    {
        validate_zmq_url(&server_bind_address)?;
        let socket = context.socket(zmq::ROUTER).unwrap();
        Ok(Self {
            context_ref: context.clone(),
            server_bind_address,
            current_state: FeagiServerBindState::Inactive,
            socket,
            data_process_callback,
            cache_processed_bytes: Vec::new()
        })
    }
}

impl FeagiServer for FEAGIZMQServerRouter {
    fn start(&mut self) -> Result<(), FeagiNetworkError> {
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
    fn _received_request(&mut self, request_data: &[u8]) -> Result<(), FeagiNetworkError> {
        self.data_process_callback(request_data, &mut self.cache_processed_bytes)?;
        self.socket.send(&self.cache_processed_bytes, 0).map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;
        Ok(())
    }
}

//endregion