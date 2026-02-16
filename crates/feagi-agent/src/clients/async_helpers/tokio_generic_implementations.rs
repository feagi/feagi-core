//! Tokio runtime helpers (adapter layer) for `feagi-agent`.
//!
//! These helpers are intentionally runtime-specific and feature-gated.
//! They drive the runtime-agnostic `SessionStateMachine` and execute the produced
//! actions using poll-based `feagi-io` client implementations.
//!
//! Design constraints:
//! - No hardcoded sleep intervals or timeouts: timing comes from `TokioDriverConfig`
//! - ZMQ and WebSocket are first-class via `TransportProtocolEndpoint`
//!
//! @cursor:critical-path

use std::time::{Duration, Instant};

use feagi_io::traits_and_enums::client::FeagiClientRequesterProperties;
use feagi_io::traits_and_enums::client::{FeagiClientPusher, FeagiClientSubscriber};
use feagi_io::traits_and_enums::shared::FeagiEndpointState;
use feagi_sensorimotor::ConnectorCache;

use crate::clients::{NowMs, SessionAction, SessionEvent, SessionInit, SessionPhase, SessionStateMachine, SessionTimingConfig};
use crate::{AgentCapabilities, AgentDescriptor, AuthToken, FeagiAgentError};

/// Tokio driver policy (provided by caller/config).
#[derive(Debug, Clone)]
pub struct TokioDriverConfig {
    /// Poll cadence used by async loops in this module.
    pub poll_interval: Duration,
    /// Timing policy forwarded into the runtime-agnostic state machine.
    pub timing: SessionTimingConfig,
}

/// Tokio adapter over the runtime-agnostic session state machine.
///
/// This type is appropriate for desktop/server apps (e.g., Tauri) that already run Tokio.
pub struct TokioEmbodimentAgent {
    sm: SessionStateMachine,
    driver: TokioDriverConfig,
    base: Instant,

    control: crate::clients::CommandControlAgent,
    sensor_pusher: Option<Box<dyn FeagiClientPusher>>,
    motor_subscriber: Option<Box<dyn FeagiClientSubscriber>>,

    embodiment: ConnectorCache,
}

impl TokioEmbodimentAgent {
    /// Create a new, unconnected agent session.
    ///
    /// This constructor does not perform any network I/O. Call `connect_and_register_spin()`
    /// (sync, busy-wait) or use `new_connect_and_register()` (async) to establish a session.
    pub fn new_unconnected(
        registration_endpoint: Box<dyn FeagiClientRequesterProperties>,
        agent_descriptor: AgentDescriptor,
        auth_token: AuthToken,
        requested_capabilities: Vec<AgentCapabilities>,
        driver: TokioDriverConfig,
    ) -> Self {
        let init = SessionInit {
            agent_descriptor,
            auth_token,
            requested_capabilities,
            timing: driver.timing.clone(),
        };
        Self {
            sm: SessionStateMachine::new(init),
            driver,
            base: Instant::now(),
            control: crate::clients::CommandControlAgent::new(registration_endpoint),
            sensor_pusher: None,
            motor_subscriber: None,
            embodiment: ConnectorCache::new(),
        }
    }

    /// Synchronously connect and register by driving the session state machine in a busy loop.
    ///
    /// This method performs **no sleeping**. It is deterministic and leaves scheduling policy
    /// to the caller/OS. It is suitable for environments that cannot `await` (or prefer explicit
    /// scheduling), but it may spin briefly while waiting for network progress.
    pub fn connect_and_register_spin(&mut self) -> Result<(), FeagiAgentError> {
        let mut pending_actions = self.sm.start_connect(0);
        loop {
            self.execute_actions(&pending_actions)?;
            pending_actions = self.poll_and_step()?;

            match self.sm.phase() {
                SessionPhase::Active => return Ok(()),
                SessionPhase::Failed => {
                    return Err(FeagiAgentError::ConnectionFailed(
                        self.sm.last_error().unwrap_or("session failed").to_string(),
                    ));
                }
                _ => {
                    std::thread::yield_now();
                }
            }
        }
    }

    /// Create, connect, and register an agent session.
    ///
    /// `driver.poll_interval` and `driver.timing` MUST come from centralized configuration.
    pub async fn new_connect_and_register(
        registration_endpoint: Box<dyn FeagiClientRequesterProperties>,
        agent_descriptor: AgentDescriptor,
        auth_token: AuthToken,
        requested_capabilities: Vec<AgentCapabilities>,
        driver: TokioDriverConfig,
    ) -> Result<Self, FeagiAgentError> {
        let mut agent = Self::new_unconnected(
            registration_endpoint,
            agent_descriptor,
            auth_token,
            requested_capabilities,
            driver,
        );

        let mut pending_actions = agent.sm.start_connect(0);
        loop {
            agent.execute_actions(&pending_actions)?;
            pending_actions = agent.poll_and_step()?;

            match agent.sm.phase() {
                SessionPhase::Active => return Ok(agent),
                SessionPhase::Failed => {
                    return Err(FeagiAgentError::ConnectionFailed(
                        agent.sm.last_error().unwrap_or("session failed").to_string(),
                    ));
                }
                _ => {
                    tokio::time::sleep(agent.driver.poll_interval).await;
                }
            }
        }
    }

    pub fn get_embodiment(&self) -> &ConnectorCache {
        &self.embodiment
    }

    pub fn get_embodiment_mut(&mut self) -> &mut ConnectorCache {
        &mut self.embodiment
    }

    /// Drive one tick of session maintenance (poll + step + execute actions).
    pub fn tick(&mut self) -> Result<(), FeagiAgentError> {
        let actions = self.poll_and_step()?;
        self.execute_actions(&actions)?;
        Ok(())
    }

    /// Encode all registered sensors and publish the payload.
    pub fn send_stored_sensor_data(&mut self) -> Result<(), FeagiAgentError> {
        // Progress session maintenance (heartbeat, channel state, etc.) deterministically.
        self.tick()?;

        let Some(pusher) = self.sensor_pusher.as_mut() else {
            return Err(FeagiAgentError::ConnectionFailed(
                "No sensory channel active".to_string(),
            ));
        };
        let session_id = self.sm.session_id().ok_or_else(|| {
            FeagiAgentError::ConnectionFailed("No session id available".to_string())
        })?;

        match pusher.poll() {
            FeagiEndpointState::ActiveWaiting => {}
            FeagiEndpointState::Inactive => {
                return Err(FeagiAgentError::UnableToSendData(
                    "Cannot send to inactive sensory socket".to_string(),
                ));
            }
            FeagiEndpointState::Pending => {
                return Err(FeagiAgentError::UnableToSendData(
                    "Cannot send to pending sensory socket".to_string(),
                ));
            }
            FeagiEndpointState::ActiveHasData => {
                return Err(FeagiAgentError::UnableToSendData(
                    "Sensory socket unexpectedly has data".to_string(),
                ));
            }
            FeagiEndpointState::Errored(e) => {
                return Err(FeagiAgentError::from(e.clone()));
            }
        }

        let mut sensors = self.embodiment.get_sensor_cache();
        sensors.encode_all_sensors_to_neurons(Instant::now())?;
        sensors.encode_neurons_to_bytes()?;
        let bytes = sensors.get_feagi_byte_container_mut();
        bytes.set_agent_identifier(session_id)?;
        pusher.publish_data(bytes.get_byte_ref())?;
        Ok(())
    }

    /// Compatibility alias for downstream code that used the old blocking API.
    pub fn send_encoded_sensor_data(&mut self) -> Result<(), FeagiAgentError> {
        self.send_stored_sensor_data()
    }

    /// Compatibility alias for downstream code that used the old blocking API.
    pub fn tick_liveness(&mut self) -> Result<(), FeagiAgentError> {
        self.tick()
    }

    /// Poll motor stream once and decode a single payload into the motor cache.
    pub fn poll_and_decode_motor_data(&mut self) -> Result<bool, FeagiAgentError> {
        self.poll_and_decode_motor_once()
    }

    /// Await and decode the next available motor frame (using driver poll interval).
    pub async fn await_motor_data(&mut self) -> Result<(), FeagiAgentError> {
        loop {
            if self.poll_and_decode_motor_once()? {
                return Ok(());
            }
            tokio::time::sleep(self.driver.poll_interval).await;
        }
    }

    /// Request graceful deregistration (best-effort).
    pub fn request_deregistration(&mut self, reason: Option<String>) -> Result<(), FeagiAgentError> {
        let actions = self.sm.start_deregister(reason);
        self.execute_actions(&actions)
    }

    fn now_ms(&self) -> NowMs {
        self.base.elapsed().as_millis() as u64
    }

    fn poll_and_step(&mut self) -> Result<Vec<SessionAction>, FeagiAgentError> {
        let mut events: Vec<SessionEvent> = Vec::new();

        if let Ok((state, message)) = self.control.poll_for_messages() {
            events.push(SessionEvent::ControlObserved {
                state: state.clone(),
                message,
            });
        }

        let sensor_state = self
            .sensor_pusher
            .as_mut()
            .map(|p| p.poll().clone())
            .unwrap_or(FeagiEndpointState::Inactive);
        let motor_state = self
            .motor_subscriber
            .as_mut()
            .map(|s| s.poll().clone())
            .unwrap_or(FeagiEndpointState::Inactive);
        events.push(SessionEvent::SensorObserved {
            state: sensor_state.clone(),
        });
        events.push(SessionEvent::MotorObserved {
            state: motor_state.clone(),
        });

        let actions = self.sm.step(self.now_ms(), &events);
        self.sm
            .try_mark_data_channels_active(&sensor_state, &motor_state);
        Ok(actions)
    }

    fn poll_and_decode_motor_once(&mut self) -> Result<bool, FeagiAgentError> {
        let Some(sub) = self.motor_subscriber.as_mut() else {
            return Ok(false);
        };
        let state = sub.poll().clone();
        match state {
            FeagiEndpointState::ActiveHasData => {
                let payload = sub.consume_retrieved_data()?.to_vec();
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
            FeagiEndpointState::Errored(e) => Err(FeagiAgentError::from(e)),
            _ => Ok(false),
        }
    }

    fn execute_actions(&mut self, actions: &[SessionAction]) -> Result<(), FeagiAgentError> {
        for action in actions {
            match action {
                SessionAction::ControlRequestConnect => {
                    self.control.request_connect()?;
                }
                SessionAction::ControlSendRegistration {
                    agent_descriptor,
                    auth_token,
                    requested_capabilities,
                } => {
                    self.control.request_registration(
                        agent_descriptor.clone(),
                        auth_token.clone(),
                        requested_capabilities.clone(),
                    )?;
                }
                SessionAction::ControlSendHeartbeat => {
                    self.control.send_heartbeat()?;
                }
                SessionAction::ControlSendDeregistration { reason } => {
                    self.control.request_deregistration(reason.clone())?;
                }
                SessionAction::SensorConnectTo { endpoint } => {
                    let props = endpoint.try_create_boxed_client_pusher_properties()?;
                    let mut pusher = props.as_boxed_client_pusher();
                    pusher.request_connect()?;
                    self.sensor_pusher = Some(pusher);
                }
                SessionAction::MotorConnectTo { endpoint } => {
                    let props = endpoint.try_create_boxed_client_subscriber_properties()?;
                    let mut sub = props.as_boxed_client_subscriber();
                    sub.request_connect()?;
                    self.motor_subscriber = Some(sub);
                }
            }
        }
        Ok(())
    }
}

