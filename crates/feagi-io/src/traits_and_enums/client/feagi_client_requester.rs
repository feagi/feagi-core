use crate::FeagiNetworkError;
use crate::traits_and_enums::client::FeagiClient;
use crate::traits_and_enums::shared::{TransportProtocolEndpoint, TransportProtocolImplementation};

/// A client that sends requests and receives responses from a server.
///
/// Implements the request-reply messaging pattern where the client sends a
/// request and waits for a response from the server.
///
/// # Usage
///
/// ```ignore
/// // 1. Send request (must be in Active* state)
/// client.publish_request(&request_data)?;
///
/// // 2. Poll until response arrives
/// loop {
///     match client.poll() {
///         FeagiEndpointState::ActiveHasData => {
///             let response = client.consume_retrieved_response()?;
///             process(response); // Use immediately - lifetime ends after this
///             break;
///         }
///         FeagiEndpointState::ActiveWaiting => { /* still waiting */ }
///         FeagiEndpointState::Errored(e) => return Err(e),
///         _ => {}
///     }
/// }
/// ```
pub trait FeagiClientRequester: FeagiClient {
    /// Sends a request to the server.
    ///
    /// After calling, poll until the state becomes `ActiveHasData`, then
    /// call `consume_retrieved_response()` to get the response.
    ///
    /// # Arguments
    ///
    /// * `request` - The request data to send.
    ///
    /// # State Requirements
    ///
    /// The client must be in `ActiveWaiting` state.
    ///
    /// # Errors
    ///
    /// Returns an error if the request cannot be sent.
    fn publish_request(&mut self, request: &[u8]) -> Result<(), FeagiNetworkError>;

    /// Consumes and returns the retrieved response data.
    ///
    /// # Lifetime
    ///
    /// The returned slice is valid only for the duration of this call. The data
    /// must be copied or fully processed before calling any other method on this
    /// client, as the internal buffer may be reused.
    ///
    /// # State Requirements
    ///
    /// Only call when `poll()` returns `ActiveHasData`.
    ///
    /// # Errors
    ///
    /// Returns an error if no response is available or if retrieval fails.
    fn consume_retrieved_response(&mut self) -> Result<&[u8], FeagiNetworkError>;

    /// Creates a boxed properties object for this requester.
    ///
    /// This allows decoupling the configuration/properties from the active
    /// requester instance, enabling creation of new requesters with the same
    /// configuration.
    fn as_boxed_requester_properties(&self) -> Box<dyn FeagiClientRequesterProperties>;
}

/// Factory trait for creating requester client instances from stored properties.
///
/// This enables storing client configuration separately from active instances,
/// allowing new requesters to be created on demand with the same settings.
/// 
/// Must be Send to allow usage in multi-threaded contexts (e.g., Tauri desktop apps).
pub trait FeagiClientRequesterProperties: Send {
    /// Creates a new boxed requester client from these properties.
    fn as_boxed_client_requester(&self) -> Box<dyn FeagiClientRequester>;

    fn get_endpoint_target(&self) -> TransportProtocolEndpoint;
}
