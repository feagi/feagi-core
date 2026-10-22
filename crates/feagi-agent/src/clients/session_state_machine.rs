//! Runtime-agnostic session orchestration state machine.
//!
//! This module provides a pure, deterministic orchestration layer above the
//! poll-based `feagi-io` endpoint state machines.
//!
//! Design constraints:
//! - No sleeps, no threads, no blocking waits
//! - No hardcoded timeouts/retries/backoff
//! - Runtime-agnostic (Tokio/WASM/Embassy/RTOS drivers can all execute Actions)
//! - Transport-agnostic (ZMQ and WebSocket are supported via `TransportProtocolEndpoint`)
//!
//! @cursor:critical-path

use crate::command_and_control::agent_registration_message::{
    AgentRegistrationMessage, DeregistrationResponse, RegistrationResponse,
};
use crate::command_and_control::FeagiMessage;
use crate::{AgentCapabilities, AgentDescriptor, AuthToken};
use feagi_io::traits_and_enums::shared::{FeagiEndpointState, TransportProtocolEndpoint};
use feagi_io::AgentID;
use std::collections::HashMap;

/// Milliseconds in a monotonic clock domain provided by the driver.
pub type NowMs = u64;

#[derive(Debug, Clone)]
pub struct SessionTimingConfig {
    /// Heartbeat cadence for command/control channel, in milliseconds.
    pub heartbeat_interval_ms: u64,
    /// Optional registration deadline relative to `start_connect()` (in ms).
    pub registration_deadline_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct SessionInit {
    pub agent_descriptor: AgentDescriptor,
    pub auth_token: AuthToken,
    pub requested_capabilities: Vec<AgentCapabilities>,
    pub timing: SessionTimingConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionPhase {
    Idle,
    ControlConnecting,
    Registering,
    DataConnecting,
    Active,
    Deregistering,
    Failed,
}

#[derive(Debug, Clone)]
pub struct SessionStateMachine {
    init: SessionInit,
    phase: SessionPhase,
    connect_started_at_ms: Option<NowMs>,
    last_heartbeat_sent_at_ms: Option<NowMs>,
    session_id: Option<AgentID>,
    endpoints: Option<HashMap<AgentCapabilities, TransportProtocolEndpoint>>,
    last_error: Option<String>,
}

impl SessionStateMachine {
    pub fn new(init: SessionInit) -> Self {
        Self {
            init,
            phase: SessionPhase::Idle,
            connect_started_at_ms: None,
            last_heartbeat_sent_at_ms: None,
            session_id: None,
            endpoints: None,
            last_error: None,
        }
    }

    pub fn phase(&self) -> &SessionPhase {
        &self.phase
    }

    pub fn session_id(&self) -> Option<AgentID> {
        self.session_id
    }

    pub fn endpoints(&self) -> Option<&HashMap<AgentCapabilities, TransportProtocolEndpoint>> {
        self.endpoints.as_ref()
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    /// Begin connection orchestration. Returns the initial actions.
    pub fn start_connect(&mut self, now_ms: NowMs) -> Vec<SessionAction> {
        self.connect_started_at_ms = Some(now_ms);
        self.last_error = None;
        self.session_id = None;
        self.endpoints = None;
        self.last_heartbeat_sent_at_ms = None;
        self.phase = SessionPhase::ControlConnecting;
        vec![SessionAction::ControlRequestConnect]
    }

    /// Request a graceful deregistration. Returns actions to perform now.
    pub fn start_deregister(&mut self, reason: Option<String>) -> Vec<SessionAction> {
        match self.phase {
            SessionPhase::Active | SessionPhase::DataConnecting | SessionPhase::Registering => {
                self.phase = SessionPhase::Deregistering;
                vec![SessionAction::ControlSendDeregistration { reason }]
            }
            _ => Vec::new(),
        }
    }

    /// Advance the state machine by providing observed events.
    pub fn step(&mut self, now_ms: NowMs, events: &[SessionEvent]) -> Vec<SessionAction> {
        let mut actions: Vec<SessionAction> = Vec::new();

        // Optional registration deadline enforcement (policy from config, not hardcoded).
        if matches!(
            self.phase,
            SessionPhase::ControlConnecting | SessionPhase::Registering
        ) {
            if let (Some(start_ms), Some(deadline_ms)) = (
                self.connect_started_at_ms,
                self.init.timing.registration_deadline_ms,
            ) {
                if now_ms.saturating_sub(start_ms) > deadline_ms {
                    self.fail("registration deadline exceeded");
                    return actions;
                }
            }
        }

        for event in events {
            match event {
                SessionEvent::ControlObserved { state, message } => {
                    actions.extend(self.on_control_observed(now_ms, state, message.clone()));
                }
                SessionEvent::SensorObserved { state } => {
                    actions.extend(self.on_sensor_observed(state.clone()));
                }
                SessionEvent::MotorObserved { state } => {
                    actions.extend(self.on_motor_observed(state.clone()));
                }
                SessionEvent::Deregistered { response } => {
                    actions.extend(self.on_deregistered(response.clone()));
                }
            }
        }

        // Heartbeat scheduling: driver provides `now_ms`, we emit action only when due.
        if self.phase == SessionPhase::Active && self.heartbeat_due(now_ms) {
            actions.push(SessionAction::ControlSendHeartbeat);
            self.last_heartbeat_sent_at_ms = Some(now_ms);
        }

        actions
    }

    fn heartbeat_due(&self, now_ms: NowMs) -> bool {
        let interval = self.init.timing.heartbeat_interval_ms;
        if interval == 0 {
            // Policy: treat as disabled; caller should never set to 0.
            return false;
        }
        match self.last_heartbeat_sent_at_ms {
            None => true,
            Some(last) => now_ms.saturating_sub(last) >= interval,
        }
    }

    fn on_control_observed(
        &mut self,
        now_ms: NowMs,
        state: &FeagiEndpointState,
        message: Option<FeagiMessage>,
    ) -> Vec<SessionAction> {
        match self.phase {
            SessionPhase::ControlConnecting => match state {
                FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData => {
                    self.phase = SessionPhase::Registering;
                    vec![SessionAction::ControlSendRegistration {
                        agent_descriptor: self.init.agent_descriptor.clone(),
                        auth_token: self.init.auth_token.clone(),
                        requested_capabilities: self.init.requested_capabilities.clone(),
                    }]
                }
                FeagiEndpointState::Errored(e) => {
                    self.fail(&format!("control errored: {e}"));
                    Vec::new()
                }
                _ => Vec::new(),
            },
            SessionPhase::Registering => {
                if let Some(FeagiMessage::AgentRegistration(reg_msg)) = message {
                    match reg_msg {
                        AgentRegistrationMessage::ServerRespondsRegistration(resp) => {
                            return self.on_registration_response(now_ms, resp);
                        }
                        AgentRegistrationMessage::ServerRespondsDeregistration(resp) => {
                            return self.on_deregistered(resp);
                        }
                        _ => {}
                    }
                }
                if let FeagiEndpointState::Errored(e) = state {
                    self.fail(&format!("control errored: {e}"));
                }
                Vec::new()
            }
            SessionPhase::Active => {
                if let Some(FeagiMessage::HeartBeat) = message {
                    // Heartbeat ack; no state change required.
                }
                if let Some(FeagiMessage::AgentRegistration(
                    AgentRegistrationMessage::ServerRespondsDeregistration(resp),
                )) = message
                {
                    return self.on_deregistered(resp);
                }
                if let FeagiEndpointState::Errored(e) = state {
                    self.fail(&format!("control errored: {e}"));
                }
                Vec::new()
            }
            SessionPhase::Deregistering => {
                if let Some(FeagiMessage::AgentRegistration(
                    AgentRegistrationMessage::ServerRespondsDeregistration(resp),
                )) = message
                {
                    return self.on_deregistered(resp);
                }
                if let FeagiEndpointState::Errored(e) = state {
                    self.fail(&format!("control errored: {e}"));
                }
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    fn on_registration_response(
        &mut self,
        now_ms: NowMs,
        resp: RegistrationResponse,
    ) -> Vec<SessionAction> {
        match resp {
            RegistrationResponse::Success(session_id, endpoints) => {
                self.session_id = Some(session_id);
                self.endpoints = Some(endpoints.clone());
                self.phase = SessionPhase::DataConnecting;
                self.last_heartbeat_sent_at_ms = Some(now_ms);

                // Require sensory + motor endpoints.
                let sensory = endpoints.get(&AgentCapabilities::SendSensorData);
                let motor = endpoints.get(&AgentCapabilities::ReceiveMotorData);
                if sensory.is_none() || motor.is_none() {
                    self.fail("registration success missing required endpoints");
                    return Vec::new();
                }
                vec![
                    SessionAction::SensorConnectTo {
                        endpoint: sensory.unwrap().clone(),
                    },
                    SessionAction::MotorConnectTo {
                        endpoint: motor.unwrap().clone(),
                    },
                ]
            }
            RegistrationResponse::FailedInvalidAuth => {
                self.fail("registration failed: invalid auth");
                Vec::new()
            }
            RegistrationResponse::FailedInvalidRequest => {
                self.fail("registration failed: invalid request");
                Vec::new()
            }
            RegistrationResponse::AlreadyRegistered => {
                self.fail("registration failed: already registered");
                Vec::new()
            }
        }
    }

    fn on_sensor_observed(&mut self, state: FeagiEndpointState) -> Vec<SessionAction> {
        if self.phase != SessionPhase::DataConnecting {
            return Vec::new();
        }
        if let FeagiEndpointState::Errored(e) = state {
            self.fail(&format!("sensor errored: {e}"));
            return Vec::new();
        }
        // The driver will send both SensorObserved and MotorObserved; we transition to Active when both are active.
        Vec::new()
    }

    fn on_motor_observed(&mut self, state: FeagiEndpointState) -> Vec<SessionAction> {
        if self.phase != SessionPhase::DataConnecting {
            return Vec::new();
        }
        if let FeagiEndpointState::Errored(e) = state {
            self.fail(&format!("motor errored: {e}"));
            return Vec::new();
        }
        Vec::new()
    }

    fn on_deregistered(&mut self, response: DeregistrationResponse) -> Vec<SessionAction> {
        if self.phase != SessionPhase::Deregistering && self.phase != SessionPhase::Registering {
            // Server may respond NotRegistered at any time; accept it as terminal.
        }
        match response {
            DeregistrationResponse::Success | DeregistrationResponse::NotRegistered => {
                self.phase = SessionPhase::Idle;
                self.session_id = None;
                self.endpoints = None;
                self.last_heartbeat_sent_at_ms = None;
                self.connect_started_at_ms = None;
            }
        }
        Vec::new()
    }

    fn fail(&mut self, msg: &str) {
        self.phase = SessionPhase::Failed;
        self.last_error = Some(msg.to_string());
    }

    /// Helper for drivers: after polling sensor and motor channels, call this to decide if
    /// data channels are fully active and transition to Active.
    pub fn try_mark_data_channels_active(
        &mut self,
        sensor_state: &FeagiEndpointState,
        motor_state: &FeagiEndpointState,
    ) {
        if self.phase != SessionPhase::DataConnecting {
            return;
        }
        let sensor_ok = matches!(
            sensor_state,
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData
        );
        let motor_ok = matches!(
            motor_state,
            FeagiEndpointState::ActiveWaiting | FeagiEndpointState::ActiveHasData
        );
        if sensor_ok && motor_ok {
            self.phase = SessionPhase::Active;
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum SessionEvent {
    ControlObserved {
        state: FeagiEndpointState,
        message: Option<FeagiMessage>,
    },
    SensorObserved {
        state: FeagiEndpointState,
    },
    MotorObserved {
        state: FeagiEndpointState,
    },
    Deregistered {
        response: DeregistrationResponse,
    },
}

#[derive(Debug, Clone)]
pub enum SessionAction {
    ControlRequestConnect,
    ControlSendRegistration {
        agent_descriptor: AgentDescriptor,
        auth_token: AuthToken,
        requested_capabilities: Vec<AgentCapabilities>,
    },
    ControlSendHeartbeat,
    ControlSendDeregistration {
        reason: Option<String>,
    },
    SensorConnectTo {
        endpoint: TransportProtocolEndpoint,
    },
    MotorConnectTo {
        endpoint: TransportProtocolEndpoint,
    },
}
