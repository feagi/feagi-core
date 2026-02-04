use serde::{Deserialize, Serialize};

/// Defines what type of protocol
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum ProtocolImplementation {
    WebSocket,
    ZMQ
}