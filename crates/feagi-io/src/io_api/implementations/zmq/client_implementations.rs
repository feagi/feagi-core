use crate::io_api::implementations::zmq::shared_functions::validate_zmq_url;
use crate::io_api::traits_and_enums::client::client_shared::FeagiClientConnectionStateChange;
use crate::io_api::traits_and_enums::client::{
    FeagiClient, FeagiClientPusher, FeagiClientPusherProperties, FeagiClientRequester,
    FeagiClientRequesterProperties, FeagiClientSubscriber, FeagiClientSubscriberProperties,
};
use crate::io_api::{FeagiClientConnectionState, FeagiNetworkError};
use futures_util::FutureExt;
use parking_lot::Mutex;
use std::future::Future;
use tokio::runtime::{Handle, Runtime};
use tokio::task::block_in_place;
use zeromq::{DealerSocket, PushSocket, Socket, SocketRecv, SocketSend, SubSocket, ZmqMessage};

/// Type alias for the client state change callback.
type StateChangeCallback = Box<dyn Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static>;

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

//region Subscriber

pub struct FEAGIZMQClientSubscriber {
    server_address: String,
    current_state: FeagiClientConnectionState,
    socket: SubSocket,
    runtime: Runtime,
    state_change_callback: StateChangeCallback,
    cached_data: Vec<u8>,
    // Configuration options (applied on connect)
    linger: i32,
    rcvhwm: i32,
}

impl FEAGIZMQClientSubscriber {
    const DEFAULT_LINGER: i32 = 0;
    const DEFAULT_RCVHWM: i32 = 1000;

    pub fn new(
        server_address: String,
        state_change_callback: StateChangeCallback,
    ) -> Result<Self, FeagiNetworkError> {
        validate_zmq_url(&server_address)?;
        let runtime = build_runtime()?;
        let socket = SubSocket::new();
        Ok(Self {
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            socket,
            runtime,
            state_change_callback,
            cached_data: Vec::new(),
            linger: Self::DEFAULT_LINGER,
            rcvhwm: Self::DEFAULT_RCVHWM,
        })
    }

    /// Set the linger period for socket shutdown (milliseconds).
    /// Returns error if socket is already connected.
    pub fn set_linger(&mut self, linger: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiClientConnectionState::Connected {
            return Err(FeagiNetworkError::GeneralFailure(
                "Cannot change configuration while connected".to_string(),
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
    /// Returns error if socket is already connected.
    pub fn set_rcvhwm(&mut self, rcvhwm: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiClientConnectionState::Connected {
            return Err(FeagiNetworkError::GeneralFailure(
                "Cannot change configuration while connected".to_string(),
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

    fn ensure_supported_options(&self) -> Result<(), FeagiNetworkError> {
        if self.linger != Self::DEFAULT_LINGER || self.rcvhwm != Self::DEFAULT_RCVHWM {
            return Err(FeagiNetworkError::GeneralFailure(format!(
                "zeromq transport does not support custom socket options (linger={}, rcvhwm={})",
                self.linger, self.rcvhwm
            )));
        }
        Ok(())
    }

    /// Poll for incoming subscription data.
    /// Returns `Ok(Some(data))` if data was received, `Ok(None)` if no data available.
    pub fn try_poll_receive(&mut self) -> Result<Option<&[u8]>, FeagiNetworkError> {
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

impl FeagiClient for FEAGIZMQClientSubscriber {
    fn connect(&mut self, host: &str) -> Result<(), FeagiNetworkError> {
        self.ensure_supported_options()?;
        block_on_runtime(&self.runtime, self.socket.subscribe(""))
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        block_on_runtime(&self.runtime, self.socket.connect(host))
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;

        self.server_address = host.to_string(); // Store for disconnect
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Connected;
        (self.state_change_callback)(FeagiClientConnectionStateChange::new(
            previous,
            self.current_state,
        ));
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), FeagiNetworkError> {
        let socket = std::mem::replace(&mut self.socket, SubSocket::new());
        let _ = block_on_runtime(&self.runtime, socket.close());
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
    // Polling method is on the impl block directly since trait is empty
}

//endregion

//region Pusher

pub struct FEAGIZMQClientPusher {
    server_address: String,
    current_state: FeagiClientConnectionState,
    socket: Mutex<PushSocket>,
    runtime: Runtime,
    state_change_callback: StateChangeCallback,
    // Configuration options (applied on connect)
    linger: i32,
    sndhwm: i32,
}

impl FEAGIZMQClientPusher {
    const DEFAULT_LINGER: i32 = 0;
    const DEFAULT_SNDHWM: i32 = 1000;

    pub fn new(
        server_address: String,
        state_change_callback: StateChangeCallback,
    ) -> Result<Self, FeagiNetworkError> {
        validate_zmq_url(&server_address)?;
        let runtime = build_runtime()?;
        let socket = Mutex::new(PushSocket::new());
        Ok(Self {
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            socket,
            runtime,
            state_change_callback,
            linger: Self::DEFAULT_LINGER,
            sndhwm: Self::DEFAULT_SNDHWM,
        })
    }

    /// Set the linger period for socket shutdown (milliseconds).
    /// Returns error if socket is already connected.
    pub fn set_linger(&mut self, linger: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiClientConnectionState::Connected {
            return Err(FeagiNetworkError::GeneralFailure(
                "Cannot change configuration while connected".to_string(),
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
    /// Returns error if socket is already connected.
    pub fn set_sndhwm(&mut self, sndhwm: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiClientConnectionState::Connected {
            return Err(FeagiNetworkError::GeneralFailure(
                "Cannot change configuration while connected".to_string(),
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

impl FeagiClient for FEAGIZMQClientPusher {
    fn connect(&mut self, host: &str) -> Result<(), FeagiNetworkError> {
        self.ensure_supported_options()?;
        block_on_runtime(&self.runtime, self.socket.lock().connect(host))
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;

        self.server_address = host.to_string(); // Store for disconnect
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Connected;
        (self.state_change_callback)(FeagiClientConnectionStateChange::new(
            previous,
            self.current_state,
        ));
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), FeagiNetworkError> {
        let socket = std::mem::replace(&mut self.socket, Mutex::new(PushSocket::new()));
        let socket = socket.into_inner();
        let _ = block_on_runtime(&self.runtime, socket.close());
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
    fn push_data(&self, data: &[u8]) {
        let message = ZmqMessage::from(data.to_vec());
        let _ = block_on_runtime(&self.runtime, self.socket.lock().send(message));
    }
}

//endregion

//region Requester (Dealer)

pub struct FEAGIZMQClientRequester {
    server_address: String,
    current_state: FeagiClientConnectionState,
    socket: Mutex<DealerSocket>,
    state_change_callback: StateChangeCallback,
    cached_response_data: Vec<u8>,
    // Configuration options (applied on connect)
    linger: i32,
    rcvhwm: i32,
    sndhwm: i32,
    runtime: Runtime,
}

impl FEAGIZMQClientRequester {
    const DEFAULT_LINGER: i32 = 0;
    const DEFAULT_RCVHWM: i32 = 1000;
    const DEFAULT_SNDHWM: i32 = 1000;

    pub fn new(
        server_address: String,
        state_change_callback: StateChangeCallback,
    ) -> Result<Self, FeagiNetworkError> {
        validate_zmq_url(&server_address)?;
        let runtime = build_runtime()?;
        let socket = Mutex::new(DealerSocket::new());
        Ok(Self {
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            socket,
            state_change_callback,
            cached_response_data: Vec::new(),
            linger: Self::DEFAULT_LINGER,
            rcvhwm: Self::DEFAULT_RCVHWM,
            sndhwm: Self::DEFAULT_SNDHWM,
            runtime,
        })
    }

    /// Set the linger period for socket shutdown (milliseconds).
    /// Returns error if socket is already connected.
    pub fn set_linger(&mut self, linger: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiClientConnectionState::Connected {
            return Err(FeagiNetworkError::GeneralFailure(
                "Cannot change configuration while connected".to_string(),
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
    /// Returns error if socket is already connected.
    pub fn set_rcvhwm(&mut self, rcvhwm: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiClientConnectionState::Connected {
            return Err(FeagiNetworkError::GeneralFailure(
                "Cannot change configuration while connected".to_string(),
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
    /// Returns error if socket is already connected.
    pub fn set_sndhwm(&mut self, sndhwm: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiClientConnectionState::Connected {
            return Err(FeagiNetworkError::GeneralFailure(
                "Cannot change configuration while connected".to_string(),
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
        if self.linger != Self::DEFAULT_LINGER
            || self.rcvhwm != Self::DEFAULT_RCVHWM
            || self.sndhwm != Self::DEFAULT_SNDHWM
        {
            return Err(FeagiNetworkError::GeneralFailure(format!(
                "zeromq transport does not support custom socket options (linger={}, rcvhwm={}, sndhwm={})",
                self.linger, self.rcvhwm, self.sndhwm
            )));
        }
        Ok(())
    }
}

impl FeagiClient for FEAGIZMQClientRequester {
    fn connect(&mut self, host: &str) -> Result<(), FeagiNetworkError> {
        self.ensure_supported_options()?;
        block_on_runtime(&self.runtime, self.socket.lock().connect(host))
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;

        self.server_address = host.to_string(); // Store for disconnect
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Connected;
        (self.state_change_callback)(FeagiClientConnectionStateChange::new(
            previous,
            self.current_state,
        ));
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), FeagiNetworkError> {
        let socket = std::mem::replace(&mut self.socket, Mutex::new(DealerSocket::new()));
        let socket = socket.into_inner();
        let _ = block_on_runtime(&self.runtime, socket.close());
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
    fn send_request(&self, request: &[u8]) -> Result<(), FeagiNetworkError> {
        let mut message = ZmqMessage::from(request.to_vec());
        message.prepend(&ZmqMessage::from(Vec::new()));
        block_on_runtime(&self.runtime, self.socket.lock().send(message))
            .map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;
        Ok(())
    }

    fn try_poll_receive(&mut self) -> Result<Option<&[u8]>, FeagiNetworkError> {
        let result = block_on_runtime(&self.runtime, async {
            self.socket.lock().recv().now_or_never()
        });
        let message = match result {
            None => return Ok(None),
            Some(Ok(message)) => message,
            Some(Err(e)) => return Err(FeagiNetworkError::ReceiveFailed(e.to_string())),
        };

        let mut frames = message.into_vec();
        if frames
            .first()
            .map(|frame| frame.is_empty())
            .unwrap_or(false)
        {
            frames.remove(0);
        }

        if frames.len() != 1 {
            return Err(FeagiNetworkError::ReceiveFailed(
                "Unexpected multipart response payload".to_string(),
            ));
        }

        self.cached_response_data = frames.remove(0).to_vec();
        Ok(Some(&self.cached_response_data))
    }
}

//endregion

//region Properties

//region Subscriber Properties

/// Properties for configuring and building a ZMQ Client Subscriber.
pub struct FEAGIZMQClientSubscriberProperties {
    server_address: String,
    linger: i32,
    rcvhwm: i32,
}

impl FEAGIZMQClientSubscriberProperties {
    const DEFAULT_LINGER: i32 = 0;
    const DEFAULT_RCVHWM: i32 = 1000;

    /// Create new properties with the given server address.
    pub fn new(server_address: String) -> Self {
        Self {
            server_address,
            linger: Self::DEFAULT_LINGER,
            rcvhwm: Self::DEFAULT_RCVHWM,
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
}

impl FeagiClientSubscriberProperties for FEAGIZMQClientSubscriberProperties {
    fn build(
        self: Box<Self>,
        state_change_callback: StateChangeCallback,
    ) -> Box<dyn FeagiClientSubscriber> {
        let mut subscriber =
            FEAGIZMQClientSubscriber::new(self.server_address, state_change_callback)
                .expect("Failed to create ZMQ subscriber");

        subscriber
            .set_linger(self.linger)
            .expect("Unsupported subscriber linger configuration");
        subscriber
            .set_rcvhwm(self.rcvhwm)
            .expect("Unsupported subscriber rcvhwm configuration");

        Box::new(subscriber)
    }
}

//endregion

//region Pusher Properties

/// Properties for configuring and building a ZMQ Client Pusher.
pub struct FEAGIZMQClientPusherProperties {
    server_address: String,
    linger: i32,
    sndhwm: i32,
}

impl FEAGIZMQClientPusherProperties {
    const DEFAULT_LINGER: i32 = 0;
    const DEFAULT_SNDHWM: i32 = 1000;

    /// Create new properties with the given server address.
    pub fn new(server_address: String) -> Self {
        Self {
            server_address,
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

impl FeagiClientPusherProperties for FEAGIZMQClientPusherProperties {
    fn build(
        self: Box<Self>,
        state_change_callback: StateChangeCallback,
    ) -> Box<dyn FeagiClientPusher> {
        let mut pusher = FEAGIZMQClientPusher::new(self.server_address, state_change_callback)
            .expect("Failed to create ZMQ pusher");

        pusher
            .set_linger(self.linger)
            .expect("Unsupported pusher linger configuration");
        pusher
            .set_sndhwm(self.sndhwm)
            .expect("Unsupported pusher sndhwm configuration");

        Box::new(pusher)
    }
}

//endregion

//region Requester Properties

/// Properties for configuring and building a ZMQ Client Requester.
pub struct FEAGIZMQClientRequesterProperties {
    server_address: String,
    linger: i32,
    rcvhwm: i32,
    sndhwm: i32,
}

impl FEAGIZMQClientRequesterProperties {
    const DEFAULT_LINGER: i32 = 0;
    const DEFAULT_RCVHWM: i32 = 1000;
    const DEFAULT_SNDHWM: i32 = 1000;

    /// Create new properties with the given server address.
    pub fn new(server_address: String) -> Self {
        Self {
            server_address,
            linger: Self::DEFAULT_LINGER,
            rcvhwm: Self::DEFAULT_RCVHWM,
            sndhwm: Self::DEFAULT_SNDHWM,
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
}

impl FeagiClientRequesterProperties for FEAGIZMQClientRequesterProperties {
    fn build(
        self: Box<Self>,
        state_change_callback: StateChangeCallback,
    ) -> Box<dyn FeagiClientRequester> {
        let mut requester =
            FEAGIZMQClientRequester::new(self.server_address, state_change_callback)
                .expect("Failed to create ZMQ requester");

        requester
            .set_linger(self.linger)
            .expect("Unsupported requester linger configuration");
        requester
            .set_rcvhwm(self.rcvhwm)
            .expect("Unsupported requester rcvhwm configuration");
        requester
            .set_sndhwm(self.sndhwm)
            .expect("Unsupported requester sndhwm configuration");

        Box::new(requester)
    }
}

//endregion

//endregion
