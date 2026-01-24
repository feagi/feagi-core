use crate::io_api::traits_and_enums::server::server_shared::ClientId;
use crate::io_api::traits_and_enums::server::FeagiServer;
use crate::io_api::FeagiNetworkError;

/// A server that handles request-response communication with automatic client routing.
///
/// Implements the request-reply pattern where clients send requests and receive
/// responses. The server automatically tracks client identities for proper routing.
pub trait FeagiServerRouter: FeagiServer {
    fn try_poll_receive(&mut self) -> Result<Option<(ClientId, &[u8])>, FeagiNetworkError>;

    fn send_response(&mut self, client: ClientId, response: &[u8])
        -> Result<(), FeagiNetworkError>;
}
