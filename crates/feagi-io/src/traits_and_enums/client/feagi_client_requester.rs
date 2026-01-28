use std::future::Future;

use crate::FeagiNetworkError;
use crate::traits_and_enums::client::FeagiClient;

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
    fn send_request(&self, request: &[u8]) -> impl Future<Output = Result<(), FeagiNetworkError>>;

    /// Poll after sending a request to get the response
    fn get_response(&mut self) -> impl Future<Output = Result<&[u8], FeagiNetworkError>>;
}
