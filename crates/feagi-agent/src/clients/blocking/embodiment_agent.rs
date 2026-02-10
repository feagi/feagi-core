//! Connector agent: connect to registration endpoint, register, then use returned
//! data channels (sensory, motor, optional visualization). Sensory data must be
//! sent as FeagiByteContainer bytes with the session_id set (see `session_id()` and
//! `push_sensor_data`).
//!
//! Use `connect` for ZMQ or `connect_ws` for WebSocket; flow and API are the same.

use std::time::Instant;
use feagi_io::traits_and_enums::client::{FeagiClientPusher, FeagiClientRequesterProperties, FeagiClientSubscriber};
use feagi_io::traits_and_enums::shared::TransportProtocolEndpoint;
use feagi_sensorimotor::ConnectorCache;
use crate::clients::CommandControlSubAgent;
use crate::command_and_control::agent_registration_message::{AgentRegistrationMessage, RegistrationResponse};
use crate::{AgentCapabilities, AgentDescriptor, AuthToken, FeagiAgentError};
use crate::command_and_control::FeagiMessage;

/// Established connection to FEAGI after registration: sensory push and motor
/// Build sensory payloads with the returned session_id (FeagiByteContainer) so the server accepts them.
pub struct EmbodimentAgent {
    embodiment: ConnectorCache,
    client: Option<BlockingEmbodimentClient>
}

impl EmbodimentAgent {

    pub fn new() -> Result<EmbodimentAgent, FeagiAgentError> {
        Ok(Self {
            embodiment: ConnectorCache::new(),
            client: None
        })
    }

    pub fn get_embodiment(&self) -> &ConnectorCache {
        &self.embodiment
    }

    pub fn get_embodiment_mut(&mut self) -> &mut ConnectorCache {
        &mut self.embodiment
    }

    pub fn connect_to_feagi(&mut self, feagi_registration_endpoint: Box<dyn FeagiClientRequesterProperties>, agent_descriptor: AgentDescriptor, auth_token: AuthToken) -> Result<(), FeagiAgentError> {
        let client = BlockingEmbodimentClient::new_and_generic_connect(feagi_registration_endpoint, agent_descriptor, auth_token)?;
        self.client = Some(client);
        Ok(())
    }

    pub fn poll(&mut self) -> Result<Option<FeagiMessage>, FeagiAgentError> {
        if self.client.is_none() {
            return Ok(None)
        }
        let client = self.client.as_mut().unwrap();

        // TODO actually do something with this data
        client.motor_subscriber.poll();
        client.sensor_pusher.poll();
        let possible_message = client.command_and_control.poll_for_messages()?;
        Ok(possible_message)
    }
    
    pub fn send_encoded_sensor_data(&mut self) -> Result<(), FeagiAgentError> {
        if self.client.is_none() {
            return Err(FeagiAgentError::ConnectionFailed("No Connection!".to_string()))
        }
        let mut sensors = self.embodiment.get_sensor_cache();
        sensors.encode_all_sensors_to_neurons(Instant::now())?;
        sensors.encode_neurons_to_bytes()?;
        let bytes = sensors.get_feagi_byte_container();
        let client = self.client.as_mut().unwrap();
        client.sensor_pusher.publish_data(bytes.get_byte_ref())?;
        Ok(())
    }
    
    // TODO how can we handle motor callback hookups? 
    

}

struct BlockingEmbodimentClient {
    command_and_control: CommandControlSubAgent,
    sensor_pusher: Box<dyn FeagiClientPusher>,
    motor_subscriber: Box<dyn FeagiClientSubscriber>,
}

impl BlockingEmbodimentClient {

    pub fn new_and_generic_connect(command_and_control_properties: Box<dyn FeagiClientRequesterProperties>, agent_descriptor: AgentDescriptor, auth_token: AuthToken) -> Result<Self, FeagiAgentError> {

        let requested_capabilities = vec![AgentCapabilities::ReceiveMotorData, AgentCapabilities::SendSensorData];

        let mut command_control = CommandControlSubAgent::new(command_and_control_properties);

        command_control.request_connect()?; // TODO shouldn't this be blocking somehow?

        command_control.request_registration(agent_descriptor, auth_token, requested_capabilities)?;

        // NOTE blocking!
        loop {
            let data = command_control.poll_for_messages()?;
            if let Some(message) = data {
                // We are looking only for registration response. Anything else is invalid
                match &message {
                    FeagiMessage::AgentRegistration(registration_message) => {
                        match registration_message {
                            AgentRegistrationMessage::ClientRequestRegistration(_) => {
                                // wtf
                                return Err(FeagiAgentError::ConnectionFailed("Server cannot register to client as a client!".to_string()))
                            }
                            AgentRegistrationMessage::ServerRespondsRegistration(registration_response) => {
                                match registration_response {
                                    RegistrationResponse::FailedInvalidRequest => {
                                        return Err(FeagiAgentError::UnableToDecodeReceivedData("Unable to connect due to invalid request".to_string()))
                                    }
                                    RegistrationResponse::FailedInvalidAuth => {
                                        return Err(FeagiAgentError::AuthenticationFailed("Unable to connect due to invalid auth".to_string()))
                                    }
                                    RegistrationResponse::AlreadyRegistered => {
                                        return Err(FeagiAgentError::ConnectionFailed("Unable to connect due to agent already being registered".to_string()))
                                    }
                                    RegistrationResponse::Success(_, connection_endpoints) => {
                                        // We already handled the details within the struct


                                        let sensor_pusher_endpoint = connection_endpoints.get(&AgentCapabilities::SendSensorData).ok_or_else(|| FeagiAgentError::ConnectionFailed("unable to get sensor endpoint!".to_string()))?;
                                        let motor_pusher_endpoint = connection_endpoints.get(&AgentCapabilities::ReceiveMotorData).ok_or_else(|| FeagiAgentError::ConnectionFailed("unable to get motor endpoint!".to_string()))?;

                                        let sensor_pusher_properties = TransportProtocolEndpoint::create_boxed_client_pusher_properties(sensor_pusher_endpoint);
                                        let motor_subscriber_properties = TransportProtocolEndpoint::create_boxed_client_subscriber_properties(motor_pusher_endpoint);

                                        let mut sensor_server = sensor_pusher_properties.as_boxed_client_pusher();
                                        let mut motor_server = motor_subscriber_properties.as_boxed_client_subscriber();

                                        // TODO wait to confirm connection?
                                        sensor_server.request_connect()?;
                                        motor_server.request_connect()?;

                                        return Ok(
                                            BlockingEmbodimentClient {
                                                command_and_control: command_control,
                                                sensor_pusher: sensor_server,
                                                motor_subscriber: motor_server,
                                            }
                                        )
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        return Err(FeagiAgentError::ConnectionFailed("Invalid message received".to_string()))
                    }
                }
            }

            // TODO timeout?
        }






    }
}