use crate::traits_and_enums::shared::{FeagiEndpointState, TransportProtocolEndpoint, TransportProtocolImplementation};
use crate::FeagiNetworkError;

/// Base trait for all FEAGI server types.
///
/// Provides lifecycle management for server sockets using a poll-based state machine,
/// mirroring [`crate::traits_and_enums::client::FeagiClient`] on the client side. This design is
/// runtime-agnostic and works with any async executor or in synchronous contexts.
///
/// All specialized server traits ([`super::FeagiServerPublisher`],
/// [`super::FeagiServerPuller`], [`super::FeagiServerRouter`]) extend this trait.
///
/// Because those traits are consumed as `Box<dyn ...>` by their corresponding `*Properties`
/// traits, every method here must be dyn compatible.
///
/// `Send` is required because servers are owned by handler state that crosses threads: the HTTP
/// transport stores them in `ApiState`, which axum requires to be `Send + Sync`.
pub trait FeagiServer: Send {
    /// Advances the internal state machine and returns the current state.
    ///
    /// This method should be called regularly to:
    /// - Progress bind/unbind operations
    /// - Accept new connections and check for incoming data
    /// - Detect errors
    ///
    /// # Returns
    ///
    /// The current [`FeagiEndpointState`]. Check this before performing data operations
    /// to ensure the server is in a valid state.
    fn poll(&mut self) -> &FeagiEndpointState;

    /// Initiates binding the server socket to the configured address.
    ///
    /// This is a non-blocking request. After calling, poll until the state transitions
    /// from `Pending` to either `ActiveWaiting` (success) or `Errored` (failure).
    ///
    /// # Errors
    ///
    /// Returns [`FeagiNetworkError::CannotBind`] if the socket cannot bind to the address,
    /// or [`FeagiNetworkError::InvalidSocketProperties`] if the server is not `Inactive`.
    fn request_start(&mut self) -> Result<(), FeagiNetworkError>;

    /// Initiates unbinding the server socket from the address.
    ///
    /// This is a non-blocking request. After calling, poll until the state
    /// transitions to `Inactive`.
    ///
    /// # Errors
    ///
    /// Returns [`FeagiNetworkError::CannotUnbind`] if the socket cannot be unbound, or
    /// [`FeagiNetworkError::InvalidSocketProperties`] if the server is not in an active state.
    fn request_stop(&mut self) -> Result<(), FeagiNetworkError>;

    /// Acknowledges an error and closes the socket.
    ///
    /// Call this when the server is in `Errored` state to acknowledge the error and
    /// transition back to `Inactive`. This allows the server to be reused for a new
    /// bind attempt.
    ///
    /// # Errors
    ///
    /// Returns an error if the server is not in `Errored` state, or if cleanup fails.
    fn confirm_error_and_close(&mut self) -> Result<(), FeagiNetworkError>;

    /// Gets the local bind point.
    fn get_bind_point(&self) -> TransportProtocolEndpoint;

    /// Gets the bind point that is given to agents (the remote bind point).
    fn get_agent_endpoint(&self) -> TransportProtocolEndpoint;

    /// The protocol both endpoints use.
    fn get_protocol(&self) -> TransportProtocolImplementation;
}
