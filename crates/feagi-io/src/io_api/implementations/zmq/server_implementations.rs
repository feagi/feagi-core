use std::collections::HashMap;

use crate::io_api::implementations::zmq::shared_functions::validate_zmq_url;
use crate::io_api::traits_and_enums::server::server_shared::{
    ClientId, FeagiServerBindStateChange,
};
use crate::io_api::traits_and_enums::server::{
    FeagiServer, FeagiServerPublisher, FeagiServerPublisherProperties, FeagiServerPuller,
    FeagiServerPullerProperties, FeagiServerRouter, FeagiServerRouterProperties,
};
use crate::io_api::{FeagiNetworkError, FeagiServerBindState};
use futures_util::FutureExt;
use std::future::Future;
use tokio::runtime::{Handle, Runtime};
use tokio::task::block_in_place;
use zeromq::{
    Endpoint, PubSocket, PullSocket, RouterSocket, Socket, SocketRecv, SocketSend, ZmqMessage,
};

/// Type alias for the server state change callback.
type StateChangeCallback = Box<dyn Fn(FeagiServerBindStateChange) + Send + Sync + 'static>;

fn build_runtime() -> Result<Runtime, FeagiNetworkError> {
    Runtime::new().map_err(|e| FeagiNetworkError::GeneralFailure(e.to_string()))
}

fn block_on_runtime<T>(runtime: &Runtime, future: impl Future<Output = T>) -> T {
    if Handle::try_current().is_ok() {
        block_in_place(|| Handle::current().block_on(future))
    } else {
        runtime.block_on(future)
    }
}

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
    socket: PubSocket,
    runtime: Runtime,
    bound_endpoint: Option<Endpoint>,
    state_change_callback: StateChangeCallback,
    // Configuration options (applied on start)
    linger: i32,
    sndhwm: i32,
}

impl FEAGIZMQServerPublisher {
    const DEFAULT_LINGER: i32 = 0;
    const DEFAULT_SNDHWM: i32 = 1000;

    pub fn new(
        server_bind_address: String,
        state_change_callback: StateChangeCallback,
    ) -> Result<Self, FeagiNetworkError> {
        validate_zmq_url(&server_bind_address)?;
        let runtime = build_runtime()?;
        let socket = PubSocket::new();
        Ok(Self {
            server_bind_address,
            current_state: FeagiServerBindState::Inactive,
            socket,
            runtime,
            bound_endpoint: None,
            state_change_callback,
            linger: Self::DEFAULT_LINGER,
            sndhwm: Self::DEFAULT_SNDHWM,
        })
    }

    /// Set the linger period for socket shutdown (milliseconds).
    /// Returns error if socket is already running.
    pub fn set_linger(&mut self, linger: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiServerBindState::Active {
            return Err(FeagiNetworkError::GeneralFailure(
                "Cannot change configuration while socket is active".to_string(),
            ));
        }
        if linger != Self::DEFAULT_LINGER {
            return Err(FeagiNetworkError::GeneralFailure(format!(
                "zeromq transport does not support custom linger (requested: {})",
                linger
            )));
        }
        self.linger = linger;
        Ok(())
    }

    /// Set the send high water mark (message queue size).
    /// Returns error if socket is already running.
    pub fn set_sndhwm(&mut self, sndhwm: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiServerBindState::Active {
            return Err(FeagiNetworkError::GeneralFailure(
                "Cannot change configuration while socket is active".to_string(),
            ));
        }
        if sndhwm != Self::DEFAULT_SNDHWM {
            return Err(FeagiNetworkError::GeneralFailure(format!(
                "zeromq transport does not support custom sndhwm (requested: {})",
                sndhwm
            )));
        }
        self.sndhwm = sndhwm;
        Ok(())
    }

    fn ensure_supported_options(&self) -> Result<(), FeagiNetworkError> {
        if self.linger != Self::DEFAULT_LINGER || self.sndhwm != Self::DEFAULT_SNDHWM {
            return Err(FeagiNetworkError::GeneralFailure(format!(
                "zeromq transport does not support custom socket options (linger={}, sndhwm={})",
                self.linger, self.sndhwm
            )));
        }
        Ok(())
    }
}

impl FeagiServer for FEAGIZMQServerPublisher {
    fn start(&mut self) -> Result<(), FeagiNetworkError> {
        self.ensure_supported_options()?;
        let endpoint = block_on_runtime(
            &self.runtime,
            self.socket.bind(&self.server_bind_address),
        )
            .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;
        self.bound_endpoint = Some(endpoint);
        self.current_state = FeagiServerBindState::Active;
        (self.state_change_callback)(FeagiServerBindStateChange::new(
            FeagiServerBindState::Inactive,
            FeagiServerBindState::Active,
        ));
        Ok(())
    }

    fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        if let Some(endpoint) = self.bound_endpoint.take() {
            block_on_runtime(&self.runtime, self.socket.unbind(endpoint))
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

impl FeagiServerPublisher for FEAGIZMQServerPublisher {
    fn poll(&mut self) -> Result<(), FeagiNetworkError> {
        // ZMQ handles connections internally - no-op
        Ok(())
    }

    fn publish(&mut self, buffered_data_to_send: &[u8]) -> Result<(), FeagiNetworkError> {
        let message = ZmqMessage::from(buffered_data_to_send.to_vec());
        block_on_runtime(&self.runtime, self.socket.send(message))
            .map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;
        Ok(())
    }
}
//endregion

//region Puller

pub struct FEAGIZMQServerPuller {
    server_bind_address: String,
    current_state: FeagiServerBindState,
    socket: PullSocket,
    runtime: Runtime,
    bound_endpoint: Option<Endpoint>,
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

    pub fn new(
        server_bind_address: String,
        state_change_callback: StateChangeCallback,
    ) -> Result<Self, FeagiNetworkError> {
        validate_zmq_url(&server_bind_address)?;
        let runtime = build_runtime()?;
        let socket = PullSocket::new();
        Ok(Self {
            server_bind_address,
            current_state: FeagiServerBindState::Inactive,
            socket,
            runtime,
            bound_endpoint: None,
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
            return Err(FeagiNetworkError::GeneralFailure(
                "Cannot change configuration while socket is active".to_string(),
            ));
        }
        if linger != Self::DEFAULT_LINGER {
            return Err(FeagiNetworkError::GeneralFailure(format!(
                "zeromq transport does not support custom linger (requested: {})",
                linger
            )));
        }
        self.linger = linger;
        Ok(())
    }

    /// Set the receive high water mark (message queue size).
    /// Returns error if socket is already running.
    pub fn set_rcvhwm(&mut self, rcvhwm: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiServerBindState::Active {
            return Err(FeagiNetworkError::GeneralFailure(
                "Cannot change configuration while socket is active".to_string(),
            ));
        }
        if rcvhwm != Self::DEFAULT_RCVHWM {
            return Err(FeagiNetworkError::GeneralFailure(format!(
                "zeromq transport does not support custom rcvhwm (requested: {})",
                rcvhwm
            )));
        }
        self.rcvhwm = rcvhwm;
        Ok(())
    }

    /// Set immediate mode (only queue messages to completed connections).
    /// Returns error if socket is already running.
    pub fn set_immediate(&mut self, immediate: bool) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiServerBindState::Active {
            return Err(FeagiNetworkError::GeneralFailure(
                "Cannot change configuration while socket is active".to_string(),
            ));
        }
        if immediate != Self::DEFAULT_IMMEDIATE {
            return Err(FeagiNetworkError::GeneralFailure(format!(
                "zeromq transport does not support custom immediate (requested: {})",
                immediate
            )));
        }
        self.immediate = immediate;
        Ok(())
    }

    fn ensure_supported_options(&self) -> Result<(), FeagiNetworkError> {
        if self.linger != Self::DEFAULT_LINGER
            || self.rcvhwm != Self::DEFAULT_RCVHWM
            || self.immediate != Self::DEFAULT_IMMEDIATE
        {
            return Err(FeagiNetworkError::GeneralFailure(format!(
                "zeromq transport does not support custom socket options (linger={}, rcvhwm={}, immediate={})",
                self.linger, self.rcvhwm, self.immediate
            )));
        }
        Ok(())
    }
}

impl FeagiServer for FEAGIZMQServerPuller {
    fn start(&mut self) -> Result<(), FeagiNetworkError> {
        self.ensure_supported_options()?;
        let endpoint = block_on_runtime(
            &self.runtime,
            self.socket.bind(&self.server_bind_address),
        )
            .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;
        self.bound_endpoint = Some(endpoint);
        self.current_state = FeagiServerBindState::Active;
        (self.state_change_callback)(FeagiServerBindStateChange::new(
            FeagiServerBindState::Inactive,
            FeagiServerBindState::Active,
        ));
        Ok(())
    }

    fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        if let Some(endpoint) = self.bound_endpoint.take() {
            block_on_runtime(&self.runtime, self.socket.unbind(endpoint))
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

impl FeagiServerPuller for FEAGIZMQServerPuller {
    fn try_poll_receive(&mut self) -> Result<Option<&[u8]>, FeagiNetworkError> {
        let result = block_on_runtime(&self.runtime, async { self.socket.recv().now_or_never() });
        match result {
            None => Ok(None),
            Some(Ok(message)) => {
                self.cached_data = message_to_single_frame(message)?;
                Ok(Some(&self.cached_data))
            }
            Some(Err(e)) => Err(FeagiNetworkError::ReceiveFailed(e.to_string())),
        }
    }
}

//endregion

//region Router

pub struct FEAGIZMQServerRouter {
    server_bind_address: String,
    current_state: FeagiServerBindState,
    socket: RouterSocket,
    runtime: Runtime,
    bound_endpoint: Option<Endpoint>,
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

    pub fn new(
        server_bind_address: String,
        state_change_callback: StateChangeCallback,
    ) -> Result<Self, FeagiNetworkError> {
        validate_zmq_url(&server_bind_address)?;
        let runtime = build_runtime()?;
        let socket = RouterSocket::new();
        Ok(Self {
            server_bind_address,
            current_state: FeagiServerBindState::Inactive,
            socket,
            runtime,
            bound_endpoint: None,
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
            return Err(FeagiNetworkError::GeneralFailure(
                "Cannot change configuration while socket is active".to_string(),
            ));
        }
        if linger != Self::DEFAULT_LINGER {
            return Err(FeagiNetworkError::GeneralFailure(format!(
                "zeromq transport does not support custom linger (requested: {})",
                linger
            )));
        }
        self.linger = linger;
        Ok(())
    }

    /// Set the receive high water mark (message queue size).
    /// Returns error if socket is already running.
    pub fn set_rcvhwm(&mut self, rcvhwm: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiServerBindState::Active {
            return Err(FeagiNetworkError::GeneralFailure(
                "Cannot change configuration while socket is active".to_string(),
            ));
        }
        if rcvhwm != Self::DEFAULT_RCVHWM {
            return Err(FeagiNetworkError::GeneralFailure(format!(
                "zeromq transport does not support custom rcvhwm (requested: {})",
                rcvhwm
            )));
        }
        self.rcvhwm = rcvhwm;
        Ok(())
    }

    /// Set the send high water mark (message queue size).
    /// Returns error if socket is already running.
    pub fn set_sndhwm(&mut self, sndhwm: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiServerBindState::Active {
            return Err(FeagiNetworkError::GeneralFailure(
                "Cannot change configuration while socket is active".to_string(),
            ));
        }
        if sndhwm != Self::DEFAULT_SNDHWM {
            return Err(FeagiNetworkError::GeneralFailure(format!(
                "zeromq transport does not support custom sndhwm (requested: {})",
                sndhwm
            )));
        }
        self.sndhwm = sndhwm;
        Ok(())
    }

    /// Set immediate mode (only queue messages to completed connections).
    /// Returns error if socket is already running.
    pub fn set_immediate(&mut self, immediate: bool) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiServerBindState::Active {
            return Err(FeagiNetworkError::GeneralFailure(
                "Cannot change configuration while socket is active".to_string(),
            ));
        }
        if immediate != Self::DEFAULT_IMMEDIATE {
            return Err(FeagiNetworkError::GeneralFailure(format!(
                "zeromq transport does not support custom immediate (requested: {})",
                immediate
            )));
        }
        self.immediate = immediate;
        Ok(())
    }

    fn ensure_supported_options(&self) -> Result<(), FeagiNetworkError> {
        if self.linger != Self::DEFAULT_LINGER
            || self.rcvhwm != Self::DEFAULT_RCVHWM
            || self.sndhwm != Self::DEFAULT_SNDHWM
            || self.immediate != Self::DEFAULT_IMMEDIATE
        {
            return Err(FeagiNetworkError::GeneralFailure(format!(
                "zeromq transport does not support custom socket options (linger={}, rcvhwm={}, sndhwm={}, immediate={})",
                self.linger, self.rcvhwm, self.sndhwm, self.immediate
            )));
        }
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
        self.ensure_supported_options()?;
        let endpoint = block_on_runtime(
            &self.runtime,
            self.socket.bind(&self.server_bind_address),
        )
            .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;
        self.bound_endpoint = Some(endpoint);
        self.current_state = FeagiServerBindState::Active;
        (self.state_change_callback)(FeagiServerBindStateChange::new(
            FeagiServerBindState::Inactive,
            FeagiServerBindState::Active,
        ));
        Ok(())
    }

    fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        if let Some(endpoint) = self.bound_endpoint.take() {
            block_on_runtime(&self.runtime, self.socket.unbind(endpoint))
                .map_err(|e| FeagiNetworkError::CannotUnbind(e.to_string()))?;
        }
        // Clear client mappings when stopping
        self.identity_to_id.clear();
        self.id_to_identity.clear();
        self.next_client_id = 1;
        self.last_client_id = None;

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

impl FeagiServerRouter for FEAGIZMQServerRouter {
    fn try_poll_receive(&mut self) -> Result<Option<(ClientId, &[u8])>, FeagiNetworkError> {
        let result = block_on_runtime(&self.runtime, async { self.socket.recv().now_or_never() });
        let message = match result {
            None => return Ok(None),
            Some(Ok(message)) => message,
            Some(Err(e)) => return Err(FeagiNetworkError::ReceiveFailed(e.to_string())),
        };

        let mut frames = message.into_vec();
        if frames.is_empty() {
            return Err(FeagiNetworkError::ReceiveFailed(
                "Empty ZMQ message received".to_string(),
            ));
        }

        let identity = frames.remove(0).to_vec();
        let mut payload_frames = frames;
        if payload_frames.first().map(|frame| frame.is_empty()).unwrap_or(false) {
            payload_frames.remove(0);
        }

        if payload_frames.len() != 1 {
            return Err(FeagiNetworkError::ReceiveFailed(
                "Unexpected multipart request payload".to_string(),
            ));
        }

        let client_id = self.get_or_create_client_id(identity);
        self.last_client_id = Some(client_id);
        self.cached_request_data = payload_frames.remove(0).to_vec();
        Ok(Some((client_id, &self.cached_request_data)))
    }

    fn send_response(
        &mut self,
        client: ClientId,
        response: &[u8],
    ) -> Result<(), FeagiNetworkError> {
        // Look up the ZMQ identity for this ClientId
        let identity = self.id_to_identity.get(&client.0).ok_or_else(|| {
            FeagiNetworkError::SendFailed(format!("Unknown client ID: {:?}", client))
        })?;

        let mut message = ZmqMessage::from(response.to_vec());
        message.prepend(&ZmqMessage::from(Vec::new()));
        message.prepend(&ZmqMessage::from(identity.clone()));
        block_on_runtime(&self.runtime, self.socket.send(message))
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
    fn build(
        self: Box<Self>,
        state_change_callback: StateChangeCallback,
    ) -> Box<dyn FeagiServerPublisher> {
        let mut publisher =
            FEAGIZMQServerPublisher::new(self.server_bind_address, state_change_callback)
                .expect("Failed to create ZMQ publisher");

        publisher
            .set_linger(self.linger)
            .expect("Unsupported publisher linger configuration");
        publisher
            .set_sndhwm(self.sndhwm)
            .expect("Unsupported publisher sndhwm configuration");

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
    fn build(
        self: Box<Self>,
        state_change_callback: StateChangeCallback,
    ) -> Box<dyn FeagiServerPuller> {
        let mut puller = FEAGIZMQServerPuller::new(self.server_bind_address, state_change_callback)
            .expect("Failed to create ZMQ puller");

        puller
            .set_linger(self.linger)
            .expect("Unsupported puller linger configuration");
        puller
            .set_rcvhwm(self.rcvhwm)
            .expect("Unsupported puller rcvhwm configuration");
        puller
            .set_immediate(self.immediate)
            .expect("Unsupported puller immediate configuration");

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
    fn build(
        self: Box<Self>,
        state_change_callback: StateChangeCallback,
    ) -> Box<dyn FeagiServerRouter> {
        let mut router = FEAGIZMQServerRouter::new(self.server_bind_address, state_change_callback)
            .expect("Failed to create ZMQ router");

        router
            .set_linger(self.linger)
            .expect("Unsupported router linger configuration");
        router
            .set_rcvhwm(self.rcvhwm)
            .expect("Unsupported router rcvhwm configuration");
        router
            .set_sndhwm(self.sndhwm)
            .expect("Unsupported router sndhwm configuration");
        router
            .set_immediate(self.immediate)
            .expect("Unsupported router immediate configuration");

        Box::new(router)
    }
}

//endregion

//endregion
