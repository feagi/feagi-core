//! ZMQ server implementations using the poll-based trait design.
//!
//! These implementations use the `zmq` crate (C bindings) which provides
//! true non-blocking operations via `zmq::DONTWAIT`, making them compatible
//! with any async runtime or synchronous usage.
//!
//! # Memory Efficiency
//!
//! These implementations reuse `zmq::Message` objects to minimize allocations.
//! ZMQ handles internal memory pooling and zero-copy optimizations.

use std::collections::HashMap;
use std::env;

use feagi_serialization::FeagiByteContainer;
use zmq::{Context, Message, Socket};

use crate::{AgentID, FeagiNetworkError};
use crate::protocol_implementations::zmq::shared::ZmqUrl;
use crate::traits_and_enums::shared::{FeagiEndpointState, TransportProtocolEndpoint, TransportProtocolImplementation};
use crate::traits_and_enums::server::{
    FeagiServer, FeagiServerPublisher, FeagiServerPublisherProperties,
    FeagiServerPuller, FeagiServerPullerProperties,
    FeagiServerRouter, FeagiServerRouterProperties,
};

fn parse_bool_env(name: &str) -> Result<Option<bool>, FeagiNetworkError> {
    let Ok(raw) = env::var(name) else {
        return Ok(None);
    };
    let normalized = raw.trim().to_ascii_lowercase();
    let value = match normalized.as_str() {
        "1" | "true" | "yes" | "on" => true,
        "0" | "false" | "no" | "off" => false,
        _ => {
            return Err(FeagiNetworkError::InvalidSocketProperties(format!(
                "Invalid boolean value for {name}: '{raw}'"
            )));
        }
    };
    Ok(Some(value))
}

fn parse_i32_env(name: &str) -> Result<Option<i32>, FeagiNetworkError> {
    let Ok(raw) = env::var(name) else {
        return Ok(None);
    };
    let parsed = raw.trim().parse::<i32>().map_err(|_| {
        FeagiNetworkError::InvalidSocketProperties(format!(
            "Invalid integer value for {name}: '{raw}'"
        ))
    })?;
    Ok(Some(parsed))
}

fn apply_common_server_zmq_tuning(socket: &Socket) -> Result<(), FeagiNetworkError> {
    if let Some(linger_ms) = parse_i32_env("FEAGI_ZMQ_LINGER_MS")? {
        socket
            .set_linger(linger_ms)
            .map_err(|e| FeagiNetworkError::InvalidSocketProperties(e.to_string()))?;
    }
    if let Some(immediate) = parse_bool_env("FEAGI_ZMQ_IMMEDIATE")? {
        socket
            .set_immediate(immediate)
            .map_err(|e| FeagiNetworkError::InvalidSocketProperties(e.to_string()))?;
    }
    Ok(())
}

fn apply_server_send_tuning(socket: &Socket) -> Result<(), FeagiNetworkError> {
    if let Some(sndhwm) = parse_i32_env("FEAGI_ZMQ_SNDHWM")? {
        socket
            .set_sndhwm(sndhwm)
            .map_err(|e| FeagiNetworkError::InvalidSocketProperties(e.to_string()))?;
    }
    Ok(())
}

fn apply_server_receive_tuning(socket: &Socket) -> Result<(), FeagiNetworkError> {
    if let Some(rcvhwm) = parse_i32_env("FEAGI_ZMQ_RCVHWM")? {
        socket
            .set_rcvhwm(rcvhwm)
            .map_err(|e| FeagiNetworkError::InvalidSocketProperties(e.to_string()))?;
    }
    Ok(())
}

// ============================================================================
// Publisher
// ============================================================================

//region Publisher Properties

/// Configuration properties for creating a ZMQ PUB server.
///
/// This allows storing configuration separately from active instances,
/// enabling creation of new publishers with the same settings.
#[derive(Debug, Clone, PartialEq)]
pub struct FeagiZmqServerPublisherProperties {
    local_bind_address: ZmqUrl,
    remote_bind_address: ZmqUrl,
}

impl FeagiZmqServerPublisherProperties {
    /// Creates new publisher properties with the given bind address.
    ///
    /// # Arguments
    ///
    /// * `bind_address` - The ZMQ address to bind to (e.g., "tcp://*:5555").
    ///
    /// # Errors
    ///
    /// Returns an error if the address is invalid.
    pub fn new(local_bind_address: &str, remote_bind_address: &str) -> Result<Self, FeagiNetworkError> {
        let local_bind_address = ZmqUrl::new(local_bind_address)?;
        let remote_bind_address =  ZmqUrl::new(remote_bind_address)?;

        Ok(Self { local_bind_address, remote_bind_address })
    }
}

impl FeagiServerPublisherProperties for FeagiZmqServerPublisherProperties {
    fn as_boxed_server_publisher(&self) -> Box<dyn FeagiServerPublisher> {
        let context = Context::new();
        let socket = context
            .socket(zmq::PUB)
            .expect("Failed to create ZMQ PUB socket");

        Box::new(FeagiZmqServerPublisher {
            local_bind_address: self.local_bind_address.clone(),
            remote_bind_address: self.remote_bind_address.clone(),
            current_state: FeagiEndpointState::Inactive,
            context,
            socket,
        })
    }

    fn get_bind_point(&self) -> TransportProtocolEndpoint {
        TransportProtocolEndpoint::Zmq(self.local_bind_address.clone())
    }

    fn get_agent_endpoint(&self) -> TransportProtocolEndpoint {
        TransportProtocolEndpoint::Zmq(self.remote_bind_address.clone())
    }

    fn get_protocol(&self) -> TransportProtocolImplementation {
        TransportProtocolImplementation::Zmq
    }
}

//endregion

//region Publisher Implementation

/// A ZMQ PUB server that broadcasts data to all connected subscribers.
///
/// # Example
///
/// ```ignore
/// let props = FeagiZmqServerPublisherProperties::new("tcp://*:5555")?;
/// let mut publisher = props.as_boxed_server_publisher();
///
/// publisher.request_start()?;
/// while publisher.poll() == FeagiEndpointState::Pending {
///     // wait/yield
/// }
///
/// // Now active - can publish
/// publisher.publish_data(b"Hello subscribers!")?;
/// ```
pub struct FeagiZmqServerPublisher {
    local_bind_address: ZmqUrl,
    remote_bind_address: ZmqUrl,
    current_state: FeagiEndpointState,
    #[allow(dead_code)] // Context must be kept alive for socket lifetime
    context: Context,
    socket: Socket,
}

impl FeagiServer for FeagiZmqServerPublisher {
    fn poll(&mut self) -> &FeagiEndpointState {
        // For ZMQ PUB sockets, there's no incoming data to check.
        // State transitions happen synchronously in request_start/request_stop.
        &self.current_state
    }

    fn request_start(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::Inactive => {
                apply_common_server_zmq_tuning(&self.socket)?;
                apply_server_send_tuning(&self.socket)?;
                // ZMQ bind is synchronous
                self.socket
                    .bind(&self.local_bind_address.to_string())
                    .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;

                self.current_state = FeagiEndpointState::ActiveWaiting;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot start: server is not in Inactive state".to_string(),
            )),
        }
    }

    fn request_stop(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                // ZMQ unbind is synchronous
                self.socket
                    .unbind(&self.local_bind_address.to_string())
                    .map_err(|e| FeagiNetworkError::CannotUnbind(e.to_string()))?;

                self.current_state = FeagiEndpointState::Inactive;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot stop: server is not in Active state".to_string(),
            )),
        }
    }

    fn confirm_error_and_close(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::Errored(_) => {
                // Attempt to unbind if we were bound
                let _ = self.socket.unbind(&self.local_bind_address.to_string());
                self.current_state = FeagiEndpointState::Inactive;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot confirm error: server is not in Errored state".to_string(),
            )),
        }
    }

    fn get_bind_point(&self) -> TransportProtocolEndpoint {
        TransportProtocolEndpoint::Zmq(self.local_bind_address.clone())
    }

    fn get_agent_endpoint(&self) -> TransportProtocolEndpoint {
        TransportProtocolEndpoint::Zmq(self.remote_bind_address.clone())
    }

    fn get_protocol(&self) -> TransportProtocolImplementation {
        TransportProtocolImplementation::Zmq
    }
}

impl FeagiServerPublisher for FeagiZmqServerPublisher {
    fn publish_data(&mut self, data: &[u8]) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                // Use DONTWAIT for non-blocking send
                // For PUB sockets, this typically succeeds immediately (queues the message)
                self.socket
                    .send(data, zmq::DONTWAIT)
                    .map_err(|e| {
                        if e == zmq::Error::EAGAIN {
                            FeagiNetworkError::SendFailed("Socket would block".to_string())
                        } else {
                            FeagiNetworkError::SendFailed(e.to_string())
                        }
                    })?;
                Ok(())
            }
            _ => Err(FeagiNetworkError::SendFailed(
                "Cannot publish: server is not in Active state".to_string(),
            )),
        }
    }

    fn as_boxed_publisher_properties(&self) -> Box<dyn FeagiServerPublisherProperties> {
        Box::new(FeagiZmqServerPublisherProperties {
            local_bind_address: self.local_bind_address.clone(),
            remote_bind_address: self.remote_bind_address.clone(),
        })
    }
}

//endregion

// ============================================================================
// Puller
// ============================================================================

//region Puller Properties

/// Configuration properties for creating a ZMQ PULL server.
///
/// This allows storing configuration separately from active instances,
/// enabling creation of new pullers with the same settings.
#[derive(Debug, Clone, PartialEq)]
pub struct FeagiZmqServerPullerProperties {
    local_bind_address: ZmqUrl,
    remote_bind_address: ZmqUrl,
}

impl FeagiZmqServerPullerProperties {
    /// Creates new puller properties with the given bind address.
    ///
    /// # Arguments
    ///
    /// * `bind_address` - The ZMQ address to bind to (e.g., "tcp://*:5556").
    ///
    /// # Errors
    ///
    /// Returns an error if the address is invalid.
    pub fn new(local_bind_address: &str, remote_bind_address: &str) -> Result<Self, FeagiNetworkError> {
        let local_bind_address = ZmqUrl::new(local_bind_address)?;
        let remote_bind_address =  ZmqUrl::new(remote_bind_address)?;
        Ok(Self { local_bind_address, remote_bind_address })
    }
}

impl FeagiServerPullerProperties for FeagiZmqServerPullerProperties {
    fn as_boxed_server_puller(&self) -> Box<dyn FeagiServerPuller> {
        let context = Context::new();
        let socket = context
            .socket(zmq::PULL)
            .expect("Failed to create ZMQ PULL socket");

        Box::new(FeagiZmqServerPuller {
            local_bind_address: self.local_bind_address.clone(),
            remote_bind_address: self.remote_bind_address.clone(),
            current_state: FeagiEndpointState::Inactive,
            context,
            socket,
            recv_msg: Message::new(),
            latest_non_empty_valid_msg: Message::new(),
            has_data: false,
        })
    }

    fn get_bind_point(&self) -> TransportProtocolEndpoint {
        TransportProtocolEndpoint::Zmq(self.local_bind_address.clone())
    }

    fn get_agent_endpoint(&self) -> TransportProtocolEndpoint {
        TransportProtocolEndpoint::Zmq(self.remote_bind_address.clone())
    }

    fn get_protocol(&self) -> TransportProtocolImplementation {
        TransportProtocolImplementation::Zmq
    }
}

//endregion

//region Puller Implementation

/// A ZMQ PULL server that receives pushed data from clients.
///
/// Uses a reusable `zmq::Message` to minimize memory allocations.
///
/// # Example
///
/// ```ignore
/// let props = FeagiZmqServerPullerProperties::new("tcp://*:5556")?;
/// let mut puller = props.as_boxed_server_puller();
///
/// puller.request_start()?;
/// loop {
///     match puller.poll() {
///         FeagiEndpointState::ActiveHasData => {
///             let data = puller.consume_retrieved_data()?;
///             process(data);
///         }
///         FeagiEndpointState::ActiveWaiting => { /* no data */ }
///         _ => break,
///     }
/// }
/// ```
pub struct FeagiZmqServerPuller {
    local_bind_address: ZmqUrl,
    remote_bind_address: ZmqUrl,
    current_state: FeagiEndpointState,
    #[allow(dead_code)]
    context: Context,
    socket: Socket,
    /// Reusable message buffer - ZMQ handles internal memory management
    recv_msg: Message,
    /// Reusable buffer that keeps latest valid FEAGI frame with non-empty payload
    latest_non_empty_valid_msg: Message,
    /// Whether recv_msg contains valid data ready to consume
    has_data: bool,
}

impl FeagiZmqServerPuller {
    const MIN_FEAGI_FRAME_BYTES: usize = 12;
    const STRUCT_LOOKUP_BYTES_PER_ENTRY: usize = 4;

    /// Lightweight FEAGI frame sanity check used for latest-wins filtering.
    ///
    /// This avoids selecting trailing noise frames that can appear during
    /// reconnect churn and would otherwise cause downstream decode drops.
    fn is_plausible_feagi_frame(bytes: &[u8]) -> bool {
        if bytes.len() < Self::MIN_FEAGI_FRAME_BYTES {
            return false;
        }
        // Byte 0: FEAGI binary structure version
        if bytes[0] != FeagiByteContainer::CURRENT_FBS_VERSION {
            return false;
        }
        // Byte 3: number of structures in container header
        let structure_count = bytes[3] as usize;
        let min_required = Self::MIN_FEAGI_FRAME_BYTES
            + structure_count.saturating_mul(Self::STRUCT_LOOKUP_BYTES_PER_ENTRY);
        bytes.len() >= min_required
    }

    fn has_non_empty_payload(bytes: &[u8]) -> bool {
        // For sensory channels, empty containers (struct_count=0) are valid protocol frames
        // but should not eclipse meaningful sensory updates inside the same drain window.
        bytes.get(3).copied().unwrap_or(0) > 0
    }

    /// Drain all currently-queued frames and keep only the latest payload.
    ///
    /// This avoids replaying stale sensory backlog after reconnect/restart.
    /// Additionally, it keeps the latest *non-empty valid* FEAGI frame so malformed
    /// or empty trailing frames do not eclipse usable sensory payloads.
    fn try_recv_latest(&mut self) -> Result<bool, zmq::Error> {
        let mut has_latest_non_empty_valid = false;

        self.socket.recv(&mut self.recv_msg, zmq::DONTWAIT)?;
        if Self::is_plausible_feagi_frame(&self.recv_msg)
            && Self::has_non_empty_payload(&self.recv_msg)
        {
            std::mem::swap(&mut self.recv_msg, &mut self.latest_non_empty_valid_msg);
            has_latest_non_empty_valid = true;
        }

        loop {
            match self.socket.recv(&mut self.recv_msg, zmq::DONTWAIT) {
                Ok(()) => {
                    if Self::is_plausible_feagi_frame(&self.recv_msg)
                        && Self::has_non_empty_payload(&self.recv_msg)
                    {
                        std::mem::swap(&mut self.recv_msg, &mut self.latest_non_empty_valid_msg);
                        has_latest_non_empty_valid = true;
                    }
                }
                Err(zmq::Error::EAGAIN) => {
                    if has_latest_non_empty_valid {
                        std::mem::swap(
                            &mut self.recv_msg,
                            &mut self.latest_non_empty_valid_msg,
                        );
                        return Ok(true);
                    }
                    // Drop malformed/empty windows instead of emitting a frame that can
                    // transiently blank downstream visualization.
                    return Ok(false);
                }
                Err(e) => return Err(e),
            }
        }
    }
}

impl FeagiServer for FeagiZmqServerPuller {
    fn poll(&mut self) -> &FeagiEndpointState {
        // Only check for data if we're active and don't already have buffered data
        if matches!(self.current_state, FeagiEndpointState::ActiveWaiting) && !self.has_data {
            // Receive into reusable message - no allocation if message capacity is sufficient.
            // @cursor:critical-path latest-wins drain to prevent startup backlog lag.
            match self.try_recv_latest() {
                Ok(true) => {
                    self.has_data = true;
                    self.current_state = FeagiEndpointState::ActiveHasData;
                }
                Ok(false) => {
                    // No payload captured; remain waiting.
                }
                Err(zmq::Error::EAGAIN) => {
                    // No data available, stay in ActiveWaiting
                }
                Err(e) => {
                    self.current_state = FeagiEndpointState::Errored(
                        FeagiNetworkError::ReceiveFailed(e.to_string()),
                    );
                }
            }
        }
        &self.current_state
    }

    fn request_start(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::Inactive => {
                apply_common_server_zmq_tuning(&self.socket)?;
                apply_server_receive_tuning(&self.socket)?;
                self.socket
                    .bind(&self.local_bind_address.to_string())
                    .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;

                self.current_state = FeagiEndpointState::ActiveWaiting;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot start: server is not in Inactive state".to_string(),
            )),
        }
    }

    fn request_stop(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                self.socket
                    .unbind(&self.local_bind_address.to_string())
                    .map_err(|e| FeagiNetworkError::CannotUnbind(e.to_string()))?;

                self.has_data = false;
                self.current_state = FeagiEndpointState::Inactive;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot stop: server is not in Active state".to_string(),
            )),
        }
    }

    fn confirm_error_and_close(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::Errored(_) => {
                let _ = self.socket.unbind(&self.local_bind_address.to_string());
                self.has_data = false;
                self.current_state = FeagiEndpointState::Inactive;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot confirm error: server is not in Errored state".to_string(),
            )),
        }
    }

    fn get_bind_point(&self) -> TransportProtocolEndpoint {
        TransportProtocolEndpoint::Zmq(self.local_bind_address.clone())
    }

    fn get_agent_endpoint(&self) -> TransportProtocolEndpoint {
        TransportProtocolEndpoint::Zmq(self.remote_bind_address.clone())
    }

    fn get_protocol(&self) -> TransportProtocolImplementation {
        TransportProtocolImplementation::Zmq
    }
}

impl FeagiServerPuller for FeagiZmqServerPuller {
    fn consume_retrieved_data(&mut self) -> Result<&[u8], FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveHasData => {
                if self.has_data {
                    self.has_data = false;
                    self.current_state = FeagiEndpointState::ActiveWaiting;
                    // Message implements Deref<Target=[u8]>
                    Ok(&self.recv_msg)
                } else {
                    Err(FeagiNetworkError::ReceiveFailed(
                        "No data available despite ActiveHasData state".to_string(),
                    ))
                }
            }
            _ => Err(FeagiNetworkError::ReceiveFailed(
                "Cannot consume: no data available".to_string(),
            )),
        }
    }

    fn as_boxed_puller_properties(&self) -> Box<dyn FeagiServerPullerProperties> {
        Box::new(FeagiZmqServerPullerProperties {
            local_bind_address: self.local_bind_address.clone(),
            remote_bind_address: self.remote_bind_address.clone(),
        })
    }
}

//endregion

// ============================================================================
// Router
// ============================================================================

//region Router Properties

/// Configuration properties for creating a ZMQ ROUTER server.
///
/// This allows storing configuration separately from active instances,
/// enabling creation of new routers with the same settings.
#[derive(Debug, Clone, PartialEq)]
pub struct FeagiZmqServerRouterProperties {
    local_bind_address: ZmqUrl,
    remote_bind_address: ZmqUrl,
}

impl FeagiZmqServerRouterProperties {
    /// Creates new router properties with the given bind address.
    ///
    /// # Arguments
    ///
    /// * `bind_address` - The ZMQ address to bind to (e.g., "tcp://*:5557").
    ///
    /// # Errors
    ///
    /// Returns an error if the address is invalid.
    pub fn new(local_bind_address: &str, remote_bind_address: &str) -> Result<Self, FeagiNetworkError> {
        let local_bind_address = ZmqUrl::new(local_bind_address)?;
        let remote_bind_address =  ZmqUrl::new(remote_bind_address)?;
        Ok(Self { local_bind_address, remote_bind_address })
    }
}

impl FeagiServerRouterProperties for FeagiZmqServerRouterProperties {
    fn as_boxed_server_router(&self) -> Box<dyn FeagiServerRouter> {
        let context = Context::new();
        let socket = context
            .socket(zmq::ROUTER)
            .expect("Failed to create ZMQ ROUTER socket");

        Box::new(FeagiZmqServerRouter {
            local_bind_address: self.local_bind_address.clone(),
            remote_bind_address: self.remote_bind_address.clone(),
            current_state: FeagiEndpointState::Inactive,
            context,
            socket,
            // Reusable messages for multipart receive
            identity_msg: Message::new(),
            delimiter_msg: Message::new(),
            payload_msg: Message::new(),
            // Current request data
            current_session: None,
            has_data: false,
            // Session tracking
            identity_to_session: HashMap::new(),
            session_to_identity: HashMap::new(),
        })
    }

    fn get_bind_point(&self) -> TransportProtocolEndpoint {
        TransportProtocolEndpoint::Zmq(self.local_bind_address.clone())
    }

    fn get_agent_endpoint(&self) -> TransportProtocolEndpoint {
        TransportProtocolEndpoint::Zmq(self.remote_bind_address.clone())
    }

    fn get_protocol(&self) -> TransportProtocolImplementation {
        TransportProtocolImplementation::Zmq
    }
}

//endregion

//region Router Implementation

/// A ZMQ ROUTER server that handles request-response with multiple clients.
///
/// Automatically tracks client identities via [`AgentID`] for proper routing.
/// Uses reusable `zmq::Message` objects to minimize allocations.
///
/// # Example
///
/// ```ignore
/// let props = FeagiZmqServerRouterProperties::new("tcp://*:5557")?;
/// let mut router = props.as_boxed_server_router();
///
/// router.request_start()?;
/// loop {
///     match router.poll() {
///         FeagiEndpointState::ActiveHasData => {
///             let (session_id, request) = router.consume_retrieved_request()?;
///             let response = process(request);
///             router.publish_response(session_id, &response)?;
///         }
///         FeagiEndpointState::ActiveWaiting => { /* no requests */ }
///         _ => break,
///     }
/// }
/// ```
pub struct FeagiZmqServerRouter {
    local_bind_address: ZmqUrl,
    remote_bind_address: ZmqUrl,
    current_state: FeagiEndpointState,
    #[allow(dead_code)]
    context: Context,
    socket: Socket,
    /// Reusable message for identity frame
    identity_msg: Message,
    /// Reusable message for delimiter frame
    delimiter_msg: Message,
    /// Reusable message for payload frame
    payload_msg: Message,
    /// Current session ID for the buffered request
    current_session: Option<AgentID>,
    /// Whether we have valid data ready to consume
    has_data: bool,
    /// Bidirectional mapping between AgentID and ZMQ identity
    identity_to_session: HashMap<Vec<u8>, AgentID>,
    session_to_identity: HashMap<[u8; AgentID::NUMBER_BYTES], Vec<u8>>,
}

impl FeagiZmqServerRouter {
    /// Look up an existing AgentID for the given ZMQ identity.
    fn lookup_session_id(&self, identity: &[u8]) -> Option<AgentID> {
        self.identity_to_session.get(identity).copied()
    }

    /// Create and register a new AgentID for the given ZMQ identity.
    ///
    /// Uses cryptographically random session IDs to prevent enumeration attacks.
    fn create_session_id(&mut self, identity: Vec<u8>) -> AgentID {
        let session_id = AgentID::new_random();
        self.identity_to_session.insert(identity.clone(), session_id);
        self.session_to_identity.insert(*session_id.bytes(), identity);
        session_id
    }

    /// Receive a multipart message into reusable buffers.
    /// Returns true if a complete message was received.
    fn try_recv_multipart(&mut self) -> Result<bool, zmq::Error> {
        // Receive identity frame
        self.socket.recv(&mut self.identity_msg, zmq::DONTWAIT)?;

        // Check for more frames
        if !self.socket.get_rcvmore()? {
            // Single frame message - unexpected for ROUTER
            return Ok(false);
        }

        // Receive second frame (might be delimiter or payload)
        self.socket.recv(&mut self.delimiter_msg, 0)?;

        // Check if we have more frames (delimiter was empty, payload follows)
        if self.socket.get_rcvmore()? {
            // delimiter_msg was the empty delimiter, receive actual payload
            self.socket.recv(&mut self.payload_msg, 0)?;
            
            // Drain any extra frames (shouldn't happen in normal use)
            while self.socket.get_rcvmore()? {
                let mut discard = Message::new();
                self.socket.recv(&mut discard, 0)?;
            }
        } else {
            // No delimiter - delimiter_msg is actually the payload
            // Swap so payload_msg has the data
            std::mem::swap(&mut self.delimiter_msg, &mut self.payload_msg);
        }

        Ok(true)
    }
}

impl FeagiServer for FeagiZmqServerRouter {
    fn poll(&mut self) -> &FeagiEndpointState {
        if matches!(self.current_state, FeagiEndpointState::ActiveWaiting) && !self.has_data {
            match self.try_recv_multipart() {
                Ok(true) => {
                    // Successfully received a complete multipart message
                    let session_id = match self.lookup_session_id(&self.identity_msg) {
                        Some(id) => id,
                        None => {
                            // New client - allocate identity and create session
                            let identity = self.identity_msg.to_vec();
                            self.create_session_id(identity)
                        }
                    };
                    self.current_session = Some(session_id);
                    self.has_data = true;
                    self.current_state = FeagiEndpointState::ActiveHasData;
                }
                Ok(false) => {
                    // Incomplete message - treat as error
                    self.current_state = FeagiEndpointState::Errored(
                        FeagiNetworkError::ReceiveFailed("Incomplete multipart message".to_string()),
                    );
                }
                Err(zmq::Error::EAGAIN) => {
                    // No data available
                }
                Err(e) => {
                    self.current_state = FeagiEndpointState::Errored(
                        FeagiNetworkError::ReceiveFailed(e.to_string()),
                    );
                }
            }
        }
        &self.current_state
    }

    fn request_start(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::Inactive => {
                apply_common_server_zmq_tuning(&self.socket)?;
                apply_server_send_tuning(&self.socket)?;
                apply_server_receive_tuning(&self.socket)?;
                self.socket
                    .bind(&self.local_bind_address.to_string())
                    .map_err(|e| FeagiNetworkError::CannotBind(e.to_string()))?;

                self.current_state = FeagiEndpointState::ActiveWaiting;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot start: server is not in Inactive state".to_string(),
            )),
        }
    }

    fn request_stop(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                self.socket
                    .unbind(&self.local_bind_address.to_string())
                    .map_err(|e| FeagiNetworkError::CannotUnbind(e.to_string()))?;

                // Clear all state
                self.current_session = None;
                self.has_data = false;
                self.identity_to_session.clear();
                self.session_to_identity.clear();
                self.current_state = FeagiEndpointState::Inactive;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot stop: server is not in Active state".to_string(),
            )),
        }
    }

    fn confirm_error_and_close(&mut self) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::Errored(_) => {
                let _ = self.socket.unbind(&self.local_bind_address.to_string());
                self.current_session = None;
                self.has_data = false;
                self.identity_to_session.clear();
                self.session_to_identity.clear();
                self.current_state = FeagiEndpointState::Inactive;
                Ok(())
            }
            _ => Err(FeagiNetworkError::InvalidSocketProperties(
                "Cannot confirm error: server is not in Errored state".to_string(),
            )),
        }
    }

    fn get_bind_point(&self) -> TransportProtocolEndpoint {
        TransportProtocolEndpoint::Zmq(self.local_bind_address.clone())
    }

    fn get_agent_endpoint(&self) -> TransportProtocolEndpoint {
        TransportProtocolEndpoint::Zmq(self.remote_bind_address.clone())
    }

    fn get_protocol(&self) -> TransportProtocolImplementation {
        TransportProtocolImplementation::Zmq
    }
}

impl FeagiServerRouter for FeagiZmqServerRouter {
    fn consume_retrieved_request(&mut self) -> Result<(AgentID, &[u8]), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveHasData => {
                if self.has_data {
                    if let Some(session_id) = self.current_session {
                        self.has_data = false;
                        self.current_session = None;
                        self.current_state = FeagiEndpointState::ActiveWaiting;
                        Ok((session_id, &self.payload_msg))
                    } else {
                        Err(FeagiNetworkError::ReceiveFailed(
                            "No session ID despite having data".to_string(),
                        ))
                    }
                } else {
                    Err(FeagiNetworkError::ReceiveFailed(
                        "No data available despite ActiveHasData state".to_string(),
                    ))
                }
            }
            _ => Err(FeagiNetworkError::ReceiveFailed(
                "Cannot consume: no request available".to_string(),
            )),
        }
    }

    fn publish_response(
        &mut self,
        session_id: AgentID,
        message: &[u8],
    ) -> Result<(), FeagiNetworkError> {
        match &self.current_state {
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                let identity = self
                    .session_to_identity
                    .get(session_id.bytes())
                    .ok_or_else(|| {
                        FeagiNetworkError::SendFailed(format!("Unknown session ID: {:?}", session_id))
                    })?;

                // ROUTER response: [identity, empty delimiter, payload]
                let frames = &[identity.as_slice(), &[], message];
                self.socket
                    .send_multipart(frames, zmq::DONTWAIT)
                    .map_err(|e| {
                        if e == zmq::Error::EAGAIN {
                            FeagiNetworkError::SendFailed("Socket would block".to_string())
                        } else {
                            FeagiNetworkError::SendFailed(e.to_string())
                        }
                    })?;
                Ok(())
            }
            _ => Err(FeagiNetworkError::SendFailed(
                "Cannot send response: server is not in Active state".to_string(),
            )),
        }
    }

    fn as_boxed_router_properties(&self) -> Box<dyn FeagiServerRouterProperties> {
        Box::new(FeagiZmqServerRouterProperties {
            local_bind_address: self.local_bind_address.clone(),
            remote_bind_address: self.remote_bind_address.clone(),
        })
    }
}

//endregion
