use feagi_sensorimotor::configuration::jsonable::JSONInputOutputDefinition;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentEmbodimentConfigurationMessage {
    AgentConfigurationDetails(JSONInputOutputDefinition),
}
