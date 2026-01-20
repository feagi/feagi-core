use crate::next::FeagiNetworkError;
use crate::next::traits::client::feagi_client::FeagiClient;

/// A client that sends requests and receives responses from a server.
///
/// Implements the request-reply pattern where the client sends a request
/// and waits for (or polls for) a response from the server.
///
/// # ZMQ Implementation
/// Uses a `DEALER` socket. Connects to a server with a `ROUTER` socket.
///
/// # Usage
/// ```ignore
/// let client = FEAGIZMQClientRequester::new(&mut context, address)?;
/// client.connect(address);
///
/// // Send a request
/// client.send_request(b"Hello server")?;
///
/// // Poll for response
/// loop {
///     if client.try_poll_response()? {
///         let response = client.get_response_data();
///         println!("Response: {:?}", response);
///         break;
///     }
/// }
/// ```
pub trait FeagiClientRequester: FeagiClient {
    /// Send a request to the server.
    ///
    /// # Arguments
    /// * `request` - The request data to send.
    ///
    /// # Errors
    /// Returns an error if the request cannot be sent.
    fn send_request(&self, request: &[u8]) -> Result<(), FeagiNetworkError>;

    /// Non-blocking poll for a response from the server.
    ///
    /// Checks if a response is available. If a response is received,
    /// it is cached internally and can be accessed via [`get_response_data`].
    ///
    /// # Returns
    /// - `Ok(true)` - Response was received and cached
    /// - `Ok(false)` - No response available yet
    /// - `Err(...)` - An error occurred while polling
    fn try_poll_response(&mut self) -> Result<bool, FeagiNetworkError>;

    /// Get a reference to the last received response data.
    ///
    /// Returns the response from the most recent successful [`try_poll_response`] call.
    /// If no response has been received yet, returns an empty slice.
    fn get_response_data(&self) -> &[u8];
}
