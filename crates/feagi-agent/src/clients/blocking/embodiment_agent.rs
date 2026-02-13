//! Connector agent: connect to registration endpoint, register, then use returned
//! data channels (sensory, motor, optional visualization). Sensory data must be
//! sent as FeagiByteContainer bytes with the session_id set (see `session_id()` and
//! `push_sensor_data`).
//!
//! Use `connect` for ZMQ or `connect_ws` for WebSocket; flow and API are the same.

use crate::clients::CommandControlAgent;
use crate::command_and_control::agent_registration_message::{
    AgentRegistrationMessage, DeregistrationResponse, RegistrationResponse,
};
use crate::command_and_control::FeagiMessage;
use crate::{AgentCapabilities, AgentDescriptor, AuthToken, FeagiAgentError};
use feagi_io::traits_and_enums::client::{
    FeagiClientPusher, FeagiClientRequester, FeagiClientRequesterProperties, FeagiClientSubscriber,
};
use feagi_io::traits_and_enums::shared::FeagiEndpointState;
use feagi_io::traits_and_enums::shared::TransportProtocolEndpoint;
use feagi_io::AgentID;
use feagi_sensorimotor::ConnectorCache;
use feagi_serialization::FeagiByteContainer;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const DEFAULT_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

// TODO the entire heartbeat thread system must be removed

/// Optional background heartbeat helper state.
///
/// This is a convenience mode layered on top of the deterministic tick-driven
/// liveness core. The background thread uses a dedicated command/control requester
/// so it does not contend with the main data-plane loop.
struct BackgroundHeartbeatHandle {
    stop_flag: Arc<AtomicBool>,
    join_handle: Option<JoinHandle<()>>,
}

impl BackgroundHeartbeatHandle {
    fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::Release);
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

/// Established connection to FEAGI after registration: sensory push and motor
/// Build sensory payloads with the returned session_id (FeagiByteContainer) so the server accepts them.
pub struct EmbodimentAgent {
    embodiment: ConnectorCache,





    client: Option<BlockingEmbodimentClient>,
    heartbeat_interval: Duration,
    implicit_background_heartbeat: bool,
}

impl EmbodimentAgent {
    /// Create a new embodiment agent using the default heartbeat interval.
    ///
    /// The default interval is conservative and can be overridden via
    /// `set_heartbeat_interval`. FEAGI runtime callers should source this value
    /// from centralized configuration (`feagi_configuration.toml`).
    pub fn new() -> Result<EmbodimentAgent, FeagiAgentError> {
        Ok(Self {
            embodiment: ConnectorCache::new(),
            client: None,
            heartbeat_interval: DEFAULT_HEARTBEAT_INTERVAL,
            implicit_background_heartbeat: true,
        })
    }

    pub fn get_embodiment(&self) -> &ConnectorCache {
        &self.embodiment
    }

    pub fn get_embodiment_mut(&mut self) -> &mut ConnectorCache {
        &mut self.embodiment
    }

    /// Set the tick/background heartbeat interval used after connection.
    ///
    /// This call is deterministic and does not start heartbeat traffic by itself.
    /// Use `tick_liveness` for explicit/tick-driven mode or
    /// `start_background_heartbeat` for optional convenience mode.
    pub fn set_heartbeat_interval(
        &mut self,
        heartbeat_interval: Duration,
    ) -> Result<(), FeagiAgentError> {
        if heartbeat_interval.is_zero() {
            return Err(FeagiAgentError::ConnectionFailed(
                "Heartbeat interval must be greater than zero".to_string(),
            ));
        }
        self.heartbeat_interval = heartbeat_interval;
        if let Some(client) = self.client.as_mut() {
            client.heartbeat_interval = heartbeat_interval;
        }
        Ok(())
    }

    /// Control whether heartbeat starts automatically after successful connect.
    ///
    /// Default is `true` to prevent missed heartbeat setup in application code.
    /// When set to `false`, call `tick_liveness()` or `start_background_heartbeat()`
    /// explicitly.
    pub fn set_implicit_background_heartbeat(&mut self, enabled: bool) {
        self.implicit_background_heartbeat = enabled;
    }

    pub fn connect_to_feagi(
        &mut self,
        feagi_registration_endpoint: Box<dyn FeagiClientRequesterProperties>,
        agent_descriptor: AgentDescriptor,
        auth_token: AuthToken,
    ) -> Result<(), FeagiAgentError> {
        let mut client = BlockingEmbodimentClient::new_and_generic_connect(
            feagi_registration_endpoint,
            agent_descriptor,
            auth_token,
            self.heartbeat_interval,
        )?;
        if self.implicit_background_heartbeat {
            client.start_background_heartbeat()?;
        }
        self.client = Some(client);
        Ok(())
    }

    pub fn poll(&mut self) -> Result<Option<FeagiMessage>, FeagiAgentError> {
        if self.client.is_none() {
            return Ok(None);
        }
        let client = self.client.as_mut().unwrap();

        // TODO actually do something with this data
        client.motor_subscriber.poll();
        client.sensor_pusher.poll();
        let possible_message = client.command_and_control.poll_for_messages()?;
        Ok(possible_message)
    }

    /// Tick-driven liveness update.
    ///
    /// This is the deterministic heartbeat path and is safe for RTOS-like event
    /// loops. Call this periodically; it only sends a heartbeat when the configured
    /// interval has elapsed.
    pub fn tick_liveness(&mut self) -> Result<(), FeagiAgentError> {
        let Some(client) = self.client.as_mut() else {
            return Ok(());
        };
        client.try_send_heartbeat_if_due()
    }

    /// Start optional background heartbeat convenience mode.
    ///
    /// This helper spawns a dedicated requester thread that sends heartbeat
    /// messages independently from the main poll loop.
    /// Use `tick_liveness` if you need fully explicit heartbeat control.
    pub fn start_background_heartbeat(&mut self) -> Result<(), FeagiAgentError> {
        let Some(client) = self.client.as_mut() else {
            return Err(FeagiAgentError::ConnectionFailed(
                "No connection; cannot start background heartbeat".to_string(),
            ));
        };
        if client.background_heartbeat.is_some() {
            return Ok(());
        }
        client.start_background_heartbeat()
    }

    /// Stop optional background heartbeat thread.
    pub fn stop_background_heartbeat(&mut self) {
        if let Some(client) = self.client.as_mut() {
            client.stop_background_heartbeat();
        }
    }

    /// Request voluntary deregistration and wait for server acknowledgment.
    ///
    /// This call is synchronous and blocks until FEAGI responds or timeout occurs.
    pub fn request_deregistration(
        &mut self,
        reason: Option<String>,
    ) -> Result<(), FeagiAgentError> {
        let Some(client) = self.client.as_mut() else {
            return Ok(());
        };
        client.stop_background_heartbeat();
        client
            .command_and_control
            .request_deregistration(client.session_id, reason)?;

        let timeout = Duration::from_secs(10);
        let start = Instant::now();
        loop {
            if start.elapsed() > timeout {
                return Err(FeagiAgentError::ConnectionFailed(
                    "Timed out waiting for deregistration acknowledgment".to_string(),
                ));
            }

            if let Some(message) = client.command_and_control.poll_for_messages()? {
                if let FeagiMessage::AgentRegistration(registration_message) = message {
                    if let AgentRegistrationMessage::ServerRespondsDeregistration(response) =
                        registration_message
                    {
                        match response {
                            DeregistrationResponse::Success
                            | DeregistrationResponse::NotRegistered => {
                                self.client = None;
                                return Ok(());
                            }
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn send_encoded_sensor_data(&mut self) -> Result<(), FeagiAgentError> {
        if self.client.is_none() {
            return Err(FeagiAgentError::ConnectionFailed(
                "No Connection!".to_string(),
            ));
        }
        let mut sensors = self.embodiment.get_sensor_cache();
        sensors.encode_all_sensors_to_neurons(Instant::now())?;
        sensors.encode_neurons_to_bytes()?;
        let bytes = sensors.get_feagi_byte_container();
        let client = self.client.as_mut().unwrap();
        client.sensor_pusher.publish_data(bytes.get_byte_ref())?;
        Ok(())
    }

    /// Poll the motor subscriber and decode a single motor payload into the motor cache.
    ///
    /// Returns `Ok(true)` when a motor frame was received and decoded during this call.
    /// Returns `Ok(false)` when no new motor frame is currently available.
    pub fn poll_and_decode_motor_data(&mut self) -> Result<bool, FeagiAgentError> {
        if self.client.is_none() {
            return Ok(false);
        }

        let client = self.client.as_mut().unwrap();
        let endpoint_state = client.motor_subscriber.poll().clone();
        match endpoint_state {
            FeagiEndpointState::ActiveHasData => {
                let payload = client.motor_subscriber.consume_retrieved_data()?.to_vec();
                let mut motor_cache = self.embodiment.get_motor_cache();
                motor_cache
                    .get_feagi_byte_container_mut()
                    .try_write_data_by_copy_and_verify(&payload)?;

                let had_neural_data = motor_cache.try_decode_bytes_to_neural_data()?;
                if had_neural_data {
                    motor_cache.try_decode_neural_data_into_cache(Instant::now())?;
                    return Ok(true);
                }
                Ok(false)
            }
            FeagiEndpointState::Errored(err) => Err(FeagiAgentError::from(err)),
            _ => Ok(false),
        }
    }

    // TODO how can we handle motor callback hookups?
}





struct BlockingEmbodimentClient {
    command_and_control: CommandControlAgent,
    sensor_pusher: Box<dyn FeagiClientPusher>,
    motor_subscriber: Box<dyn FeagiClientSubscriber>,
    session_id: AgentID,
    heartbeat_interval: Duration,
    last_heartbeat_sent_at: Instant,
    command_endpoint: TransportProtocolEndpoint,
    background_heartbeat: Option<BackgroundHeartbeatHandle>,
}

impl BlockingEmbodimentClient {
    pub fn new_and_generic_connect(
        command_and_control_properties: Box<dyn FeagiClientRequesterProperties>,
        agent_descriptor: AgentDescriptor,
        auth_token: AuthToken,
        heartbeat_interval: Duration,
    ) -> Result<Self, FeagiAgentError> {
        let requested_capabilities = vec![
            AgentCapabilities::ReceiveMotorData,
            AgentCapabilities::SendSensorData,
        ];

        let mut command_control = CommandControlAgent::new(command_and_control_properties);
        let command_endpoint = command_control.registered_endpoint_target();

        command_control.request_connect()?; // TODO shouldn't this be blocking somehow?

        command_control.request_registration(
            agent_descriptor,
            auth_token,
            requested_capabilities,
        )?;

        // NOTE blocking! Poll for registration response with timeout
        let timeout = Duration::from_secs(30);
        let start = Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err(FeagiAgentError::ConnectionFailed(
                    "Registration timeout - no response from FEAGI".to_string(),
                ));
            }

            let data = command_control.poll_for_messages()?;
            if let Some(message) = data {
                // We are looking only for registration response. Anything else is invalid
                match &message {
                    FeagiMessage::AgentRegistration(registration_message) => {
                        match registration_message {
                            AgentRegistrationMessage::ClientRequestRegistration(_) => {
                                // wtf
                                return Err(FeagiAgentError::ConnectionFailed(
                                    "Server cannot register to client as a client!".to_string(),
                                ));
                            }
                            AgentRegistrationMessage::ServerRespondsRegistration(
                                registration_response,
                            ) => {
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
                                    RegistrationResponse::Success(session_id, connection_endpoints) => {
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
                                                session_id: *session_id,
                                                heartbeat_interval,
                                                last_heartbeat_sent_at: Instant::now(),
                                                command_endpoint,
                                                background_heartbeat: None,
                                            }
                                        )
                                    }
                                }
                            }
                            AgentRegistrationMessage::ClientRequestDeregistration(_)
                            | AgentRegistrationMessage::ServerRespondsDeregistration(_) => {
                                return Err(FeagiAgentError::ConnectionFailed(
                                    "Unexpected deregistration message during registration handshake"
                                        .to_string(),
                                ))
                            }
                        }
                    }
                    _ => {
                        return Err(FeagiAgentError::ConnectionFailed(
                            "Invalid message received".to_string(),
                        ))
                    }
                }
            }

            // Small sleep to avoid tight loop CPU burn
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn try_send_heartbeat_if_due(&mut self) -> Result<(), FeagiAgentError> {
        if self.last_heartbeat_sent_at.elapsed() < self.heartbeat_interval {
            return Ok(());
        }
        self.command_and_control
            .send_heartbeat()?;
        self.last_heartbeat_sent_at = Instant::now();
        Ok(())
    }

    fn start_background_heartbeat(&mut self) -> Result<(), FeagiAgentError> {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = Arc::clone(&stop_flag);
        let endpoint = self.command_endpoint.clone();
        let session_id = self.session_id;
        let interval = self.heartbeat_interval;

        let join_handle = thread::Builder::new()
            .name("feagi-agent-heartbeat".to_string())
            .spawn(move || {
                let props = endpoint.create_boxed_client_requester_properties();
                let mut requester: Box<dyn FeagiClientRequester> =
                    props.as_boxed_client_requester();
                if requester.request_connect().is_err() {
                    return;
                }

                let mut last_sent = Instant::now();
                while !stop_flag_clone.load(Ordering::Acquire) {
                    if last_sent.elapsed() >= interval {
                        let heartbeat = FeagiMessage::HeartBeat;
                        let mut payload = FeagiByteContainer::new_empty();
                        if heartbeat
                            .serialize_to_byte_container(&mut payload, session_id, 0)
                            .is_err()
                        {
                            break;
                        }
                        if requester.publish_request(payload.get_byte_ref()).is_err() {
                            break;
                        }

                        // Drain reply for this REQ/REP round.
                        let wait_start = Instant::now();
                        while wait_start.elapsed() < Duration::from_secs(2) {
                            if stop_flag_clone.load(Ordering::Acquire) {
                                break;
                            }
                            match requester.poll().clone() {
                                FeagiEndpointState::ActiveHasData => {
                                    let _ = requester.consume_retrieved_response();
                                    break;
                                }
                                FeagiEndpointState::Errored(_) => {
                                    let _ = requester.confirm_error_and_close();
                                    return;
                                }
                                _ => {
                                    thread::sleep(Duration::from_millis(10));
                                }
                            }
                        }
                        last_sent = Instant::now();
                    } else {
                        thread::sleep(Duration::from_millis(10));
                    }
                }
                let _ = requester.request_disconnect();
            })
            .map_err(|err| {
                FeagiAgentError::Other(format!("Failed to start heartbeat thread: {}", err))
            })?;

        self.background_heartbeat = Some(BackgroundHeartbeatHandle {
            stop_flag,
            join_handle: Some(join_handle),
        });
        Ok(())
    }

    fn stop_background_heartbeat(&mut self) {
        if let Some(mut background_heartbeat) = self.background_heartbeat.take() {
            background_heartbeat.stop();
        }
    }
}
