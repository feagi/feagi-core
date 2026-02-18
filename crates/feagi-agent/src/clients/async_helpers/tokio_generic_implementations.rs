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

use crate::clients::{
    NowMs, SessionAction, SessionEvent, SessionInit, SessionPhase, SessionStateMachine,
    SessionTimingConfig,
};
use crate::{AgentCapabilities, AgentDescriptor, AuthToken, FeagiAgentError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensoryRateNegotiationPolicy {
    Strict,
    CapAndWarn,
}

#[derive(Debug, Clone)]
pub struct SensoryRateNegotiationConfig {
    pub requested_sensory_rate_hz: f64,
    pub feagi_api_host: String,
    pub feagi_api_port: u16,
    pub api_timeout: Duration,
    pub policy: SensoryRateNegotiationPolicy,
}

/// Tokio driver policy (provided by caller/config).
#[derive(Debug, Clone)]
pub struct TokioDriverConfig {
    /// Poll cadence used by async loops in this module.
    pub poll_interval: Duration,
    /// Timing policy forwarded into the runtime-agnostic state machine.
    pub timing: SessionTimingConfig,
    /// Optional sensory-rate negotiation policy applied after registration.
    pub sensory_rate_negotiation: Option<SensoryRateNegotiationConfig>,
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
    effective_sensory_rate_hz: Option<f64>,
    min_sensory_send_interval: Option<Duration>,
    last_sensor_payload_sent_at: Option<Instant>,
    capped_sensory_frame_count: u64,
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
            effective_sensory_rate_hz: None,
            min_sensory_send_interval: None,
            last_sensor_payload_sent_at: None,
            capped_sensory_frame_count: 0,
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
                SessionPhase::Active => {
                    self.apply_sensory_rate_negotiation_blocking()?;
                    return Ok(());
                }
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
                SessionPhase::Active => {
                    agent.apply_sensory_rate_negotiation_async().await?;
                    return Ok(agent);
                }
                SessionPhase::Failed => {
                    return Err(FeagiAgentError::ConnectionFailed(
                        agent
                            .sm
                            .last_error()
                            .unwrap_or("session failed")
                            .to_string(),
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

        let now = Instant::now();
        if let (Some(min_interval), Some(last_sent)) = (
            self.min_sensory_send_interval,
            self.last_sensor_payload_sent_at,
        ) {
            if now.saturating_duration_since(last_sent) < min_interval {
                self.capped_sensory_frame_count = self.capped_sensory_frame_count.saturating_add(1);
                if self.capped_sensory_frame_count <= 10
                    || self.capped_sensory_frame_count % 100 == 0
                {
                    tracing::warn!(
                        "[feagi-agent] Sensory send capped at {:.2}Hz (skipped_total={})",
                        self.effective_sensory_rate_hz.unwrap_or(0.0),
                        self.capped_sensory_frame_count
                    );
                }
                return Ok(());
            }
        }

        let mut sensors = self.embodiment.get_sensor_cache();
        sensors.encode_all_sensors_to_neurons(now)?;
        sensors.encode_neurons_to_bytes()?;
        let bytes = sensors.get_feagi_byte_container_mut();
        bytes.set_agent_identifier(session_id)?;
        pusher.publish_data(bytes.get_byte_ref())?;
        self.last_sensor_payload_sent_at = Some(now);
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
    pub fn request_deregistration(
        &mut self,
        reason: Option<String>,
    ) -> Result<(), FeagiAgentError> {
        self.request_deregistration_and_disconnect(
            reason,
            self.driver.timing.registration_deadline_ms,
        )
    }

    /// Request deregistration and wait (bounded) for terminal state, then force disconnect transports.
    ///
    /// `deadline_ms` must come from caller/config policy; no hardcoded timeout is used.
    pub fn request_deregistration_and_disconnect(
        &mut self,
        reason: Option<String>,
        deadline_ms: Option<u64>,
    ) -> Result<(), FeagiAgentError> {
        let actions = self.sm.start_deregister(reason);
        self.execute_actions(&actions)?;

        let start_ms = self.now_ms();
        loop {
            let pending = self.poll_and_step()?;
            self.execute_actions(&pending)?;

            match self.sm.phase() {
                SessionPhase::Idle => break,
                SessionPhase::Failed => {
                    let msg = self
                        .sm
                        .last_error()
                        .unwrap_or("deregistration failed")
                        .to_string();
                    self.force_disconnect_transports()?;
                    return Err(FeagiAgentError::ConnectionFailed(msg));
                }
                _ => {}
            }

            if let Some(limit_ms) = deadline_ms {
                if self.now_ms().saturating_sub(start_ms) >= limit_ms {
                    self.force_disconnect_transports()?;
                    return Err(FeagiAgentError::ConnectionFailed(
                        "deregistration deadline exceeded".to_string(),
                    ));
                }
            } else {
                // Deterministic single-step behavior without policy deadline.
                break;
            }

            std::thread::yield_now();
        }

        self.force_disconnect_transports()?;
        Ok(())
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

    fn force_disconnect_transports(&mut self) -> Result<(), FeagiAgentError> {
        let mut first_error: Option<FeagiAgentError> = None;

        if let Some(pusher) = self.sensor_pusher.as_mut() {
            if let Err(e) = pusher.request_disconnect() {
                if first_error.is_none() {
                    first_error = Some(FeagiAgentError::from(e));
                }
            }
        }
        if let Some(sub) = self.motor_subscriber.as_mut() {
            if let Err(e) = sub.request_disconnect() {
                if first_error.is_none() {
                    first_error = Some(FeagiAgentError::from(e));
                }
            }
        }
        if let Err(e) = self.control.request_disconnect() {
            if first_error.is_none() {
                first_error = Some(e);
            }
        }

        self.sensor_pusher = None;
        self.motor_subscriber = None;

        if let Some(err) = first_error {
            return Err(err);
        }
        Ok(())
    }

    fn health_check_url(config: &SensoryRateNegotiationConfig) -> String {
        format!(
            "http://{}:{}/v1/system/health_check",
            config.feagi_api_host, config.feagi_api_port
        )
    }

    fn simulation_timestep_url(config: &SensoryRateNegotiationConfig) -> String {
        format!(
            "http://{}:{}/v1/burst_engine/simulation_timestep",
            config.feagi_api_host, config.feagi_api_port
        )
    }

    fn parse_effective_rate_hz_from_health(
        payload: &serde_json::Value,
    ) -> Result<f64, FeagiAgentError> {
        let timestep = payload
            .get("simulation_timestep")
            .and_then(serde_json::Value::as_f64)
            .ok_or_else(|| {
                FeagiAgentError::ConnectionFailed(
                    "FEAGI health_check missing simulation_timestep".to_string(),
                )
            })?;
        if !timestep.is_finite() || timestep <= 0.0 {
            return Err(FeagiAgentError::ConnectionFailed(format!(
                "FEAGI health_check returned invalid simulation_timestep={}",
                timestep
            )));
        }
        Ok(1.0 / timestep)
    }

    fn apply_effective_sensory_rate_hz(&mut self, effective_rate_hz: f64) {
        self.effective_sensory_rate_hz = Some(effective_rate_hz);
        self.min_sensory_send_interval = Some(Duration::from_secs_f64(1.0 / effective_rate_hz));
        self.last_sensor_payload_sent_at = None;
        self.capped_sensory_frame_count = 0;
    }

    fn apply_negotiated_rate_policy(
        &mut self,
        config: &SensoryRateNegotiationConfig,
        effective_rate_hz: f64,
    ) -> Result<(), FeagiAgentError> {
        const RATE_EPSILON_HZ: f64 = 0.01;
        if effective_rate_hz + RATE_EPSILON_HZ >= config.requested_sensory_rate_hz {
            self.apply_effective_sensory_rate_hz(config.requested_sensory_rate_hz);
            return Ok(());
        }
        match config.policy {
            SensoryRateNegotiationPolicy::Strict => Err(FeagiAgentError::ConnectionFailed(
                format!(
                    "FEAGI denied requested sensory rate increase. requested={:.2}Hz effective={:.2}Hz",
                    config.requested_sensory_rate_hz, effective_rate_hz
                ),
            )),
            SensoryRateNegotiationPolicy::CapAndWarn => {
                tracing::warn!(
                    "[feagi-agent] Requested sensory rate {:.2}Hz capped to FEAGI {:.2}Hz",
                    config.requested_sensory_rate_hz,
                    effective_rate_hz
                );
                self.apply_effective_sensory_rate_hz(effective_rate_hz);
                Ok(())
            }
        }
    }

    async fn apply_sensory_rate_negotiation_async(&mut self) -> Result<(), FeagiAgentError> {
        let Some(config) = self.driver.sensory_rate_negotiation.clone() else {
            return Ok(());
        };
        if !config.requested_sensory_rate_hz.is_finite() || config.requested_sensory_rate_hz <= 0.0
        {
            return Err(FeagiAgentError::ConnectionFailed(format!(
                "Invalid requested_sensory_rate_hz={}",
                config.requested_sensory_rate_hz
            )));
        }

        let client = reqwest::Client::builder()
            .timeout(config.api_timeout)
            .build()
            .map_err(|e| {
                FeagiAgentError::ConnectionFailed(format!("HTTP client init failed: {e}"))
            })?;
        let health_url = Self::health_check_url(&config);
        let health_response = client.get(&health_url).send().await.map_err(|e| {
            FeagiAgentError::ConnectionFailed(format!("health_check request failed: {e}"))
        })?;
        let health_json = health_response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| {
                FeagiAgentError::ConnectionFailed(format!("health_check parse failed: {e}"))
            })?;
        let current_rate_hz = Self::parse_effective_rate_hz_from_health(&health_json)?;

        const RATE_EPSILON_HZ: f64 = 0.01;
        if current_rate_hz + RATE_EPSILON_HZ >= config.requested_sensory_rate_hz {
            return self.apply_negotiated_rate_policy(&config, current_rate_hz);
        }

        let update_url = Self::simulation_timestep_url(&config);
        let timestep = 1.0 / config.requested_sensory_rate_hz;
        let update_result = client
            .post(&update_url)
            .json(&serde_json::json!({ "simulation_timestep": timestep }))
            .send()
            .await;
        if let Err(err) = update_result {
            return self.apply_negotiated_rate_policy(&config, current_rate_hz).and_then(|_| {
                tracing::warn!(
                    "[feagi-agent] FEAGI rate update request failed, keeping effective rate {:.2}Hz: {}",
                    current_rate_hz,
                    err
                );
                Ok(())
            });
        }

        let health_after = client.get(&health_url).send().await.map_err(|e| {
            FeagiAgentError::ConnectionFailed(format!("post-update health_check failed: {e}"))
        })?;
        let health_after_json = health_after
            .json::<serde_json::Value>()
            .await
            .map_err(|e| {
                FeagiAgentError::ConnectionFailed(format!(
                    "post-update health_check parse failed: {e}"
                ))
            })?;
        let updated_rate_hz = Self::parse_effective_rate_hz_from_health(&health_after_json)?;
        self.apply_negotiated_rate_policy(&config, updated_rate_hz)
    }

    fn apply_sensory_rate_negotiation_blocking(&mut self) -> Result<(), FeagiAgentError> {
        let Some(config) = self.driver.sensory_rate_negotiation.clone() else {
            return Ok(());
        };
        if !config.requested_sensory_rate_hz.is_finite() || config.requested_sensory_rate_hz <= 0.0
        {
            return Err(FeagiAgentError::ConnectionFailed(format!(
                "Invalid requested_sensory_rate_hz={}",
                config.requested_sensory_rate_hz
            )));
        }

        // reqwest::blocking internally owns a runtime for DNS/connectivity helpers.
        // Dropping that runtime inside a Tokio async worker thread can panic.
        // If we are inside Tokio, run blocking HTTP on a plain thread.
        let effective_rate_hz = if tokio::runtime::Handle::try_current().is_ok() {
            let cfg = config.clone();
            std::thread::spawn(move || Self::negotiate_effective_rate_blocking_http(&cfg))
                .join()
                .map_err(|_| {
                    FeagiAgentError::ConnectionFailed(
                        "sensory rate negotiation thread panicked".to_string(),
                    )
                })??
        } else {
            Self::negotiate_effective_rate_blocking_http(&config)?
        };

        self.apply_negotiated_rate_policy(&config, effective_rate_hz)
    }

    fn negotiate_effective_rate_blocking_http(
        config: &SensoryRateNegotiationConfig,
    ) -> Result<f64, FeagiAgentError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(config.api_timeout)
            .build()
            .map_err(|e| {
                FeagiAgentError::ConnectionFailed(format!("HTTP client init failed: {e}"))
            })?;
        let health_url = Self::health_check_url(&config);
        let health_response = client.get(&health_url).send().map_err(|e| {
            FeagiAgentError::ConnectionFailed(format!("health_check request failed: {e}"))
        })?;
        let health_json = health_response.json::<serde_json::Value>().map_err(|e| {
            FeagiAgentError::ConnectionFailed(format!("health_check parse failed: {e}"))
        })?;
        let current_rate_hz = Self::parse_effective_rate_hz_from_health(&health_json)?;

        const RATE_EPSILON_HZ: f64 = 0.01;
        if current_rate_hz + RATE_EPSILON_HZ >= config.requested_sensory_rate_hz {
            return Ok(current_rate_hz);
        }

        let update_url = Self::simulation_timestep_url(&config);
        let timestep = 1.0 / config.requested_sensory_rate_hz;
        let update_result = client
            .post(&update_url)
            .json(&serde_json::json!({ "simulation_timestep": timestep }))
            .send();
        if let Err(err) = update_result {
            tracing::warn!(
                "[feagi-agent] FEAGI rate update request failed, keeping effective rate {:.2}Hz: {}",
                current_rate_hz,
                err
            );
            return Ok(current_rate_hz);
        }

        let health_after = client.get(&health_url).send().map_err(|e| {
            FeagiAgentError::ConnectionFailed(format!("post-update health_check failed: {e}"))
        })?;
        let health_after_json = health_after.json::<serde_json::Value>().map_err(|e| {
            FeagiAgentError::ConnectionFailed(format!("post-update health_check parse failed: {e}"))
        })?;
        let updated_rate_hz = Self::parse_effective_rate_hz_from_health(&health_after_json)?;
        Ok(updated_rate_hz)
    }
}
