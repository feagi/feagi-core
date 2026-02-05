use crate::FeagiNetworkError;
use crate::shared::TransportProtocolImplementation;
use crate::traits_and_enums::server::FeagiServer;
use feagi_serialization::SessionID;

/// A server that handles request-response communication with multiple clients.
///
/// Implements the router side of the request-reply messaging pattern. Clients
/// send requests, the server processes them and sends responses back to the
/// specific client that made each request.
///
/// The server automatically tracks client identities via [`SessionID`] to ensure
/// responses are routed to the correct client.
pub trait FeagiServerRouter: FeagiServer {
    /// Consumes and returns the next pending request along with the client's session ID.
    ///
    /// # Lifetime
    ///
    /// The returned slice is valid only for the duration of this call. The data
    /// must be copied or fully processed before calling any other method on this
    /// server, as the internal buffer may be reused.
    ///
    /// # State Requirements
    ///
    /// Only call when `poll()` returns `ActiveHasData`.
    ///
    /// # Returns
    ///
    /// A tuple of `(SessionID, &[u8])` where the session ID identifies the client
    /// for routing the response, and the slice contains the request data.
    ///
    /// # Errors
    ///
    /// Returns an error if no request is available or if retrieval fails.
    fn consume_retrieved_request(&mut self) -> Result<(SessionID, &[u8]), FeagiNetworkError>;

    /// Sends a response to a specific client.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The client identifier from `consume_retrieved_request()`.
    /// * `message` - The response data to send.
    ///
    /// # State Requirements
    ///
    /// The server must be in `ActiveWaiting` state.
    ///
    /// # Errors
    ///
    /// Returns an error if the response cannot be sent (e.g., client disconnected).
    fn publish_response(
        &mut self,
        session_id: SessionID,
        message: &[u8],
    ) -> Result<(), FeagiNetworkError>;

    // TODO functions to add clienbts, remove clients, lock registering new clients
}

pub trait FeagiServerRouterProperties: Send + Sync {
    /// Creates a new boxed router from these properties.
    fn as_boxed_server_router(&self) -> Box<dyn FeagiServerRouter>;

    fn get_protocol(&self) -> TransportProtocolImplementation;
}