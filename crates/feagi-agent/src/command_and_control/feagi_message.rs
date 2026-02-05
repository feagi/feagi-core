use serde::{Deserialize, Serialize};
use feagi_serialization::{FeagiByteContainer, SessionID};
use feagi_structures::FeagiJSON;
use crate::command_and_control::agent_registration_message::AgentRegistrationMessage;
use crate::FeagiAgentError;

// All Command and Control messages are within this nested enum.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeagiMessage {
    HeartBeat,
    AgentRegistration(AgentRegistrationMessage),
    HealthCheck(),
    AgentConfiguration(),
}

impl FeagiMessage {
    pub fn serialize_to_byte_container(&self, container: &mut FeagiByteContainer, session_id: SessionID, increment_value: u16) -> Result<(), FeagiAgentError> {
        let json: serde_json::Value = self.into();
        let feagi_json: FeagiJSON = json.into();
        container.overwrite_byte_data_with_single_struct_data(&feagi_json, increment_value)?;
        container.set_session_id(session_id)?;
        Ok(())
    }
}

// TODO we should consider our ownh implementation for feagi messages instead of just piggybacking off of JSON
// Note: We do not get messages at a high rate. We can simply instantiate on the stack them every time
impl TryFrom<&FeagiByteContainer> for FeagiMessage {
    type Error = FeagiAgentError;
    fn try_from(value: &FeagiByteContainer) -> Result<Self, Self::Error> {
        let serialized_data = value.try_create_new_struct_from_index(0.into())?;
        let feagi_json: FeagiJSON = serialized_data.try_into()?;
        let json = feagi_json.borrow_json_value().clone();
        serde_json::from_value(json).map_err(|err| FeagiAgentError::UnableToDecodeReceivedData(err.to_string()))
    }
}

// TODO we should consider our ownh implementation for feagi messages instead of just piggybacking off of JSON
impl From<FeagiMessage> for FeagiByteContainer {
    fn from(message: FeagiMessage) -> Self {
        let json: serde_json::Value = message.into();
        let feagi_json: FeagiJSON = json.into();
        let byte_container: FeagiByteContainer = FeagiByteContainer::try_from(feagi_json).unwrap();
        byte_container
    }
}