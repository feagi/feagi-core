use std::collections::HashMap;
use std::time::Duration;
use feagi_io::AgentID;
use feagi_io::traits_and_enums::client::FeagiClientRequesterProperties;
use feagi_io::traits_and_enums::shared::{FeagiEndpointState, TransportProtocolEndpoint};
use crate::clients::CommandControlSubAgent;
use crate::command_and_control::FeagiMessage;
use crate::{AgentCapabilities, AgentDescriptor, AuthToken, FeagiAgentError};
use crate::command_and_control::agent_registration_message::{AgentRegistrationMessage, RegistrationResponse};

const TOKIO_SLEEP_TIME_MS: u64 = 1;

pub struct TokioCommandControlSubAgent {
    inner: CommandControlSubAgent
}

impl TokioCommandControlSubAgent {

    /// Creates a new unconnected agent
    pub fn new(endpoint_properties: Box<dyn FeagiClientRequesterProperties>) -> Self {
        TokioCommandControlSubAgent {
            inner: CommandControlSubAgent::new(endpoint_properties)
        }
    }

    /// Connects to the endpoint. Resolves when connection is established.
    pub async fn request_connect(&mut self) -> Result<(), FeagiAgentError> {
        _ = self.inner.request_connect()?;

        loop {
            match self.inner.poll_state()? {
                FeagiEndpointState::Pending => {
                    tokio::time::sleep(Duration::from_millis(TOKIO_SLEEP_TIME_MS)).await;
                }
                FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                    return Ok(());
                }
                FeagiEndpointState::Errored(e) => {
                    return Err(FeagiAgentError::ConnectionFailed(e.to_string()));
                }
                FeagiEndpointState::Inactive => {
                    return Err(FeagiAgentError::ConnectionFailed("Connection failed".to_string()));
                }
            }
        }
    }

    /// Register with FEAGI and wait for response.
    pub async fn register(
        &mut self,
        agent_descriptor: AgentDescriptor,
        auth_token: AuthToken,
        requested_capabilities: Vec<AgentCapabilities>,
    ) -> Result<(AgentID, HashMap<AgentCapabilities, TransportProtocolEndpoint>), FeagiAgentError> {

        // Send registration response
        self.inner.request_registration(agent_descriptor, auth_token, requested_capabilities)?;

        // Poll until we get the registration response
        loop {
            match self.inner.poll_for_messages()? {
                Some(FeagiMessage::AgentRegistration(
                         AgentRegistrationMessage::ServerRespondsRegistration(
                             RegistrationResponse::Success(session_id, endpoints)
                         )
                     )) => {
                    return Ok((session_id, endpoints));
                }
                Some(message) => {
                    todo!()
                }
                None => {
                    tokio::time::sleep(Duration::from_millis(TOKIO_SLEEP_TIME_MS)).await;
                }
            }
        }

    }

    /// Wait for the next message.
    pub async fn recv(&mut self) -> Result<FeagiMessage, FeagiAgentError> {
        loop {
            match self.inner.poll_for_messages()? {
                Some(msg) => return Ok(msg),
                None => tokio::time::sleep(Duration::from_millis(TOKIO_SLEEP_TIME_MS)).await,
            }
        }
    }



}