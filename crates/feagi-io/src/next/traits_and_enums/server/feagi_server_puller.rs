use crate::next::FeagiNetworkError;
use crate::next::traits_and_enums::server::FeagiServer;

/// A server that receives pushed data from clients.
///
/// Implements the push-pull pattern where clients push data to the server.
/// The server polls for incoming data and caches it internally for zero-copy access.
pub trait FeagiServerPuller: FeagiServer {
    /// Non-blocking poll for incoming data.
    ///
    /// Checks if data is available on the socket. If data is received,
    /// it is cached internally and can be accessed via [`get_cached_data`].
    ///
    /// # Returns
    /// - `Ok(true)` - New data was received and cached (use get_cached_data to retrieve)
    /// - `Ok(false)` - No data available
    /// - `Err(...)` - An error occurred while polling
    fn try_poll(&mut self) -> Result<bool, FeagiNetworkError>;

    /// Get a reference to the last received data.
    ///
    /// Returns the data from the most recent successful [`try_poll`] call.
    /// If no data has been received yet, returns an empty slice.
    ///
    /// # Zero-Copy
    /// This returns a reference to the internal buffer, avoiding allocation.
    fn get_cached_data(&self) -> &[u8];
}
