use crate::FeagiNetworkError;
use crate::traits_and_enums::shared::{FeagiEndpointState, TransportProtocolEndpoint, TransportProtocolImplementation};

/// Base trait for all FEAGI server types.
///
/// Provides lifecycle management for server sockets using a poll-based state machine.
/// This design is runtime-agnostic and works with any async executor or in synchronous
/// contexts.
pub trait FeagiServer: Send {
    /// Advances the internal state machine and returns the current state.
    ///
    /// This method should be called regularly to:
    /// - Progress bind/unbind operations
    /// - Accept new connections (for connection-oriented protocols)
    /// - Check for incoming data
    /// - Detect errors
    ///
    /// # Returns
    ///
    /// The current [`FeagiEndpointState`]. Check this before performing data operations
    /// to ensure the server is in a valid state.
    fn poll(&mut self) -> &FeagiEndpointState;

    /// Initiates binding to the configured address and starts listening.
    ///
    /// This is a non-blocking request. After calling, poll until the state
    /// transitions from `Pending` to either `ActiveWaiting` (success) or
    /// `Errored` (failure).
    ///
    /// # Errors
    ///
    /// Returns an error if the start request cannot be initiated (e.g.,
    /// already running, invalid configuration).
    fn request_start(&mut self) -> Result<(), FeagiNetworkError>;

    /// Initiates stopping the server and unbinding from the address.
    ///
    /// This is a non-blocking request. After calling, poll until the state
    /// transitions to `Inactive`.
    ///
    /// # Errors
    ///
    /// Returns an error if the stop request cannot be initiated.
    fn request_stop(&mut self) -> Result<(), FeagiNetworkError>;

    /// Acknowledges an error and closes the server.
    ///
    /// Call this when the server is in `Errored` state to acknowledge the error
    /// and transition back to `Inactive`. This allows the server to be reused
    /// for a new start attempt.
    ///
    /// # Errors
    ///
    /// Returns an error if cleanup fails.
    fn confirm_error_and_close(&mut self) -> Result<(), FeagiNetworkError>;

    fn get_protocol(&self) -> TransportProtocolImplementation;

    fn get_endpoint(&self) -> TransportProtocolEndpoint;
}
