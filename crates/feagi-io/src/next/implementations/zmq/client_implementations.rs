use crate::next::{FeagiClientConnectionState, FeagiNetworkError};
use crate::next::implementations::zmq::shared_functions::validate_zmq_url;
use crate::next::traits_and_enums::client::{FeagiClient, FeagiClientSubscriber, FeagiClientPusher, FeagiClientRequester};
use crate::next::traits_and_enums::client::client_shared::FeagiClientConnectionStateChange;
//region Subscriber

pub struct FEAGIZMQClientSubscriber<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    context_ref: zmq::Context,
    server_address: String,
    current_state: FeagiClientConnectionState,
    socket: zmq::Socket,
    state_change_callback: S,
    cached_data: Vec<u8>,
}

impl<S> FEAGIZMQClientSubscriber<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    pub fn new(context: &mut zmq::Context, server_address: String, state_change_callback: S)
        -> Result<Self, FeagiNetworkError>
    {
        validate_zmq_url(&server_address)?;
        let socket = context.socket(zmq::SUB).map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        // Subscribe to all messages by default
        socket.set_subscribe(b"").map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        // Set socket to non-blocking mode for polling
        socket.set_rcvtimeo(0).map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        Ok(Self {
            context_ref: context.clone(),
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            socket,
            state_change_callback,
            cached_data: Vec::new(),
        })
    }

    /// Poll for incoming subscription data.
    /// Returns `Ok(Some(data))` if data was received, `Ok(None)` if no data available.
    pub fn try_poll_receive(&mut self) -> Result<Option<&[u8]>, FeagiNetworkError> {
        match self.socket.recv_bytes(0) {
            Ok(data) => {
                self.cached_data = data;
                Ok(Some(&self.cached_data))
            }
            Err(zmq::Error::EAGAIN) => Ok(None), // No data available
            Err(e) => Err(FeagiNetworkError::ReceiveFailed(e.to_string()))
        }
    }
}

impl<S> FeagiClient for FEAGIZMQClientSubscriber<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    fn connect(&mut self, host: &str) -> Result<(), FeagiNetworkError> {
        self.socket.connect(host)
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Connected;
        (self.state_change_callback)(
            FeagiClientConnectionStateChange::new(previous, self.current_state)
        );
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), FeagiNetworkError> {
        self.socket.disconnect(&self.server_address)
            .map_err(|e| FeagiNetworkError::CannotDisconnect(e.to_string()))?;
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Disconnected;
        (self.state_change_callback)(
            FeagiClientConnectionStateChange::new(previous, self.current_state)
        );
        Ok(())
    }

    fn get_current_connection_state(&self) -> FeagiClientConnectionState {
        self.current_state
    }
}

impl<S> FeagiClientSubscriber for FEAGIZMQClientSubscriber<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    // Polling method is on the impl block directly since trait is empty
}

//endregion

//region Pusher

pub struct FEAGIZMQClientPusher<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    context_ref: zmq::Context,
    server_address: String,
    current_state: FeagiClientConnectionState,
    socket: zmq::Socket,
    state_change_callback: S,
}

impl<S> FEAGIZMQClientPusher<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    pub fn new(context: &mut zmq::Context, server_address: String, state_change_callback: S) 
        -> Result<Self, FeagiNetworkError> 
    {
        validate_zmq_url(&server_address)?;
        let socket = context.socket(zmq::PUSH).map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        Ok(Self {
            context_ref: context.clone(),
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            socket,
            state_change_callback,
        })
    }
}

impl<S> FeagiClient for FEAGIZMQClientPusher<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    fn connect(&mut self, host: &str) -> Result<(), FeagiNetworkError> {
        self.socket.connect(host)
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Connected;
        (self.state_change_callback)(
            FeagiClientConnectionStateChange::new(previous, self.current_state)
        );
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), FeagiNetworkError> {
        self.socket.disconnect(&self.server_address)
            .map_err(|e| FeagiNetworkError::CannotDisconnect(e.to_string()))?;
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Disconnected;
        (self.state_change_callback)(
            FeagiClientConnectionStateChange::new(previous, self.current_state)
        );
        Ok(())
    }

    fn get_current_connection_state(&self) -> FeagiClientConnectionState {
        self.current_state
    }
}

impl<S> FeagiClientPusher for FEAGIZMQClientPusher<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    fn push_data(&self, data: &[u8]) {
        // TODO: Return Result in state changes if theres an error
        let _ = self.socket.send(data, 0);
    }
}

//endregion

//region Requester (Dealer)

pub struct FEAGIZMQClientRequester<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    context_ref: zmq::Context,
    server_address: String,
    current_state: FeagiClientConnectionState,
    socket: zmq::Socket,
    state_change_callback: S,
    cached_response_data: Vec<u8>,
}

impl<S> FEAGIZMQClientRequester<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    pub fn new(context: &mut zmq::Context, server_address: String, state_change_callback: S) 
        -> Result<Self, FeagiNetworkError> 
    {
        validate_zmq_url(&server_address)?;
        let socket = context.socket(zmq::DEALER).map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        // Set socket to non-blocking mode for try_poll_receive
        socket.set_rcvtimeo(0).map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        Ok(Self {
            context_ref: context.clone(),
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            socket,
            state_change_callback,
            cached_response_data: Vec::new(),
        })
    }
}

impl<S> FeagiClient for FEAGIZMQClientRequester<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    fn connect(&mut self, host: &str) -> Result<(), FeagiNetworkError> {
        self.socket.connect(host)
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Connected;
        (self.state_change_callback)(
            FeagiClientConnectionStateChange::new(previous, self.current_state)
        );
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), FeagiNetworkError> {
        self.socket.disconnect(&self.server_address)
            .map_err(|e| FeagiNetworkError::CannotDisconnect(e.to_string()))?;
        let previous = self.current_state;
        self.current_state = FeagiClientConnectionState::Disconnected;
        (self.state_change_callback)(
            FeagiClientConnectionStateChange::new(previous, self.current_state)
        );
        Ok(())
    }

    fn get_current_connection_state(&self) -> FeagiClientConnectionState {
        self.current_state
    }
}

impl<S> FeagiClientRequester for FEAGIZMQClientRequester<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    fn send_request(&self, request: &[u8]) -> Result<(), FeagiNetworkError> {
        // DEALER/ROUTER protocol: send empty delimiter frame first, then request
        self.socket.send(zmq::Message::new(), zmq::SNDMORE)
            .map_err(|e| FeagiNetworkError::SendFailed(format!("Failed to send delimiter: {}", e)))?;
        self.socket.send(request, 0)
            .map_err(|e| FeagiNetworkError::SendFailed(e.to_string()))?;
        Ok(())
    }

    fn try_poll_receive(&mut self) -> Result<Option<&[u8]>, FeagiNetworkError> {
        // Response format: [empty_delimiter, response_data]
        // First, try to receive the empty delimiter frame
        match self.socket.recv_bytes(0) {
            Ok(_delimiter) => {
                // Delimiter received, now get the actual response
            }
            Err(zmq::Error::EAGAIN) => return Ok(None), // No data available
            Err(e) => return Err(FeagiNetworkError::ReceiveFailed(e.to_string()))
        }

        // Receive actual response data
        match self.socket.recv_bytes(0) {
            Ok(data) => {
                self.cached_response_data = data;
                Ok(Some(&self.cached_response_data))
            }
            Err(e) => Err(FeagiNetworkError::ReceiveFailed(e.to_string()))
        }
    }
}

//endregion
