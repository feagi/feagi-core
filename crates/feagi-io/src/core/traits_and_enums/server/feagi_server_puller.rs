use crate::FeagiNetworkError;
use crate::core::traits_and_enums::server::FeagiServer;

/// A server that receives pushed data from clients.
///
/// Implements the pull side of the push-pull messaging pattern. Clients push
/// data to this server, which collects it for processing.
pub trait FeagiServerPuller: FeagiServer {
    /// Consumes and returns the retrieved data from clients.
    ///
    /// # Lifetime
    ///
    /// The returned slice is valid only for the duration of this call. The data
    /// must be copied or fully processed before calling any other method on this
    /// server, as the internal buffer may be reused.
    ///
    /// # State Requirements
    ///
    /// Only call when `poll()` returns `ActiveHasData`.
    ///
    /// # Errors
    ///
    /// Returns an error if no data is available or if retrieval fails.
    fn consume_retrieved_data(&mut self) -> Result<&[u8], FeagiNetworkError>;
}
