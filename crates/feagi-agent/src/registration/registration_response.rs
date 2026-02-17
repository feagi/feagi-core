use crate::registration::AgentCapabilities;
use feagi_serialization::SessionID;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum RegistrationResponse {
    FailedInvalidRequest, // This may not be sent back if the server ignores bad data
    FailedInvalidAuth, // Usually the auth token, may be the agent too. Server may not send this if configured to ignore invalid auth
    AlreadyRegistered,
    Success(SessionID, HashMap<AgentCapabilities, String>),
}
