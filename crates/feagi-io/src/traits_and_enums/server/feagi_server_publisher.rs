use async_trait::async_trait;

use crate::FeagiNetworkError;
use crate::traits_and_enums::server::FeagiServer;

/// A server that broadcasts data to all connected subscribers.
///
/// Implements the publish-subscribe pattern where the server pushes data
/// to any number of subscribed clients. Clients receive data passively
/// without sending requests.
#[async_trait]
pub trait FeagiServerPublisher: FeagiServer {
    /// Perform maintenance polling (e.g., accept new connections).
    ///
    /// Call this periodically to handle implementation-specific housekeeping.
    /// For some implementations (like ZMQ) this may be a no-op, while for others
    /// (like WebSocket) it accepts pending client connections.
    ///
    /// # Errors
    /// Returns an error if the polling operation fails.
    async fn poll(&mut self) -> Result<(), FeagiNetworkError>;

    /// Broadcasts data to all connected subscribers.
    ///
    /// # Arguments
    /// * `buffered_data_to_send` - The raw bytes to publish to all subscribers.
    ///
    /// # Errors
    /// Returns [`FeagiNetworkError::SendFailed`] if the data cannot be sent.
    async fn publish(&mut self, buffered_data_to_send: &[u8]) -> Result<(), FeagiNetworkError>;
}

/// Decoupling of properties into an easily passable structure, from its actual implementation
pub trait FeagiServerPublisherProperties {
    fn as_server_publisher_box(&self) -> Box<dyn FeagiServerPublisher>;
}
