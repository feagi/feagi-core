use crate::next::traits::server::FeagiServer;

/// A server that receives pushed data from clients.
///
/// Implements the push-pull pattern where clients push data to the server.
/// The server passively receives data via a callback configured at construction time.
///
/// # ZMQ Implementation
/// Uses a `PULL` socket. Clients connect with `PUSH` sockets.
///
/// # Construction
/// Implementations should accept a callback function in their constructor:
/// ```ignore
/// fn new(context: &mut Context, address: String, on_data_received: fn(&[u8])) -> Self
/// ```
///
/// The callback is invoked whenever data is received from a client.
pub trait FeagiServerPuller: FeagiServer {
    // No additional methods - data reception is handled via callback provided at construction
}
