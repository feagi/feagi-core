use crate::next::common::agent_descriptor::AgentDescriptor;
use crate::next::common::common_enums::{AgentCapabilities, FeagiConnectionConfiguration};
use crate::next::common::common_enums::AgentConnectionState;

pub trait FeagiAgent {
    fn agent_id(&self) -> &AgentDescriptor;

    fn current_connection_state(&self) -> &AgentConnectionState;

    fn agent_capabilities(&self) -> &[AgentCapabilities];

    fn connect_to_feagi(&mut self, connection_configuration: FeagiConnectionConfiguration);

    fn disconnect(&mut self);
}