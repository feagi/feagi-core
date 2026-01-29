use async_trait::async_trait;

use crate::FeagiNetworkError;
use crate::traits_and_enums::client::FeagiClient;

/// A client that sends requests and receives responses from a server.
///
/// Implements the request-reply pattern where the client sends a request
/// and waits for (or polls for) a response from the server.
#[async_trait]
pub trait FeagiClientRequester: FeagiClient {
    /// Send a request to the server.
    ///
    /// # Arguments
    /// * `request` - The request data to send.
    ///
    /// # Errors
    /// Returns an error if the request cannot be sent.
    async fn send_request(&mut self, request: &[u8]) -> Result<(), FeagiNetworkError>;

    /// Poll after sending a request to get the response.
    ///
    /// Returns owned data (`Vec<u8>`) to ensure object-safety with `dyn` trait objects.
    async fn get_response(&mut self) -> Result<Vec<u8>, FeagiNetworkError>;
}
