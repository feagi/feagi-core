use crate::FeagiNetworkError;
use crate::traits_and_enums::client::FeagiClient;

/// A client that subscribes to data broadcast by a publisher server.
///
/// Implements the subscribe side of the publish-subscribe messaging pattern.
/// The client passively receives data pushed by the server.
///
/// # Usage
///
/// ```ignore
/// loop {
///     match client.poll() {
///         FeagiEndpointState::ActiveHasData => {
///             let data = client.consume_retrieved_data()?;
///             process(data); // Use immediately - lifetime ends after this
///         }
///         FeagiEndpointState::ActiveWaiting => { /* no data yet */ }
///         FeagiEndpointState::Errored(e) => {
///             client.confirm_error_and_close()?;
///             break;
///         }
///         _ => {}
///     }
/// }
/// ```
pub trait FeagiClientSubscriber: FeagiClient {
    /// Consumes and returns the retrieved subscription data.
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
    /// Returns an error if no data is available or if retrieval fails.
    fn consume_retrieved_data(&mut self) -> Result<&[u8], FeagiNetworkError>;
}

/// Factory trait for creating subscriber client instances from stored properties.
///
/// This enables storing client configuration separately from active instances,
/// allowing new subscribers to be created on demand with the same settings.
pub trait FeagiClientSubscriberProperties {
    /// Creates a new boxed subscriber client from these properties.
    fn as_boxed_client_subscriber(&self) -> Box<dyn FeagiClientSubscriber>;
}
