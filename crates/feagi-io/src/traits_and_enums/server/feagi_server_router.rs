use std::future::Future;
use feagi_serialization::SessionID;
use crate::FeagiNetworkError;
use crate::traits_and_enums::server::FeagiServer;
/// A server that handles request-response communication with automatic client routing.
///
/// Implements the request-reply pattern where clients send requests and receive
/// responses. The server automatically tracks client identities for proper routing.
pub trait FeagiServerRouter: FeagiServer {

    /// Returns any query from a given session. Be sure to call send_response after
    fn try_poll_receive(&mut self) -> impl Future<Output = Result<(SessionID, &[u8]), FeagiNetworkError>>;


    /// Send the response to a given query
    fn send_response(&mut self, client: SessionID, response: &[u8])
                     -> impl Future<Output = Result<(), FeagiNetworkError>>;
}
