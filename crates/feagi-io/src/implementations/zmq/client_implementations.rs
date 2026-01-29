use zeromq::{DealerSocket, PushSocket, Socket, SocketRecv, SocketSend, SubSocket, ZmqMessage};
use crate::FeagiNetworkError;
use crate::implementations::zmq::shared_functions::validate_zmq_url;
use crate::traits_and_enums::client::client_shared::{FeagiClientConnectionState, FeagiClientConnectionStateChange};
use crate::traits_and_enums::client::{FeagiClient, FeagiClientPusher, FeagiClientRequester, FeagiClientSubscriber};

/// Type alias for the client state change callback.
type StateChangeCallback = Box<dyn Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static>;

fn message_to_single_frame(message: ZmqMessage) -> Result<Vec<u8>, FeagiNetworkError> {
    let mut frames = message.into_vec();
    let frame = frames.pop().ok_or_else(|| {
        FeagiNetworkError::ReceiveFailed("Empty ZMQ message received".to_string())
    })?;
    if !frames.is_empty() {
        return Err(FeagiNetworkError::ReceiveFailed(
            "Unexpected multipart message for single-frame socket".to_string(),
        ));
    }
    Ok(frame.to_vec())
}

//region Subscriber

pub struct FEAGIZMQClientSubscriber {
    server_address: String,
    current_state: FeagiClientConnectionState,
    state_change_callback: StateChangeCallback,
    socket: SubSocket,
    cached_data: Vec<u8>,
}

impl FEAGIZMQClientSubscriber {
    pub fn new(
        server_address: String,
        state_change_callback: StateChangeCallback,
    ) -> Result<Self, FeagiNetworkError> {
        validate_zmq_url(&server_address)?;

        Ok(Self {
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            state_change_callback,
            socket: SubSocket::new(),
            cached_data: Vec::new(),
        })
    }
}

impl FeagiClient for FEAGIZMQClientSubscriber {
    async fn connect(&mut self, host: &str) -> Result<(), FeagiNetworkError> {
        self.socket.subscribe("").await
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        self.socket.connect(host).await
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;

        self.server_address = host.to_string();
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Connected;
        (self.state_change_callback)(FeagiClientConnectionStateChange::new(
            previous,
            self.current_state,
        ));
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), FeagiNetworkError> {
        let socket = std::mem::replace(&mut self.socket, SubSocket::new());
        let _ = socket.close().await;
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Disconnected;
        (self.state_change_callback)(FeagiClientConnectionStateChange::new(
            previous,
            self.current_state,
        ));
        Ok(())
    }

    fn get_current_connection_state(&self) -> FeagiClientConnectionState {
        self.current_state
    }
}

impl FeagiClientSubscriber for FEAGIZMQClientSubscriber {
    async fn get_subscribed_data(&mut self) -> Result<&[u8], FeagiNetworkError> {
        let message = self.socket.recv().await
            .map_err(|e| FeagiNetworkError::ReceiveFailed(e.to_string()))?;
        self.cached_data = message_to_single_frame(message)?;
        Ok(&self.cached_data)
    }
}

//endregion

//region Pusher

pub struct FEAGIZMQClientPusher {
    server_address: String,
    current_state: FeagiClientConnectionState,
    state_change_callback: StateChangeCallback,
    socket: PushSocket,
}

impl FEAGIZMQClientPusher {
    pub fn new(
        server_address: String,
        state_change_callback: StateChangeCallback,
    ) -> Result<Self, FeagiNetworkError> {
        validate_zmq_url(&server_address)?;

        Ok(Self {
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            state_change_callback,
            socket: PushSocket::new(),
        })
    }
}

impl FeagiClient for FEAGIZMQClientPusher {
    async fn connect(&mut self, host: &str) -> Result<(), FeagiNetworkError> {
        self.socket.connect(host).await
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;

        self.server_address = host.to_string();
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Connected;
        (self.state_change_callback)(FeagiClientConnectionStateChange::new(
            previous,
            self.current_state,
        ));
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), FeagiNetworkError> {
        let socket = std::mem::replace(&mut self.socket,PushSocket::new());
        let _ = socket.close().await;
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Disconnected;
        (self.state_change_callback)(FeagiClientConnectionStateChange::new(
            previous,
            self.current_state,
        ));
        Ok(())
    }

    fn get_current_connection_state(&self) -> FeagiClientConnectionState {
        self.current_state
    }
}

impl FeagiClientPusher for FEAGIZMQClientPusher {
    async fn push_data(&mut self, data: &[u8]) -> Result<(), FeagiNetworkError> {
        let message = ZmqMessage::from(data.to_vec());
        self.socket.send(message).await
            .map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;
        Ok(())
    }
}

//endregion

//region Requester (Dealer)

pub struct FEAGIZMQClientRequester {
    server_address: String,
    current_state: FeagiClientConnectionState,
    state_change_callback: StateChangeCallback,
    socket: DealerSocket,
    cached_response_data: Vec<u8>,
}

impl FEAGIZMQClientRequester {
    pub fn new(
        server_address: String,
        state_change_callback: StateChangeCallback,
    ) -> Result<Self, FeagiNetworkError> {
        validate_zmq_url(&server_address)?;

        Ok(Self {
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            state_change_callback,
            socket: DealerSocket::new(),
            cached_response_data: Vec::new(),
        })
    }
}

impl FeagiClient for FEAGIZMQClientRequester {
    async fn connect(&mut self, host: &str) -> Result<(), FeagiNetworkError> {
        self.socket.connect(host).await
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;

        self.server_address = host.to_string();
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Connected;
        (self.state_change_callback)(FeagiClientConnectionStateChange::new(
            previous,
            self.current_state,
        ));
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), FeagiNetworkError> {
        let socket = std::mem::replace(&mut self.socket, DealerSocket::new());
        let _ = socket.close().await;
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Disconnected;
        (self.state_change_callback)(FeagiClientConnectionStateChange::new(
            previous,
            self.current_state,
        ));
        Ok(())
    }

    fn get_current_connection_state(&self) -> FeagiClientConnectionState {
        self.current_state
    }
}

impl FeagiClientRequester for FEAGIZMQClientRequester {
    async fn send_request(&mut self, request: &[u8]) -> Result<(), FeagiNetworkError> {
        let mut message = ZmqMessage::from(request.to_vec());
        message.prepend(&ZmqMessage::from(Vec::new()));
        self.socket.send(message).await
            .map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;
        Ok(())
    }

    async fn get_response(&mut self) -> Result<&[u8], FeagiNetworkError> {
        let message = self.socket.recv().await
            .map_err(|e| FeagiNetworkError::ReceiveFailed(e.to_string()))?;

        let mut frames = message.into_vec();
        if frames.first().map(|frame| frame.is_empty()).unwrap_or(false) {
            frames.remove(0);
        }

        if frames.len() != 1 {
            return Err(FeagiNetworkError::ReceiveFailed(
                "Unexpected multipart response payload".to_string(),
            ));
        }

        self.cached_response_data = frames.remove(0).to_vec();
        Ok(&self.cached_response_data)
    }
}

//endregion
