use async_trait::async_trait;

use crate::FeagiNetworkError;
use crate::traits_and_enums::client::FeagiClient;

/// A client that subscribes to data from a server.
///
/// Implements the subscribe side of the publish-subscribe pattern.
#[async_trait]
pub trait FeagiClientSubscriber: FeagiClient {
    /// Get incoming data from the subscription.
    ///
    /// Returns owned data (`Vec<u8>`) to ensure object-safety with `dyn` trait objects.
    async fn get_subscribed_data(&mut self) -> Result<Vec<u8>, FeagiNetworkError>;
}
