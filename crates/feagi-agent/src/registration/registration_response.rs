use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use feagi_serialization::SessionID;
use crate::registration::AgentCapabilities;

#[derive(Debug, Serialize, Deserialize)]
pub enum RegistrationResponse {
    FailedInvalidRequest, // This may not be sent back if the server ignores bad data
    FailedInvalidAuth, // Usually the auth token, may be the agent too. Server may not send this if configured to ignore invalid auth
    Success(SessionID, HashMap<AgentCapabilities, String>)
}