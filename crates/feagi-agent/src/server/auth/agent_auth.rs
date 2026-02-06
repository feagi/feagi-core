use crate::command_and_control::agent_registration_message::RegistrationRequest;
use crate::FeagiAgentError;

pub trait AgentAuth: Send + Sync {
    fn verify_agent_allowed_to_connect(&mut self, request: &RegistrationRequest) -> Result<(), FeagiAgentError>;
}