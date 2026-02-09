use serde::{Deserialize, Serialize};
use feagi_sensorimotor::configuration::jsonable::JSONInputOutputDefinition;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentEmbodimentConfigurationMessage {
    AgentConfigurationDetails(JSONInputOutputDefinition)
}