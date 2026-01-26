use crate::FeagiNetworkError;
use crate::traits_and_enums::server::FeagiServer;

/// A server that receives pushed data from clients.
///
/// Implements the push-pull pattern where clients push data to the server.
/// The server polls for incoming data and caches it internally for zero-copy access.
pub trait FeagiServerPuller: FeagiServer {
    fn try_poll_receive(&mut self) -> Result<Option<&[u8]>, FeagiNetworkError>;
}
