//! Exists for testing. Always Passes Auth! DO NOT USE IN PRODUCTION

use crate::command_and_control::agent_registration_message::RegistrationRequest;
use crate::FeagiAgentError;
use crate::server::auth::agent_auth::AgentAuth;

pub struct DummyAuth {}

impl AgentAuth for DummyAuth {
    fn verify_agent_allowed_to_connect(&mut self, _request: &RegistrationRequest) -> Result<(), FeagiAgentError> {
        Ok(()) // Do Nothing
    }
}