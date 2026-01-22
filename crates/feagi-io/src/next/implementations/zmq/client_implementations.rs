use crate::next::{FeagiClientConnectionState, FeagiNetworkError};
use crate::next::implementations::zmq::shared_functions::validate_zmq_url;
use crate::next::traits_and_enums::client::{
    FeagiClient, FeagiClientSubscriber, FeagiClientPusher, FeagiClientRequester,
    FeagiClientSubscriberProperties, FeagiClientPusherProperties, FeagiClientRequesterProperties
};
use crate::next::traits_and_enums::client::client_shared::FeagiClientConnectionStateChange;

//region Subscriber

pub struct FEAGIZMQClientSubscriber<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    server_address: String,
    current_state: FeagiClientConnectionState,
    socket: zmq::Socket,
    state_change_callback: S,
    cached_data: Vec<u8>,
    // Configuration options (applied on connect)
    linger: i32,
    rcvhwm: i32,
}

impl<S> FEAGIZMQClientSubscriber<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    const DEFAULT_LINGER: i32 = 0;
    const DEFAULT_RCVHWM: i32 = 1000;
    
    pub fn new(server_address: String, state_change_callback: S)
        -> Result<Self, FeagiNetworkError>
    {
        validate_zmq_url(&server_address)?;
        let context = zmq::Context::new();
        let socket = context.socket(zmq::SUB)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        Ok(Self {
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            socket,
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
            return Err(FeagiNetworkError::GeneralFailure("Cannot change configuration while connected".to_string()));
        }
        self.linger = linger;
        Ok(())
    }
    
    /// Set the receive high water mark (message queue size).
    /// Returns error if socket is already connected.
    pub fn set_rcvhwm(&mut self, rcvhwm: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiClientConnectionState::Connected {
            return Err(FeagiNetworkError::GeneralFailure("Cannot change configuration while connected".to_string()));
        }
        self.rcvhwm = rcvhwm;
        Ok(())
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
        // Apply configuration options
        self.socket.set_linger(self.linger)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        self.socket.set_rcvhwm(self.rcvhwm)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        // Subscribe to all messages by default
        self.socket.set_subscribe(b"")
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        // Set socket to non-blocking mode for polling
        self.socket.set_rcvtimeo(0)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        self.socket.connect(host)
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;
        
        self.server_address = host.to_string(); // Store for disconnect
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
    server_address: String,
    current_state: FeagiClientConnectionState,
    socket: zmq::Socket,
    state_change_callback: S,
    // Configuration options (applied on connect)
    linger: i32,
    sndhwm: i32,
}

impl<S> FEAGIZMQClientPusher<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    const DEFAULT_LINGER: i32 = 0;
    const DEFAULT_SNDHWM: i32 = 1000;
    
    pub fn new(server_address: String, state_change_callback: S) 
        -> Result<Self, FeagiNetworkError> 
    {
        validate_zmq_url(&server_address)?;
        let context = zmq::Context::new();
        let socket = context.socket(zmq::PUSH)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        Ok(Self {
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            socket,
            state_change_callback,
            linger: Self::DEFAULT_LINGER,
            sndhwm: Self::DEFAULT_SNDHWM,
        })
    }
    
    /// Set the linger period for socket shutdown (milliseconds).
    /// Returns error if socket is already connected.
    pub fn set_linger(&mut self, linger: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiClientConnectionState::Connected {
            return Err(FeagiNetworkError::GeneralFailure("Cannot change configuration while connected".to_string()));
        }
        self.linger = linger;
        Ok(())
    }
    
    /// Set the send high water mark (message queue size).
    /// Returns error if socket is already connected.
    pub fn set_sndhwm(&mut self, sndhwm: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiClientConnectionState::Connected {
            return Err(FeagiNetworkError::GeneralFailure("Cannot change configuration while connected".to_string()));
        }
        self.sndhwm = sndhwm;
        Ok(())
    }
}

impl<S> FeagiClient for FEAGIZMQClientPusher<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    fn connect(&mut self, host: &str) -> Result<(), FeagiNetworkError> {
        // Apply configuration options
        self.socket.set_linger(self.linger)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        self.socket.set_sndhwm(self.sndhwm)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        self.socket.connect(host)
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;
        
        self.server_address = host.to_string(); // Store for disconnect
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
    server_address: String,
    current_state: FeagiClientConnectionState,
    socket: zmq::Socket,
    state_change_callback: S,
    cached_response_data: Vec<u8>,
    // Configuration options (applied on connect)
    linger: i32,
    rcvhwm: i32,
    sndhwm: i32,
}

impl<S> FEAGIZMQClientRequester<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    const DEFAULT_LINGER: i32 = 0;
    const DEFAULT_RCVHWM: i32 = 1000;
    const DEFAULT_SNDHWM: i32 = 1000;
    
    pub fn new(server_address: String, state_change_callback: S) 
        -> Result<Self, FeagiNetworkError> 
    {
        validate_zmq_url(&server_address)?;
        let context = zmq::Context::new();
        let socket = context.socket(zmq::DEALER)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        Ok(Self {
            server_address,
            current_state: FeagiClientConnectionState::Disconnected,
            socket,
            state_change_callback,
            cached_response_data: Vec::new(),
            linger: Self::DEFAULT_LINGER,
            rcvhwm: Self::DEFAULT_RCVHWM,
            sndhwm: Self::DEFAULT_SNDHWM,
        })
    }
    
    /// Set the linger period for socket shutdown (milliseconds).
    /// Returns error if socket is already connected.
    pub fn set_linger(&mut self, linger: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiClientConnectionState::Connected {
            return Err(FeagiNetworkError::GeneralFailure("Cannot change configuration while connected".to_string()));
        }
        self.linger = linger;
        Ok(())
    }
    
    /// Set the receive high water mark (message queue size).
    /// Returns error if socket is already connected.
    pub fn set_rcvhwm(&mut self, rcvhwm: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiClientConnectionState::Connected {
            return Err(FeagiNetworkError::GeneralFailure("Cannot change configuration while connected".to_string()));
        }
        self.rcvhwm = rcvhwm;
        Ok(())
    }
    
    /// Set the send high water mark (message queue size).
    /// Returns error if socket is already connected.
    pub fn set_sndhwm(&mut self, sndhwm: i32) -> Result<(), FeagiNetworkError> {
        if self.current_state == FeagiClientConnectionState::Connected {
            return Err(FeagiNetworkError::GeneralFailure("Cannot change configuration while connected".to_string()));
        }
        self.sndhwm = sndhwm;
        Ok(())
    }
}

impl<S> FeagiClient for FEAGIZMQClientRequester<S>
where S: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
{
    fn connect(&mut self, host: &str) -> Result<(), FeagiNetworkError> {
        // Apply configuration options
        self.socket.set_linger(self.linger)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        self.socket.set_rcvhwm(self.rcvhwm)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        self.socket.set_sndhwm(self.sndhwm)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        // Set socket to non-blocking mode for try_poll_receive
        self.socket.set_rcvtimeo(0)
            .map_err(|e| FeagiNetworkError::SocketCreationFailed(e.to_string()))?;
        self.socket.connect(host)
            .map_err(|e| FeagiNetworkError::CannotConnect(e.to_string()))?;
        
        self.server_address = host.to_string(); // Store for disconnect
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
    fn build<F>(self, state_change_callback: F) -> Box<dyn FeagiClientSubscriber>
    where F: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
    {
        let mut subscriber = FEAGIZMQClientSubscriber::new(
            self.server_address,
            state_change_callback,
        ).expect("Failed to create ZMQ subscriber");
        
        let _ = subscriber.set_linger(self.linger);
        let _ = subscriber.set_rcvhwm(self.rcvhwm);
        
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
    fn build<F>(self, state_change_callback: F) -> Box<dyn FeagiClientPusher>
    where F: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
    {
        let mut pusher = FEAGIZMQClientPusher::new(
            self.server_address,
            state_change_callback,
        ).expect("Failed to create ZMQ pusher");
        
        let _ = pusher.set_linger(self.linger);
        let _ = pusher.set_sndhwm(self.sndhwm);
        
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
    fn build<F>(self, state_change_callback: F) -> Box<dyn FeagiClientRequester>
    where F: Fn(FeagiClientConnectionStateChange) + Send + Sync + 'static
    {
        let mut requester = FEAGIZMQClientRequester::new(
            self.server_address,
            state_change_callback,
        ).expect("Failed to create ZMQ requester");
        
        let _ = requester.set_linger(self.linger);
        let _ = requester.set_rcvhwm(self.rcvhwm);
        let _ = requester.set_sndhwm(self.sndhwm);
        
        Box::new(requester)
    }
}

//endregion

//endregion
