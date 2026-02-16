
use crate::FeagiNetworkError;
use crate::traits_and_enums::client::FeagiClient;
use crate::traits_and_enums::shared::TransportProtocolEndpoint;

/// A client that pushes data to a server in a fire-and-forget pattern.
///
/// Implements the push side of the push-pull messaging pattern. Data is sent
/// to the server without expecting a response.
///
/// # Usage
///
/// ```ignore
/// // Ensure client is in Active* state first
/// match client.poll() {
///     FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
///         client.publish_data(&my_data)?;
///     }
///     _ => { /* not ready to send */ }
/// }
/// ```
pub trait FeagiClientPusher: FeagiClient {
    /// Sends data to the connected server.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw bytes to send.
    ///
    /// # State Requirements
    ///
    /// The client must be in `ActiveWaiting` state.
    /// Call `poll()` first to verify the state.
    ///
    /// # Errors
    ///
    /// Returns [`FeagiNetworkError::SendFailed`] if the data cannot be sent.
    /// Transient send failures return an error here but do not necessarily
    /// transition the client to `Errored` state.
    fn publish_data(&mut self, data: &[u8]) -> Result<(), FeagiNetworkError>;

    /// Creates a boxed properties object for this pusher.
    ///
    /// This allows decoupling the configuration/properties from the active
    /// pusher instance, enabling creation of new pushers with the same
    /// configuration.
    fn as_boxed_pusher_properties(&self) -> Box<dyn FeagiClientPusherProperties>;
}

/// Factory trait for creating pusher client instances from stored properties.
///
/// This enables storing client configuration separately from active instances,
/// allowing new pushers to be created on demand with the same settings.
pub trait FeagiClientPusherProperties {
    /// Creates a new boxed pusher client from these properties.
    fn as_boxed_client_pusher(&self) -> Box<dyn FeagiClientPusher>;

    fn get_endpoint_target(&self) -> TransportProtocolEndpoint;
}
