//! Exists for testing. Always Passes Auth! DO NOT USE IN PRODUCTION

use crate::feagi_agent_server_error::FeagiAgentServerError;
use crate::registration::RegistrationRequest;
use crate::server::auth::agent_auth::AgentAuth;

pub struct DummyAuth {}

impl AgentAuth for DummyAuth {
    fn verify_agent_allowed_to_connect(
        &mut self,
        _request: &RegistrationRequest,
    ) -> Result<(), FeagiAgentServerError> {
        Ok(()) // Do Nothing
    }
}
