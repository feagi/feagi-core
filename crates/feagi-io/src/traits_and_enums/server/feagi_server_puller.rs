use async_trait::async_trait;

use crate::FeagiNetworkError;
use crate::traits_and_enums::server::FeagiServer;

/// A server that receives pushed data from clients.
///
/// Implements the push-pull pattern where clients push data to the server.
/// The server polls for incoming data and returns it as owned data.
#[async_trait]
pub trait FeagiServerPuller: FeagiServer {
    /// Returns any new data from clients.
    ///
    /// Returns owned data (`Vec<u8>`) to ensure object-safety with `dyn` trait objects.
    async fn try_poll_receive(&mut self) -> Result<Vec<u8>, FeagiNetworkError>;
}
