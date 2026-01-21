use crate::next::FeagiNetworkError;
use crate::next::traits_and_enums::server::FeagiServer;
use crate::next::traits_and_enums::server::server_shared::ClientId;

/// A server that receives pushed data from clients.
///
/// Implements the push-pull pattern where clients push data to the server.
/// The server polls for incoming data and caches it internally for zero-copy access.
pub trait FeagiServerPuller: FeagiServer {
    fn try_poll_receive(&mut self) -> Result<Option<(ClientId, &[u8])>, FeagiNetworkError>;
}
