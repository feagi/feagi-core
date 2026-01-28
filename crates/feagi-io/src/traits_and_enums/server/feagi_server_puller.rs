use std::future::Future;

use crate::FeagiNetworkError;
use crate::traits_and_enums::server::FeagiServer;

/// A server that receives pushed data from clients.
///
/// Implements the push-pull pattern where clients push data to the server.
/// The server polls for incoming data and caches it internally for zero-copy access.
pub trait FeagiServerPuller: FeagiServer {

    /// Returns any new data from clients
    fn try_poll_receive(&mut self) -> impl Future<Output = Result<&[u8], FeagiNetworkError>>;
}
