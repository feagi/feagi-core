use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use feagi_serialization::{FeagiByteContainer, SessionID};
use feagi_structures::FeagiJSON;
use crate::FeagiAgentClientError;
use crate::registration::AgentCapabilities;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum RegistrationResponse {
    FailedInvalidRequest, // This may not be sent back if the server ignores bad data
    FailedInvalidAuth, // Usually the auth token, may be the agent too. Server may not send this if configured to ignore invalid auth
    AlreadyRegistered,
    Success(SessionID, HashMap<AgentCapabilities, String>),
}

impl TryFrom<&FeagiByteContainer> for RegistrationResponse {
    type Error = FeagiAgentClientError;
    fn try_from(value: &FeagiByteContainer) -> Result<Self, Self::Error> {
        let serialized_data = value.try_create_new_struct_from_index(0.into())?;
        let feagi_json: FeagiJSON = serialized_data.try_into()?;
        let json = feagi_json.borrow_json_value().clone();
        serde_json::from_value(json).map_err(|err| FeagiAgentClientError::UnableToDecodeReceivedData(err.to_string()))
    }
}