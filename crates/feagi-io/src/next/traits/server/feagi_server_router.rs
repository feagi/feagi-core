use crate::next::FeagiNetworkError;
use crate::next::traits::server::FeagiServer;

/// A server that handles request-response communication with automatic client routing.
///
/// Implements the request-reply pattern where clients send requests and receive
/// responses. The server automatically tracks client identities for proper routing.
///
/// # ZMQ Implementation
/// Uses a `ROUTER` socket. Clients connect with `DEALER` or `REQ` sockets.
///
/// # Construction
/// Implementations should accept a processing function in their constructor:
/// ```ignore
/// fn new(
///     context: &mut Context,
///     address: String,
///     process_request: fn(&[u8], &mut [u8]) -> Result<(), FeagiNetworkError>
/// ) -> Self
/// ```
///
/// The processing function transforms request data into response data.
pub trait FeagiServerRouter: FeagiServer {
    /// Internal method called when a request is received.
    ///
    /// Processes the request through the configured callback and sends the response.
    /// This method is typically called by the internal receive loop, not directly by users.
    ///
    /// # Arguments
    /// * `request_data` - The raw bytes received from the client.
    ///
    /// # Errors
    /// Returns an error if processing fails or the response cannot be sent.
    fn _received_request(&mut self, request_data: &[u8]) -> Result<(), FeagiNetworkError>;
}
