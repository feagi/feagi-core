use async_trait::async_trait;

use crate::FeagiNetworkError;
use crate::traits_and_enums::client::FeagiClient;

/// A client that pushes data to a server.
///
/// Implements the push side of the push-pull pattern.
#[async_trait]
pub trait FeagiClientPusher: FeagiClient {
    /// Push data to the server.
    ///
    /// # Arguments
    /// * `data` - The raw bytes to send.
    ///
    /// # Errors
    /// Returns [`FeagiNetworkError::SendFailed`] if the data cannot be sent.
    async fn push_data(&mut self, data: &[u8]) -> Result<(), FeagiNetworkError>;
}
