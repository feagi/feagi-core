use crate::FeagiNetworkError;
use crate::core::traits_and_enums::FeagiEndpointState;

/// Base trait for all FEAGI client types.
///
/// Provides lifecycle management for client sockets using a poll-based state machine.
/// This design is runtime-agnostic and works with any async executor or in synchronous
/// contexts.
pub trait FeagiClient: Send {
    /// Advances the internal state machine and returns the current state.
    ///
    /// This method should be called regularly to:
    /// - Progress connection/disconnection operations
    /// - Check for incoming data
    /// - Detect errors
    ///
    /// # Returns
    ///
    /// The current [`FeagiEndpointState`]. Check this before performing data operations
    /// to ensure the client is in a valid state.
    fn poll(&mut self) -> &FeagiEndpointState;

    /// Initiates a connection to the configured server address.
    ///
    /// This is a non-blocking request. After calling, poll until the state
    /// transitions from `Pending` to either `ActiveWaiting` (success) or
    /// `Errored` (failure).
    ///
    /// # Errors
    ///
    /// Returns an error if the connection request cannot be initiated (e.g.,
    /// already connected, invalid configuration).
    fn request_connect(&mut self) -> Result<(), FeagiNetworkError>;

    /// Initiates disconnection from the server.
    ///
    /// This is a non-blocking request. After calling, poll until the state
    /// transitions to `Inactive`.
    ///
    /// # Errors
    ///
    /// Returns an error if the disconnection request cannot be initiated.
    fn request_disconnect(&mut self) -> Result<(), FeagiNetworkError>;

    /// Acknowledges an error and closes the connection.
    ///
    /// Call this when the client is in `Errored` state to acknowledge the error
    /// and transition back to `Inactive`. This allows the client to be reused
    /// for a new connection attempt.
    ///
    /// # Errors
    ///
    /// Returns an error if cleanup fails.
    fn confirm_error_and_close(&mut self) -> Result<(), FeagiNetworkError>;
}
