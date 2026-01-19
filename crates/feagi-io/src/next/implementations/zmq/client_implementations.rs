use crate::next::FeagiNetworkError;
use crate::next::state_enums::FeagiClientConnectionState;
use crate::next::traits::client::FeagiClient;
use crate::next::traits::client::FeagiClientSubscriber;
use crate::next::traits::client::FeagiClientPusher;
use crate::next::traits::client::FeagiClientRequester;

//region Subscriber

pub struct FEAGIZMQClientSubscriber {
    context_ref: zmq::Context,
    server_address: String,
    current_state: FeagiClientConnectionState,
    socket: zmq::Socket
}

impl FEAGIZMQClientSubscriber {
    fn new(context: &mut zmq::Context, server_address: String) -> Self {
        // TODO: inspect server_address for validity for ZMQ
        let socket = context.socket(zmq::SUB).unwrap();
        Self {
            context_ref: context.clone(),
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            socket
        }
    }
}

impl FeagiClient for FEAGIZMQClientSubscriber {
    fn connect(&self, host: String) {
        // TODO: Handle connection errors properly
        self.socket.connect(&host).unwrap();
    }

    fn disconnect(&self) -> Result<(), FeagiNetworkError> {
        self.socket.disconnect(&self.server_address)?;
        Ok(())
    }

    fn get_current_connection_state(&self) -> FeagiClientConnectionState {
        self.current_state
    }

    fn register_connection_state_changes<F>(&self, on_state_change: F)
    where
        F: Fn((FeagiClientConnectionState, FeagiClientConnectionState)) + Send + Sync + 'static
    {
        todo!()
    }
}

impl FeagiClientSubscriber for FEAGIZMQClientSubscriber {
    fn set_callback_for_data_received<F>(&self, on_data_received: F)
    where
        F: Fn(&[u8]) -> &[u8] + Send + Sync + 'static
    {
        todo!()
    }
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
    fn new(context: &mut zmq::Context, server_address: String) -> Self {
        // TODO: inspect server_address for validity for ZMQ
        let socket = context.socket(zmq::PUSH).unwrap();
        Self {
            context_ref: context.clone(),
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            socket
        }
    }
}

impl FeagiClient for FEAGIZMQClientPusher {
    fn connect(&self, host: String) {
        // TODO: Handle connection errors properly
        self.socket.connect(&host).unwrap();
    }

    fn disconnect(&self) -> Result<(), FeagiNetworkError> {
        self.socket.disconnect(&self.server_address)?;
        Ok(())
    }

    fn get_current_connection_state(&self) -> FeagiClientConnectionState {
        self.current_state
    }

    fn register_connection_state_changes<F>(&self, on_state_change: F)
    where
        F: Fn((FeagiClientConnectionState, FeagiClientConnectionState)) + Send + Sync + 'static
    {
        todo!()
    }
}

impl FeagiClientPusher for FEAGIZMQClientPusher {
    fn push_data(&self, data: &[u8]) {
        self.socket.send(data, 0).unwrap();
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
    fn new(context: &mut zmq::Context, server_address: String) -> Self {
        // TODO: inspect server_address for validity for ZMQ
        let socket = context.socket(zmq::DEALER).unwrap();
        Self {
            context_ref: context.clone(),
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            socket
        }
    }
}

impl FeagiClient for FEAGIZMQClientRequester {
    fn connect(&self, host: String) {
        // TODO: Handle connection errors properly
        self.socket.connect(&host).unwrap();
    }

    fn disconnect(&self) -> Result<(), FeagiNetworkError> {
        self.socket.disconnect(&self.server_address)?;
        Ok(())
    }

    fn get_current_connection_state(&self) -> FeagiClientConnectionState {
        self.current_state
    }

    fn register_connection_state_changes<F>(&self, on_state_change: F)
    where
        F: Fn((FeagiClientConnectionState, FeagiClientConnectionState)) + Send + Sync + 'static
    {
        todo!()
    }
}

impl FeagiClientRequester for FEAGIZMQClientRequester {
    fn send_request_and_process_response<F>(&self, request: &[u8], on_response_received: F)
    where
        F: Fn(&[u8]) -> &[u8] + Send + Sync + 'static
    {
        todo!()
    }
}

//endregion
