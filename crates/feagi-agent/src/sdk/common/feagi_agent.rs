use feagi_io::io_api::traits_and_enums::client::FeagiClientRequesterProperties;
use feagi_structures::FeagiDataError;
use crate::sdk::common::agent_descriptor::AgentDescriptor;
use crate::sdk::common::common_enums::{AgentCapabilities, FeagiConnectionConfiguration};
use crate::sdk::common::common_enums::AgentConnectionState;

pub trait FeagiAgent {
    fn agent_id(&self) -> &AgentDescriptor;

    fn current_connection_state(&self) -> &AgentConnectionState;

    fn agent_capabilities(&self) -> &[AgentCapabilities];

    fn connect_to_feagi(
        &mut self,
        connection_configuration: String,
        requester_properties: Box<dyn FeagiClientRequesterProperties>,
        agent_descriptor: AgentDescriptor,
    ) -> Result<(), FeagiDataError>;

    fn disconnect(&mut self);
}