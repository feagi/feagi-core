use std::collections::HashMap;

use async_trait::async_trait;
use feagi_serialization::SessionID;
use zeromq::{Endpoint, PubSocket, PullSocket, RouterSocket, Socket, SocketRecv, SocketSend, ZmqMessage};

use crate::FeagiNetworkError;
use crate::implementations::zmq::shared_functions::validate_zmq_url;
use crate::traits_and_enums::server::server_shared::{FeagiServerBindState, FeagiServerBindStateChange};
use crate::traits_and_enums::server::{FeagiServer, FeagiServerPublisher, FeagiServerPuller, FeagiServerRouter};

/// Type alias for the server state change callback.
type StateChangeCallback = Box<dyn Fn(FeagiServerBindStateChange) + Send + Sync + 'static>;

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

//region Publisher

pub struct FEAGIZMQServerPublisher {
    server_bind_address: String,
    current_state: FeagiServerBindState,
    state_change_callback: StateChangeCallback,
    socket: PubSocket,
    bound_endpoint: Option<Endpoint>,
}

impl FEAGIZMQServerPublisher {
    pub fn new(
        server_bind_address: String,
        state_change_callback: StateChangeCallback,
    ) -> Result<Self, FeagiNetworkError> {
        validate_zmq_url(&server_bind_address)?;

        Ok(Self {
            server_bind_address,
            current_state: FeagiServerBindState::Inactive,
            state_change_callback,
            socket: PubSocket::new(),
            bound_endpoint: None,
        })
    }
}

#[async_trait]
impl FeagiServer for FEAGIZMQServerPublisher {
    async fn start(&mut self) -> Result<(), FeagiNetworkError> {
        let endpoint = self
            .socket
            .bind(&self.server_bind_address)
            .await
            .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;
        self.bound_endpoint = Some(endpoint);
        self.current_state = FeagiServerBindState::Active;
        (self.state_change_callback)(FeagiServerBindStateChange::new(
            FeagiServerBindState::Inactive,
            FeagiServerBindState::Active,
        ));
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        if let Some(endpoint) = self.bound_endpoint.take() {
            self.socket
                .unbind(endpoint)
                .await
                .map_err(|e| FeagiNetworkError::CannotUnbind(e.to_string()))?;
        }
        self.current_state = FeagiServerBindState::Inactive;
        (self.state_change_callback)(FeagiServerBindStateChange::new(
            FeagiServerBindState::Active,
            FeagiServerBindState::Inactive,
        ));
        Ok(())
    }

    fn get_current_state(&self) -> FeagiServerBindState {
        self.current_state
    }
}

#[async_trait]
impl FeagiServerPublisher for FEAGIZMQServerPublisher {
    async fn poll(&mut self) -> Result<(), FeagiNetworkError> {
        // ZMQ handles connections internally - no-op
        Ok(())
    }

    async fn publish(&mut self, buffered_data_to_send: &[u8]) -> Result<(), FeagiNetworkError> {
        let message = ZmqMessage::from(buffered_data_to_send.to_vec());
        self.socket
            .send(message)
            .await
            .map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;
        Ok(())
    }
}

//endregion

//region Puller

pub struct FEAGIZMQServerPuller {
    server_bind_address: String,
    current_state: FeagiServerBindState,
    state_change_callback: StateChangeCallback,
    socket: PullSocket,
    bound_endpoint: Option<Endpoint>,
}

impl FEAGIZMQServerPuller {
    pub fn new(
        server_bind_address: String,
        state_change_callback: StateChangeCallback,
    ) -> Result<Self, FeagiNetworkError> {
        validate_zmq_url(&server_bind_address)?;

        Ok(Self {
            server_bind_address,
            current_state: FeagiServerBindState::Inactive,
            state_change_callback,
            socket: PullSocket::new(),
            bound_endpoint: None,
        })
    }
}

#[async_trait]
impl FeagiServer for FEAGIZMQServerPuller {
    async fn start(&mut self) -> Result<(), FeagiNetworkError> {
        let endpoint = self
            .socket
            .bind(&self.server_bind_address)
            .await
            .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;
        self.bound_endpoint = Some(endpoint);
        self.current_state = FeagiServerBindState::Active;
        (self.state_change_callback)(FeagiServerBindStateChange::new(
            FeagiServerBindState::Inactive,
            FeagiServerBindState::Active,
        ));
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        if let Some(endpoint) = self.bound_endpoint.take() {
            self.socket
                .unbind(endpoint)
                .await
                .map_err(|e| FeagiNetworkError::CannotUnbind(e.to_string()))?;
        }
        self.current_state = FeagiServerBindState::Inactive;
        (self.state_change_callback)(FeagiServerBindStateChange::new(
            FeagiServerBindState::Active,
            FeagiServerBindState::Inactive,
        ));
        Ok(())
    }

    fn get_current_state(&self) -> FeagiServerBindState {
        self.current_state
    }
}

#[async_trait]
impl FeagiServerPuller for FEAGIZMQServerPuller {
    async fn try_poll_receive(&mut self) -> Result<Vec<u8>, FeagiNetworkError> {
        match self.socket.recv().await {
            Ok(message) => message_to_single_frame(message),
            Err(e) => Err(FeagiNetworkError::ReceiveFailed(e.to_string())),
        }
    }
}

//endregion

//region Router

pub struct FEAGIZMQServerRouter {
    server_bind_address: String,
    current_state: FeagiServerBindState,
    state_change_callback: StateChangeCallback,
    socket: RouterSocket,
    bound_endpoint: Option<Endpoint>,
    // Counter for generating unique SessionID bytes
    next_client_id: u64,
    // Bidirectional mapping between SessionID and ZMQ identity
    identity_to_session: HashMap<Vec<u8>, SessionID>,
    session_to_identity: HashMap<[u8; SessionID::NUMBER_BYTES], Vec<u8>>,
}

impl FEAGIZMQServerRouter {
    pub fn new(
        server_bind_address: String,
        state_change_callback: StateChangeCallback,
    ) -> Result<Self, FeagiNetworkError> {
        validate_zmq_url(&server_bind_address)?;

        Ok(Self {
            server_bind_address,
            current_state: FeagiServerBindState::Inactive,
            state_change_callback,
            socket: RouterSocket::new(),
            bound_endpoint: None,
            next_client_id: 1, // Start from 1, reserve 0 for "no client"
            identity_to_session: HashMap::new(),
            session_to_identity: HashMap::new(),
        })
    }

    /// Get or create a SessionID for the given ZMQ identity
    fn get_or_create_session_id(&mut self, identity: Vec<u8>) -> SessionID {
        if let Some(&session_id) = self.identity_to_session.get(&identity) {
            session_id
        } else {
            // Generate SessionID bytes from the counter (u64 -> [u8; 8])
            let id_bytes = self.next_client_id.to_le_bytes();
            self.next_client_id += 1;
            let session_id = SessionID::new(id_bytes);
            self.identity_to_session.insert(identity.clone(), session_id);
            self.session_to_identity.insert(id_bytes, identity);
            session_id
        }
    }
}

#[async_trait]
impl FeagiServer for FEAGIZMQServerRouter {
    async fn start(&mut self) -> Result<(), FeagiNetworkError> {
        let endpoint = self
            .socket
            .bind(&self.server_bind_address)
            .await
            .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;
        self.bound_endpoint = Some(endpoint);
        self.current_state = FeagiServerBindState::Active;
        (self.state_change_callback)(FeagiServerBindStateChange::new(
            FeagiServerBindState::Inactive,
            FeagiServerBindState::Active,
        ));
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        if let Some(endpoint) = self.bound_endpoint.take() {
            self.socket
                .unbind(endpoint)
                .await
                .map_err(|e| FeagiNetworkError::CannotUnbind(e.to_string()))?;
        }
        // Clear session mappings when stopping
        self.identity_to_session.clear();
        self.session_to_identity.clear();
        self.next_client_id = 1;

        self.current_state = FeagiServerBindState::Inactive;
        (self.state_change_callback)(FeagiServerBindStateChange::new(
            FeagiServerBindState::Active,
            FeagiServerBindState::Inactive,
        ));

        Ok(())
    }

    fn get_current_state(&self) -> FeagiServerBindState {
        self.current_state
    }
}

#[async_trait]
impl FeagiServerRouter for FEAGIZMQServerRouter {
    async fn try_poll_receive(&mut self) -> Result<(SessionID, Vec<u8>), FeagiNetworkError> {
        let message = match self.socket.recv().await {
            Ok(message) => message,
            Err(e) => return Err(FeagiNetworkError::ReceiveFailed(e.to_string())),
        };

        let mut frames = message.into_vec();
        if frames.is_empty() {
            return Err(FeagiNetworkError::ReceiveFailed(
                "Empty ZMQ message received".to_string(),
            ));
        }

        let identity = frames.remove(0).to_vec();
        let mut payload_frames = frames;
        if payload_frames
            .first()
            .map(|frame| frame.is_empty())
            .unwrap_or(false)
        {
            payload_frames.remove(0);
        }

        if payload_frames.len() != 1 {
            return Err(FeagiNetworkError::ReceiveFailed(
                "Unexpected multipart request payload".to_string(),
            ));
        }

        let session_id = self.get_or_create_session_id(identity);
        let data = payload_frames.remove(0).to_vec();
        Ok((session_id, data))
    }

    async fn send_response(
        &mut self,
        client: SessionID,
        response: &[u8],
    ) -> Result<(), FeagiNetworkError> {
        // Look up the ZMQ identity for this SessionID
        let identity = self
            .session_to_identity
            .get(client.bytes())
            .ok_or_else(|| {
                FeagiNetworkError::SendFailed(format!("Unknown session ID: {:?}", client))
            })?;

        let mut message = ZmqMessage::from(response.to_vec());
        message.prepend(&ZmqMessage::from(Vec::new()));
        message.prepend(&ZmqMessage::from(identity.clone()));
        self.socket
            .send(message)
            .await
            .map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;
        Ok(())
    }
}

//endregion
