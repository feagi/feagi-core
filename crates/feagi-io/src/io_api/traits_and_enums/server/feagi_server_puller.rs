use crate::io_api::traits_and_enums::server::FeagiServer;
use crate::io_api::FeagiNetworkError;

/// A server that receives pushed data from clients.
///
/// Implements the push-pull pattern where clients push data to the server.
/// The server polls for incoming data and caches it internally for zero-copy access.
pub trait FeagiServerPuller: FeagiServer {
    fn try_poll_receive(&mut self) -> Result<Option<&[u8]>, FeagiNetworkError>;
}
