use std::collections::HashMap;
use std::time::Duration;
use feagi_io::AgentID;
use feagi_io::traits_and_enums::client::FeagiClientRequesterProperties;
use feagi_io::traits_and_enums::shared::{FeagiEndpointState, TransportProtocolEndpoint};
use feagi_sensorimotor::ConnectorCache;
use crate::clients::{AgentRegistrationStatus, CommandControlAgent, EmbodimentAgent};
use crate::command_and_control::FeagiMessage;
use crate::{AgentCapabilities, AgentDescriptor, AuthToken, FeagiAgentError};
use crate::command_and_control::agent_registration_message::{AgentRegistrationMessage, DeregistrationResponse, RegistrationResponse};

const TOKIO_SLEEP_TIME_MS: u64 = 1;

//region Command and Control Agent
pub struct TokioCommandControlAgent {
    inner: CommandControlAgent,
    heartbeat_interval: Duration,
}

impl TokioCommandControlAgent {

    pub const DEFAULT_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
    pub const MIN_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(1);
    pub const MAX_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(60);

    /// Creates a new unconnected agent
    pub fn new(endpoint_properties: Box<dyn FeagiClientRequesterProperties>) -> Self {
        TokioCommandControlAgent {
            inner: CommandControlAgent::new(endpoint_properties),
            heartbeat_interval: Self::DEFAULT_HEARTBEAT_INTERVAL
        }
    }

    //region Agent Properties

    pub fn registration_status(&self) -> &AgentRegistrationStatus {
        &self.inner.registration_status()
    }

    pub fn registered_endpoint_target(&mut self) -> TransportProtocolEndpoint {
        self.inner.registered_endpoint_target()
    }

    pub fn get_heartbeat_interval(&self) -> Duration {
        self.heartbeat_interval
    }

    //endregion

    //region Helpers

    /// Connects to the endpoint. Resolves when connection is established.
    pub async fn request_connect(&mut self) -> Result<(), FeagiAgentError> {
        _ = self.inner.request_connect()?;

        // Just wait until the state has reached active
        loop {
            match self.inner.poll_for_messages()? {
                (FeagiEndpointState::Pending, _) => {
                    tokio::time::sleep(Duration::from_millis(TOKIO_SLEEP_TIME_MS)).await;
                }
                (FeagiEndpointState::ActiveWaiting, _) | (FeagiEndpointState::ActiveHasData, _) => {
                    return Ok(());
                }
                (FeagiEndpointState::Errored(e), _) => {
                    return Err(FeagiAgentError::ConnectionFailed(e.to_string()));
                }
                (FeagiEndpointState::Inactive, _) => {
                    return Err(FeagiAgentError::ConnectionFailed("Connection failed".to_string()));
                }
            }
        }
    }

    /// Register with FEAGI and wait for response.
    pub async fn request_registration(
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
                (_, Some(FeagiMessage::AgentRegistration(AgentRegistrationMessage::ServerRespondsRegistration(registration_response)))) => {
                    match registration_response {
                        RegistrationResponse::Success(id, endpoints) => {
                            return Ok((id, endpoints))
                        }
                        _ => {
                            // Anything else is a failure, how should we handle it?
                            // TODO logging?
                            return Err(FeagiAgentError::AuthenticationFailed("failed to register".to_string()))
                        }
                    }
                }
                (_, Some(other_message)) => {
                    // TODO how do we deal with another kind of message being returned here?
                    tokio::time::sleep(Duration::from_millis(TOKIO_SLEEP_TIME_MS)).await;
                }
                (_, None) => {
                    // Keep waiting
                    tokio::time::sleep(Duration::from_millis(TOKIO_SLEEP_TIME_MS)).await;
                }
            }
        }
    }

    /// Deregister with FEAGI and wait for response.
    pub async fn request_deregistration(
        &mut self,
        reason: Option<String>,
    ) -> Result<DeregistrationResponse, FeagiAgentError> {

        // Send registration response
        self.inner.request_deregistration(reason)?;

        // Poll until we get the deregistration response
        loop {
            match self.inner.poll_for_messages()? {
                (_, Some(FeagiMessage::AgentRegistration(AgentRegistrationMessage::ServerRespondsDeregistration(deregistration_response)))) => {
                    return Ok(deregistration_response)
                }
                (_, Some(other_message)) => {
                    // TODO how do we deal with another kind of message being returned here?
                    tokio::time::sleep(Duration::from_millis(TOKIO_SLEEP_TIME_MS)).await;
                }
                (_, None) => {
                    // Keep waiting
                    tokio::time::sleep(Duration::from_millis(TOKIO_SLEEP_TIME_MS)).await;
                }
            }
        }
    }


    pub async fn send_heartbeat(&mut self) -> Result<(), FeagiAgentError> {
        self.inner.send_heartbeat()?;

        // Poll until we get the heartbeat back
        loop {
            match self.inner.poll_for_messages()? {
                (_, Some(FeagiMessage::HeartBeat)) => {
                    return Ok(());
                }
                (_, Some(other_message)) => {
                    // TODO how do we deal with another kind of message being returned here?
                    tokio::time::sleep(Duration::from_millis(TOKIO_SLEEP_TIME_MS)).await;
                }
                (_, None) => {
                    // Keep waiting
                    tokio::time::sleep(Duration::from_millis(TOKIO_SLEEP_TIME_MS)).await;
                }
            }
        }
    }

    //endregion

    //region Base Functions

    pub async fn poll_for_messages(&mut self) -> Result<FeagiMessage, FeagiAgentError> {
        // Poll until we find a message
        loop {
            match self.inner.poll_for_messages()? {
                (_, Some(other_message)) => {
                    // TODO how do we deal with another kind of message being returned here?
                    return Ok(other_message)
                }
                (_, None) => {
                    // Keep waiting
                    tokio::time::sleep(Duration::from_millis(TOKIO_SLEEP_TIME_MS)).await;
                }
            }
        }
    }

    pub async fn send_message(&mut self, message: FeagiMessage, increment_value: u16) -> Result<(), FeagiAgentError> {
        self.inner.send_message(message, increment_value)?;

        // wait till the socket is no longer sending the data
        loop {
            match self.inner.poll_for_messages()? {
                (&FeagiEndpointState::ActiveWaiting, _) => {
                    return Ok(())
                }
                (_, _) => {
                    // Keep waiting
                    tokio::time::sleep(Duration::from_millis(TOKIO_SLEEP_TIME_MS)).await;
                }
            }
        }

    }

    //endregion
}

//endregion

//region Embodiment Agent

pub struct TokioEmbodimentAgent {
    inner: EmbodimentAgent,
}

impl TokioEmbodimentAgent {

    // TODO heartbeat logic should all be moved here!

    pub fn new() -> Result<Self, FeagiAgentError> {
        Ok(TokioEmbodimentAgent{
            inner: EmbodimentAgent::new()?
        })
    }

    pub fn get_embodiment(&self) -> &ConnectorCache {
        self.inner.get_embodiment()
    }

    pub fn get_embodiment_mut(&mut self) -> &mut ConnectorCache {
        self.inner.get_embodiment_mut()
    }

    pub async fn connect_to_feagi(
        &mut self,
        feagi_registration_endpoint: Box<dyn FeagiClientRequesterProperties>,
        agent_descriptor: AgentDescriptor,
        auth_token: AuthToken,
    ) -> Result<(), FeagiAgentError> {

    }


}

//endregion

