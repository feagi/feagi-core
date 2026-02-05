use crate::protocol_implementations::TransportProtocolImplementation;
use crate::FeagiNetworkError;
use crate::traits_and_enums::server::FeagiServer;

/// A server that broadcasts data to all connected subscribers.
///
/// Implements the publish side of the publish-subscribe messaging pattern.
/// Data is pushed to all subscribed clients without expecting responses.
///
/// # Usage
///
/// ```ignore
/// // Ensure server is in Active* state first
/// match server.poll() {
///     FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
///         server.publish_data(&broadcast_data)?;
///     }
///     _ => { /* not ready to send */ }
/// }
/// ```
pub trait FeagiServerPublisher: FeagiServer {
    /// Broadcasts data to all connected subscribers.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw bytes to broadcast.
    ///
    /// # State Requirements
    ///
    /// The server must be in `ActiveWaiting` state.
    /// Call `poll()` first to verify the state.
    ///
    /// # Errors
    ///
    /// Returns [`FeagiNetworkError::SendFailed`] if the data cannot be sent.
    fn publish_data(&mut self, data: &[u8]) -> Result<(), FeagiNetworkError>;

    /// Creates a boxed properties object for this publisher.
    ///
    /// This allows decoupling the configuration/properties from the active
    /// publisher instance, enabling creation of new publishers with the same
    /// configuration.
    fn as_boxed_publisher_properties(&self) -> Box<dyn FeagiServerPublisherProperties>;
}

pub trait FeagiServerPublisherProperties: Send + Sync {
    /// Creates a new boxed publisher from these properties.
    fn as_boxed_server_publisher(&self) -> Box<dyn FeagiServerPublisher>;
    
    fn get_protocol(&self) -> TransportProtocolImplementation;
}
