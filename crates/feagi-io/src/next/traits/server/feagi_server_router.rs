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
/// # Usage
/// ```ignore
/// let mut server = FEAGIZMQServerRouter::new(&mut context, address)?;
/// server.start()?;
///
/// loop {
///     if server.try_poll()? {
///         let request = server.get_request_data();
///         println!("Received: {:?}", request);
///         
///         let response = process(request);
///         server.send_response(&response)?;
///     }
/// }
/// ```
pub trait FeagiServerRouter: FeagiServer {
    /// Non-blocking poll for incoming requests.
    ///
    /// Checks if a request is available on the socket. If a request is received,
    /// it is cached internally along with the client identity for routing the response.
    ///
    /// # Returns
    /// - `Ok(true)` - New request was received and cached
    /// - `Ok(false)` - No request available
    /// - `Err(...)` - An error occurred while polling
    fn try_poll(&mut self) -> Result<bool, FeagiNetworkError>;

    /// Get a reference to the last received request data.
    ///
    /// Returns the request data from the most recent successful [`try_poll`] call.
    /// If no request has been received yet, returns an empty slice.
    fn get_request_data(&self) -> &[u8];

    /// Send a response to the client who sent the last request.
    ///
    /// This uses the cached client identity from the last [`try_poll`] call
    /// to route the response to the correct client.
    ///
    /// # Arguments
    /// * `response` - The response data to send back to the client.
    ///
    /// # Errors
    /// Returns an error if the response cannot be sent.
    fn send_response(&mut self, response: &[u8]) -> Result<(), FeagiNetworkError>;
}
