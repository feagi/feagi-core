use crate::next::FeagiNetworkError;
use crate::next::implementations::zmq::shared_functions::validate_zmq_url;
use crate::next::state_enums::FeagiClientConnectionState;
use crate::next::traits::client::{FeagiClient, FeagiClientSubscriber, FeagiClientPusher, FeagiClientRequester};

//region Subscriber

pub struct FEAGIZMQClientSubscriber {
    context_ref: zmq::Context,
    server_address: String,
    current_state: FeagiClientConnectionState,
    socket: zmq::Socket,
    data_received_callback: fn(&[u8])
}

impl FEAGIZMQClientSubscriber {
    pub fn new(context: &mut zmq::Context, server_address: String, data_received_callback: fn(&[u8]))
        -> Result<Self, FeagiNetworkError>
    {
        validate_zmq_url(&server_address)?;
        let socket = context.socket(zmq::SUB).map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        // Subscribe to all messages by default
        socket.set_subscribe(b"").map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        Ok(Self {
            context_ref: context.clone(),
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            socket,
            data_received_callback
        })
    }
}

impl FeagiClient for FEAGIZMQClientSubscriber {
    fn connect(&self, host: String) {
        // TODO: Update current_state and handle errors properly
        let _ = self.socket.connect(&host);
    }

    fn disconnect(&self) -> Result<(), FeagiNetworkError> {
        self.socket.disconnect(&self.server_address)
            .map_err(|e| FeagiNetworkError::CannotDisconnect(e.to_string()))?;
        Ok(())
    }

    fn get_current_connection_state(&self) -> FeagiClientConnectionState {
        self.current_state
    }

    fn register_connection_state_changes<F>(&self, _on_state_change: F)
    where
        F: Fn((FeagiClientConnectionState, FeagiClientConnectionState)) + Send + Sync + 'static
    {
        // TODO: Implement state change notifications
    }
}

impl FeagiClientSubscriber for FEAGIZMQClientSubscriber {
    // No additional methods - callback provided at construction
}

//endregion

//region Pusher

pub struct FEAGIZMQClientPusher {
    context_ref: zmq::Context,
    server_address: String,
    current_state: FeagiClientConnectionState,
    socket: zmq::Socket
}

impl FEAGIZMQClientPusher {
    pub fn new(context: &mut zmq::Context, server_address: String) -> Result<Self, FeagiNetworkError> {
        validate_zmq_url(&server_address)?;
        let socket = context.socket(zmq::PUSH).map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        Ok(Self {
            context_ref: context.clone(),
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            socket
        })
    }
}

impl FeagiClient for FEAGIZMQClientPusher {
    fn connect(&self, host: String) {
        // TODO: Update current_state and handle errors properly
        let _ = self.socket.connect(&host);
    }

    fn disconnect(&self) -> Result<(), FeagiNetworkError> {
        self.socket.disconnect(&self.server_address)
            .map_err(|e| FeagiNetworkError::CannotDisconnect(e.to_string()))?;
        Ok(())
    }

    fn get_current_connection_state(&self) -> FeagiClientConnectionState {
        self.current_state
    }

    fn register_connection_state_changes<F>(&self, _on_state_change: F)
    where
        F: Fn((FeagiClientConnectionState, FeagiClientConnectionState)) + Send + Sync + 'static
    {
        // TODO: Implement state change notifications
    }
}

impl FeagiClientPusher for FEAGIZMQClientPusher {
    fn push_data(&self, data: &[u8]) {
        // TODO: Return Result in state changes if theres an error
        let _ = self.socket.send(data, 0);
    }
}

//endregion

//region Requester (Dealer)

pub struct FEAGIZMQClientRequester {
    context_ref: zmq::Context,
    server_address: String,
    current_state: FeagiClientConnectionState,
    socket: zmq::Socket
}

impl FEAGIZMQClientRequester {
    pub fn new(context: &mut zmq::Context, server_address: String) -> Result<Self, FeagiNetworkError> {
        validate_zmq_url(&server_address)?;
        let socket = context.socket(zmq::DEALER).map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        Ok(Self {
            context_ref: context.clone(),
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            socket
        })
    }
}

impl FeagiClient for FEAGIZMQClientRequester {
    fn connect(&self, host: String) {
        // TODO: Update current_state and handle errors properly
        let _ = self.socket.connect(&host);
    }

    fn disconnect(&self) -> Result<(), FeagiNetworkError> {
        self.socket.disconnect(&self.server_address)
            .map_err(|e| FeagiNetworkError::CannotDisconnect(e.to_string()))?;
        Ok(())
    }

    fn get_current_connection_state(&self) -> FeagiClientConnectionState {
        self.current_state
    }

    fn register_connection_state_changes<F>(&self, _on_state_change: F)
    where
        F: Fn((FeagiClientConnectionState, FeagiClientConnectionState)) + Send + Sync + 'static
    {
        // TODO: Implement state change notifications
    }
}

impl FeagiClientRequester for FEAGIZMQClientRequester {
    fn send_request_and_process_response<F>(&self, request: &[u8], on_response_received: F)
    where
        F: Fn(&[u8]) -> &[u8] + Send + Sync + 'static
    {
        // Send request
        if self.socket.send(request, 0).is_err() {
            return;
        }
        // Receive response (blocking) // TODO not blocking!
        if let Ok(response) = self.socket.recv_bytes(0) {
            on_response_received(&response);
        }
    }
}

//endregion
