use std::collections::HashMap;
use std::time::{Duration, Instant};
use feagi_io::AgentID;
use feagi_io::traits_and_enums::client::{FeagiClientPusherProperties, FeagiClientRequesterProperties, FeagiClientSubscriberProperties};
use feagi_io::traits_and_enums::shared::{FeagiEndpointState, TransportProtocolEndpoint};
use feagi_sensorimotor::ConnectorCache;
use feagi_serialization::FeagiByteContainer;
use crate::clients::{AgentRegistrationStatus, CommandControlAgent};
use crate::command_and_control::FeagiMessage;
use crate::{AgentCapabilities, AgentDescriptor, AuthToken, FeagiAgentError};
use crate::clients::blocking::motor_agent::MotorAgent;
use crate::clients::blocking::sensor_agent::SensorAgent;
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
    embodiment: ConnectorCache,
    tokio_command_control_agent: TokioCommandControlAgent,
    sensor_server: SensorAgent,
    motor_server: MotorAgent,
}

impl TokioEmbodimentAgent {

    // TODO heartbeat logic should all be moved here!


    pub async fn new_connect_and_register(
        endpoint_properties: Box<dyn FeagiClientRequesterProperties>,
        agent_descriptor: AgentDescriptor,
        auth_token: AuthToken
    ) -> Result<Self, FeagiAgentError> {

        let mut tokio_command_control_agent =
            TokioCommandControlAgent::new(endpoint_properties);

        tokio_command_control_agent.request_connect().await?;

        let requested_abilities = vec![
            AgentCapabilities::SendSensorData,
            AgentCapabilities::ReceiveMotorData
        ];

        let (agent_id, endpoints) = tokio_command_control_agent.request_registration(
            agent_descriptor,
            auth_token,
            requested_abilities
        ).await?;

        let sensor_endpoint = endpoints.get(&AgentCapabilities::SendSensorData).ok_or_else(
            || FeagiAgentError::ConnectionFailed("No sensor endpoint available".to_string())
        )?;

        let motor_endpoint = endpoints.get(&AgentCapabilities::ReceiveMotorData).ok_or_else(
            || FeagiAgentError::ConnectionFailed("No motor endpoint available".to_string())
        )?;

        let sensor_props= sensor_endpoint.create_boxed_client_pusher_properties();
        let motor_props = motor_endpoint.create_boxed_client_subscriber_properties();

        let sensor_server = Self::try_connect_sensor(sensor_props, agent_id).await?;
        let motor_server = Self::try_connect_motor(motor_props, agent_id).await?;

        Ok(TokioEmbodimentAgent {
            embodiment: ConnectorCache::new(),
            tokio_command_control_agent,
            sensor_server,
            motor_server,
        })

    }


    pub fn get_embodiment(&self) -> &ConnectorCache {
        &self.embodiment
    }

    pub fn get_embodiment_mut(&mut self) -> &mut ConnectorCache { &mut self.embodiment }

    pub async fn send_stored_sensor_data(&mut self) -> Result<(), FeagiAgentError> {
        // TODO connection checks

        let mut sensors = self.embodiment.get_sensor_cache();
        sensors.encode_all_sensors_to_neurons(Instant::now())?;
        sensors.encode_neurons_to_bytes()?;
        let bytes = sensors.get_feagi_byte_container();
        self.sensor_server.send_buffer(bytes)?;

        // Poll until we find a message
        // TODO our polling needs a more descriptive return
        loop {
            match self.sensor_server.poll()? {
                _ => return Ok(())
            }
        }
    }

    pub async fn await_motor_data(&mut self) -> Result<(), FeagiAgentError> {
        // TODO connection checks
        loop {
            match self.motor_server.poll_for_motor_data()? {
                None => {
                    // wait
                    tokio::time::sleep(Duration::from_millis(TOKIO_SLEEP_TIME_MS)).await;
                }
                Some(data) => {
                    let mut motor_cache = self.embodiment.get_motor_cache();
                    motor_cache
                        .get_feagi_byte_container_mut() // TODO some uneccessary copying going on here. We need to think about this implementation
                        .try_write_data_by_copy_and_verify(data.get_byte_ref())?;
                    let had_neural_data = motor_cache.try_decode_bytes_to_neural_data()?;
                    if had_neural_data {
                        motor_cache.try_decode_neural_data_into_cache(Instant::now())?;
                        return Ok(());
                    }
                    else {
                        // no new firings
                        tokio::time::sleep(Duration::from_millis(TOKIO_SLEEP_TIME_MS)).await;
                    }
                }
            }
        }

    }

    //region internal

    async fn try_connect_sensor(sensor_props: Box<dyn FeagiClientPusherProperties>, agent_id: AgentID) -> Result<SensorAgent, FeagiAgentError> {
        let mut sensor_agent = SensorAgent::new(sensor_props, agent_id);
        sensor_agent.request_connect()?;
        loop {
            match sensor_agent.poll() {
                Ok(_) => {
                    return Ok(sensor_agent)
                }
                Err(_) => {
                    // wait?
                    // TODO we need to return something better to work with
                    tokio::time::sleep(Duration::from_millis(TOKIO_SLEEP_TIME_MS)).await;
                }
            }
        }
    }

    async fn try_connect_motor(motor_props: Box<dyn FeagiClientSubscriberProperties>, agent_id: AgentID) -> Result<MotorAgent, FeagiAgentError> {
        let mut motor_agent = MotorAgent::new(motor_props, agent_id);
        motor_agent.request_connect()?;
        loop {
            match motor_agent.poll_for_motor_data() {
                Ok(_) => {
                    return Ok(motor_agent)
                }
                Err(_) => {
                    // wait?
                    // TODO we need to return something better to work with
                    tokio::time::sleep(Duration::from_millis(TOKIO_SLEEP_TIME_MS)).await;
                }
            }
        }
    }

    //endregion




}

//endregion

