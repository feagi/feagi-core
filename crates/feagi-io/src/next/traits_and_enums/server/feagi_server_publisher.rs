use crate::next::FeagiNetworkError;
use crate::next::traits_and_enums::server::FeagiServer;

/// A server that broadcasts data to all connected subscribers.
///
/// Implements the publish-subscribe pattern where the server pushes data
/// to any number of subscribed clients. Clients receive data passively
/// without sending requests.
pub trait FeagiServerPublisher: FeagiServer {
    /// Broadcasts data to all connected subscribers.
    ///
    /// # Arguments
    /// * `buffered_data_to_send` - The raw bytes to publish to all subscribers.
    ///
    /// # Errors
    /// Returns [`FeagiNetworkError::SendFailed`] if the data cannot be sent.
    fn publish(&mut self, buffered_data_to_send: &[u8]) -> Result<(), FeagiNetworkError>;
}
