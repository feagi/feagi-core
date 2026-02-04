
/// Defines what type of protocol
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub enum ProtocolImplementation {
    WebSocket,
    ZMQ
}