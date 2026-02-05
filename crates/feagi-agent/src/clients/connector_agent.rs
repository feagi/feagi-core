//! Connector agent: connect to registration endpoint, register, then use returned
//! data channels (sensory, motor, optional visualization). Sensory data must be
//! sent as FeagiByteContainer bytes with the session_id set (see `session_id()` and
//! `push_sensor_data`).
//!
//! Use `connect` for ZMQ or `connect_ws` for WebSocket; flow and API are the same.

use feagi_io::protocol_implementations::zmq::{
    FeagiZmqClientPusherProperties, FeagiZmqClientRequesterProperties,
    FeagiZmqClientSubscriberProperties,
};
use feagi_io::protocol_implementations::websocket::{
    FeagiWebSocketClientPusherProperties, FeagiWebSocketClientRequesterProperties,
    FeagiWebSocketClientSubscriberProperties,
};
use feagi_io::traits_and_enums::client::{
    FeagiClientPusher, FeagiClientPusherProperties, FeagiClientRequesterProperties,
    FeagiClientSubscriber, FeagiClientSubscriberProperties,
};
use feagi_io::FeagiNetworkError;
use feagi_serialization::SessionID;

use crate::clients::registration_agent::RegistrationAgent;
use crate::FeagiAgentClientError;
use crate::registration::{
    AgentCapabilities, AgentDescriptor, AuthToken, RegistrationRequest,
};

/// Optional device_registrations JSON for auto IPU/OPU creation (when server allows).
pub type DeviceRegistrations = Option<serde_json::Value>;

/// Established connection to FEAGI after registration: sensory push, motor (and optional viz) subscribe.
/// Build sensory payloads with the returned session_id (FeagiByteContainer) so the server accepts them.
pub struct ConnectorAgent {
    session_id: SessionID,
    sensor_pusher: Box<dyn FeagiClientPusher>,
    motor_subscriber: Box<dyn FeagiClientSubscriber>,
    viz_subscriber: Option<Box<dyn FeagiClientSubscriber>>,
}

impl ConnectorAgent {
    /// Register at the given endpoint, then connect to the returned data channels.
    /// Uses ZMQ. Pass `device_registrations` to trigger auto IPU/OPU creation when the server allows.
    pub fn connect(
        registration_endpoint: &str,
        agent_descriptor: AgentDescriptor,
        auth_token: AuthToken,
        requested_capabilities: Vec<AgentCapabilities>,
        device_registrations: DeviceRegistrations,
    ) -> Result<Self, FeagiAgentClientError> {
        use feagi_io::protocol_implementations::TransportProtocolImplementation;

        let requester_props = FeagiZmqClientRequesterProperties::new(registration_endpoint)
            .map_err(|e| FeagiAgentClientError::ConnectionFailed(e.to_string()))?;
        let mut registration_agent =
            RegistrationAgent::new(requester_props.as_boxed_client_requester());

        registration_agent.connect()?;

        let registration_request = RegistrationRequest::new(
            agent_descriptor,
            auth_token,
            requested_capabilities,
            TransportProtocolImplementation::ZMQ,
        )
        .with_device_registrations(device_registrations);

        let (session_id, endpoints) = registration_agent.try_register(registration_request)?;
        registration_agent.disconnect()?;

        let sensor_endpoint = endpoints
            .get(&AgentCapabilities::SendSensorData)
            .ok_or_else(|| {
                FeagiAgentClientError::ConnectionFailed(
                    "Server did not return sensory endpoint".to_string(),
                )
            })?;
        let motor_endpoint = endpoints
            .get(&AgentCapabilities::ReceiveMotorData)
            .ok_or_else(|| {
                FeagiAgentClientError::ConnectionFailed(
                    "Server did not return motor endpoint".to_string(),
                )
            })?;

        let mut sensor_pusher: Box<dyn FeagiClientPusher> =
            FeagiZmqClientPusherProperties::new(sensor_endpoint)
                .map_err(|e| FeagiAgentClientError::ConnectionFailed(e.to_string()))?
                .as_boxed_client_pusher();
        sensor_pusher
            .request_connect()
            .map_err(|e| FeagiAgentClientError::ConnectionFailed(e.to_string()))?;
        while !matches!(sensor_pusher.poll(), FeagiEndpointState::ActiveWaiting) {
            if matches!(sensor_pusher.poll(), FeagiEndpointState::Errored(_)) {
                return Err(FeagiAgentClientError::ConnectionFailed(
                    "Failed to connect sensory channel".to_string(),
                ));
            }
        }

        let mut motor_subscriber: Box<dyn FeagiClientSubscriber> =
            FeagiZmqClientSubscriberProperties::new(motor_endpoint)
                .map_err(|e| FeagiAgentClientError::ConnectionFailed(e.to_string()))?
                .as_boxed_client_subscriber();
        motor_subscriber
            .request_connect()
            .map_err(|e| FeagiAgentClientError::ConnectionFailed(e.to_string()))?;
        while !matches!(motor_subscriber.poll(), FeagiEndpointState::ActiveWaiting)
            && !matches!(motor_subscriber.poll(), FeagiEndpointState::ActiveHasData)
        {
            if matches!(motor_subscriber.poll(), FeagiEndpointState::Errored(_)) {
                return Err(FeagiAgentClientError::ConnectionFailed(
                    "Failed to connect motor channel".to_string(),
                ));
            }
        }

        let viz_subscriber = endpoints
            .get(&AgentCapabilities::ReceiveNeuronVisualizations)
            .and_then(|viz_endpoint| {
                let mut sub: Box<dyn FeagiClientSubscriber> =
                    FeagiZmqClientSubscriberProperties::new(viz_endpoint)
                        .ok()?
                        .as_boxed_client_subscriber();
                sub.request_connect().ok()?;
                Some(sub)
            });

        Ok(ConnectorAgent {
            session_id,
            sensor_pusher,
            motor_subscriber,
            viz_subscriber,
        })
    }

    /// Same as `connect` but over WebSocket. `registration_ws_url` must be a WebSocket URL
    /// (e.g. `ws://host:port`). Uses the same data-channel flow and API as the ZMQ connector.
    pub fn connect_ws(
        registration_ws_url: &str,
        agent_descriptor: AgentDescriptor,
        auth_token: AuthToken,
        requested_capabilities: Vec<AgentCapabilities>,
        device_registrations: DeviceRegistrations,
    ) -> Result<Self, FeagiAgentClientError> {
        use feagi_io::protocol_implementations::TransportProtocolImplementation;

        let requester_props = FeagiWebSocketClientRequesterProperties::new(registration_ws_url)
            .map_err(|e| FeagiAgentClientError::ConnectionFailed(e.to_string()))?;
        let mut registration_agent =
            RegistrationAgent::new(requester_props.as_boxed_client_requester());

        registration_agent.connect()?;

        let registration_request = RegistrationRequest::new(
            agent_descriptor,
            auth_token,
            requested_capabilities,
            TransportProtocolImplementation::WebSocket,
        )
        .with_device_registrations(device_registrations);

        let (session_id, endpoints) = registration_agent.try_register(registration_request)?;
        registration_agent.disconnect()?;

        let sensor_endpoint = endpoints
            .get(&AgentCapabilities::SendSensorData)
            .ok_or_else(|| {
                FeagiAgentClientError::ConnectionFailed(
                    "Server did not return sensory endpoint".to_string(),
                )
            })?;
        let motor_endpoint = endpoints
            .get(&AgentCapabilities::ReceiveMotorData)
            .ok_or_else(|| {
                FeagiAgentClientError::ConnectionFailed(
                    "Server did not return motor endpoint".to_string(),
                )
            })?;

        let mut sensor_pusher: Box<dyn FeagiClientPusher> =
            FeagiWebSocketClientPusherProperties::new(sensor_endpoint)
                .map_err(|e| FeagiAgentClientError::ConnectionFailed(e.to_string()))?
                .as_boxed_client_pusher();
        sensor_pusher
            .request_connect()
            .map_err(|e| FeagiAgentClientError::ConnectionFailed(e.to_string()))?;
        while !matches!(sensor_pusher.poll(), FeagiEndpointState::ActiveWaiting) {
            if matches!(sensor_pusher.poll(), FeagiEndpointState::Errored(_)) {
                return Err(FeagiAgentClientError::ConnectionFailed(
                    "Failed to connect sensory channel".to_string(),
                ));
            }
        }

        let mut motor_subscriber: Box<dyn FeagiClientSubscriber> =
            FeagiWebSocketClientSubscriberProperties::new(motor_endpoint)
                .map_err(|e| FeagiAgentClientError::ConnectionFailed(e.to_string()))?
                .as_boxed_client_subscriber();
        motor_subscriber
            .request_connect()
            .map_err(|e| FeagiAgentClientError::ConnectionFailed(e.to_string()))?;
        while !matches!(motor_subscriber.poll(), FeagiEndpointState::ActiveWaiting)
            && !matches!(motor_subscriber.poll(), FeagiEndpointState::ActiveHasData)
        {
            if matches!(motor_subscriber.poll(), FeagiEndpointState::Errored(_)) {
                return Err(FeagiAgentClientError::ConnectionFailed(
                    "Failed to connect motor channel".to_string(),
                ));
            }
        }

        let viz_subscriber = endpoints
            .get(&AgentCapabilities::ReceiveNeuronVisualizations)
            .and_then(|viz_endpoint| {
                let mut sub: Box<dyn FeagiClientSubscriber> =
                    FeagiWebSocketClientSubscriberProperties::new(viz_endpoint)
                        .ok()?
                        .as_boxed_client_subscriber();
                sub.request_connect().ok()?;
                Some(sub)
            });

        Ok(ConnectorAgent {
            session_id,
            sensor_pusher,
            motor_subscriber,
            viz_subscriber,
        })
    }

    /// Session ID assigned by the server. Use this when building sensory payloads
    /// (FeagiByteContainer) so the server accepts them.
    pub fn session_id(&self) -> SessionID {
        self.session_id
    }

    /// Send sensory data. `data` must be a complete FeagiByteContainer byte slice
    /// (including session_id in the header). Use `session_id()` and
    /// `feagi_serialization::FeagiByteContainer::set_session_id` when building the payload.
    pub fn push_sensor_data(&mut self, data: &[u8]) -> Result<(), FeagiNetworkError> {
        if !matches!(self.sensor_pusher.poll(), FeagiEndpointState::ActiveWaiting) {
            if matches!(self.sensor_pusher.poll(), FeagiEndpointState::Errored(_)) {
                return Err(FeagiNetworkError::SendFailed(
                    "Sensory channel in error state".to_string(),
                ));
            }
        }
        self.sensor_pusher.publish_data(data)
    }

    /// Poll for motor data. Returns `Some(bytes)` when data is available, `None` otherwise.
    pub fn poll_motor_data(&mut self) -> Result<Option<Vec<u8>>, FeagiNetworkError> {
        match self.motor_subscriber.poll() {
            FeagiEndpointState::ActiveHasData => {
                let slice = self.motor_subscriber.consume_retrieved_data()?;
                Ok(Some(slice.to_vec()))
            }
            FeagiEndpointState::Errored(_) => Err(FeagiNetworkError::ReceiveFailed(
                "Motor channel in error state".to_string(),
            )),
            _ => Ok(None),
        }
    }

    /// Poll for visualization data if the agent requested ReceiveNeuronVisualizations.
    pub fn poll_visualization_data(&mut self) -> Result<Option<Vec<u8>>, FeagiNetworkError> {
        let Some(ref mut sub) = self.viz_subscriber else {
            return Ok(None);
        };
        match sub.poll() {
            FeagiEndpointState::ActiveHasData => {
                let slice = sub.consume_retrieved_data()?;
                Ok(Some(slice.to_vec()))
            }
            FeagiEndpointState::Errored(_) => Err(FeagiNetworkError::ReceiveFailed(
                "Visualization channel in error state".to_string(),
            )),
            _ => Ok(None),
        }
    }
}
