use crate::feagi_agent_server_error::FeagiAgentServerError;
use crate::registration::RegistrationRequest;

pub trait AgentAuth: Send + Sync {
    fn verify_agent_allowed_to_connect(
        &mut self,
        request: &RegistrationRequest,
    ) -> Result<(), FeagiAgentServerError>;
}
