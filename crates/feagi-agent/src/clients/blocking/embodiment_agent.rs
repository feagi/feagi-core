//! Connector agent: connect to registration endpoint, register, then use returned
//! data channels (sensory, motor, optional visualization). Sensory data must be
//! sent as FeagiByteContainer bytes with the session_id set (see `session_id()` and
//! `push_sensor_data`).
//!
//! Use `connect` for ZMQ or `connect_ws` for WebSocket; flow and API are the same.

use std::future::Future;
use std::time::Instant;
use feagi_io::protocol_implementations::zmq::{FeagiZmqClientPusherProperties, FeagiZmqClientRequester, FeagiZmqClientRequesterProperties, FeagiZmqClientSubscriberProperties};
/*
use feagi_io::protocol_implementations::websocket::{
    FeagiWebSocketClientPusherProperties, FeagiWebSocketClientRequesterProperties,
    FeagiWebSocketClientSubscriberProperties,
};

 */
use feagi_io::traits_and_enums::client::{FeagiClientPusher, FeagiClientPusherProperties, FeagiClientRequester, FeagiClientRequesterProperties, FeagiClientSubscriber, FeagiClientSubscriberProperties};
use feagi_io::traits_and_enums::shared::FeagiEndpointState;
use feagi_sensorimotor::ConnectorCache;
use feagi_serialization::{FeagiByteContainer, SessionID};

use crate::clients::blocking::registration_agent::RegistrationAgent;
use crate::feagi_agent_server_error::FeagiAgentServerError;
use crate::FeagiAgentClientError;
use crate::registration::{AgentCapabilities, AgentDescriptor, AuthToken, RegistrationRequest, RegistrationResponse};

/// Optional device_registrations JSON for auto IPU/OPU creation (when server allows).
pub type DeviceRegistrations = Option<serde_json::Value>;

/// Established connection to FEAGI after registration: sensory push and motor
/// Build sensory payloads with the returned session_id (FeagiByteContainer) so the server accepts them.
pub struct EmbodimentAgent {
    embodiment: ConnectorCache,
    client: Option<EmbodimentClient>
}

impl EmbodimentAgent {

    pub fn new() -> Result<EmbodimentAgent, FeagiAgentClientError> {
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

    pub fn connect_to_feagi_generic(&mut self, feagi_registration_endpoint: Box<dyn FeagiClientRequesterProperties>) -> Result<(), FeagiAgentClientError> {
        if self.client.is_none() {
            let client = EmbodimentClient::new_and_generic_connect(feagi_registration_endpoint)?;
            self.client = Some(client);
        }
    }

    pub fn connect_to_feagi_zmq(&mut self, zmq_endpoint: &String) -> Result<(), FeagiAgentClientError> {
        let zmq_requester = FeagiZmqClientRequesterProperties::new(zmq_endpoint)?;
        self.connect_to_feagi_generic(Box::new(zmq_requester))?;
        Ok(())
    }

    pub fn poll(&mut self) -> Result<(), FeagiAgentServerError> {
        if self.client.is_none() {
            return Ok(())
        }
        let client = self.client.as_mut().unwrap();

        // TODO actually do something with this data
        client.motor_subscriber.poll();
        client.sensor_pusher.poll();
        client.command_and_control.poll();

        Ok(())
    }
    
    pub fn send_encoded_sensor_data(&mut self) -> Result<(), FeagiAgentServerError> {
        if self.client.is_none() {
            return Err(FeagiAgentServerError::ConnectionFailed("No Connection!".to_string()))
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

struct EmbodimentClient {
    session_id: SessionID,
    command_and_control: Box<dyn FeagiClientRequester>,
    sensor_pusher: Box<dyn FeagiClientPusher>,
    motor_subscriber: Box<dyn FeagiClientSubscriber>,
}

impl EmbodimentClient {

    pub fn new_and_generic_connect(command_and_control_properties: Box<dyn FeagiClientRequesterProperties>) -> Result<Self, FeagiAgentClientError> {

        let mut command_and_control = command_and_control_properties.as_boxed_client_requester();
        command_and_control.request_connect()?;

        // TODO this is blocking. We want to reconsider
        loop {
            let result = command_and_control.poll();
            match result {
                FeagiEndpointState::Pending => continue,
                FeagiEndpointState::ActiveWaiting || FeagiEndpointState::ActiveHasData => break,
                FeagiEndpointState::Errored(e) => e,
                _ => panic!("Unexpected state: {:?}", result),
            };
        }

        // We have a connection

        command_and_control.publish_request();
        loop {
            let result = command_and_control.poll();
            match result {
                FeagiEndpointState::Pending => continue,
                FeagiEndpointState::ActiveWaiting => continue,
                FeagiEndpointState::Errored(e) => e,
                FeagiEndpointState::ActiveHasData => break,
                _ => panic!("Unexpected state: {:?}", result),
            };
        }

        let returned_data = command_and_control.consume_retrieved_response()?;
        let mut feagi_bytes = FeagiByteContainer::new_empty();
        feagi_bytes.try_write_data_by_copy_and_verify(returned_data)?;
        let response: RegistrationResponse = (&feagi_bytes).try_into()?;

        match response {
            RegistrationResponse::Success(session_id, endpoints) => {

                // TODO How do we get the connection type?????

                return Ok(
                    EmbodimentClient {
                        session_id,
                        command_and_control: command_and_control,

                    }
                )
            }
        }





    }
}