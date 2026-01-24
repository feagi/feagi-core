use crate::io_api::traits_and_enums::client::feagi_client::FeagiClient;
use crate::io_api::FeagiNetworkError;

/// A client that sends requests and receives responses from a server.
///
/// Implements the request-reply pattern where the client sends a request
/// and waits for (or polls for) a response from the server.
pub trait FeagiClientRequester: FeagiClient {
    /// Send a request to the server.
    ///
    /// # Arguments
    /// * `request` - The request data to send.
    ///
    /// # Errors
    /// Returns an error if the request cannot be sent.
    fn send_request(&self, request: &[u8]) -> Result<(), FeagiNetworkError>;

    fn try_poll_receive(&mut self) -> Result<Option<&[u8]>, FeagiNetworkError>;
}
