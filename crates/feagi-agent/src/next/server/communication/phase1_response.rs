//! Phase 1 registration response (connection ID).

use serde::{Deserialize, Serialize};

use crate::next::common::ConnectionId;

/// Response sent back after successful phase 1 registration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase1Response {
    /// The connection ID assigned to this agent (base64 encoded).
    pub connection_id: String,
}

impl Phase1Response {
    /// Create a new phase 1 response.
    pub fn new(connection_id: &ConnectionId) -> Self {
        Self {
            connection_id: connection_id.to_base64(),
        }
    }

    /// Serialize to JSON bytes.
    pub fn to_json_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Phase1Response serialization should never fail")
    }
}
