use async_trait::async_trait;

use crate::FeagiNetworkError;
use crate::traits_and_enums::client::client_shared::FeagiClientConnectionState;

/// Base trait for all FEAGI client types.
///
/// Provides common lifecycle management for client sockets, including
/// connecting to a server and tracking the current connection state.
#[async_trait]
pub trait FeagiClient: Send + Sync {
    /// Connect to the specified host.
    ///
    /// # Arguments
    /// * `host` - The address to connect to (e.g., "tcp://127.0.0.1:5555" or "ws://localhost:8080").
    ///
    /// # Errors
    /// Returns [`FeagiNetworkError::CannotConnect`] if the connection fails.
    async fn connect(&mut self, host: &str) -> Result<(), FeagiNetworkError>;

    /// Disconnect from the server.
    ///
    /// # Errors
    /// Returns an error if disconnection fails.
    async fn disconnect(&mut self) -> Result<(), FeagiNetworkError>;

    /// Returns the current connection state.
    fn get_current_connection_state(&self) -> FeagiClientConnectionState;
}
