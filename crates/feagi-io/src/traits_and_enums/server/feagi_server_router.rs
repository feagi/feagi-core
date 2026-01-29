use async_trait::async_trait;
use feagi_serialization::SessionID;

use crate::FeagiNetworkError;
use crate::traits_and_enums::server::FeagiServer;

/// A server that handles request-response communication with automatic client routing.
///
/// Implements the request-reply pattern where clients send requests and receive
/// responses. The server automatically tracks client identities for proper routing.
#[async_trait]
pub trait FeagiServerRouter: FeagiServer {
    /// Returns any query from a given session. Be sure to call send_response after.
    ///
    /// Returns owned data (`Vec<u8>`) to ensure object-safety with `dyn` trait objects.
    async fn try_poll_receive(&mut self) -> Result<(SessionID, Vec<u8>), FeagiNetworkError>;

    /// Send the response to a given query.
    async fn send_response(
        &mut self,
        client: SessionID,
        response: &[u8],
    ) -> Result<(), FeagiNetworkError>;
}
