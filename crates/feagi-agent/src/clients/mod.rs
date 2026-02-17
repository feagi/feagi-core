//! Client-side registration and data channels.
//!
//! Flow: connect to the registration endpoint (ZMQ or WebSocket) -> register -> disconnect
//! registration -> connect to returned data endpoints (sensory, motor, optional visualization).
//! Sensory data must be sent as FeagiByteContainer bytes with the session_id set.
//!
//! Use `ConnectorAgent::connect` for ZMQ or `ConnectorAgent::connect_ws` for WebSocket.

pub mod async_helpers;
mod blocking;
mod session_state_machine;

pub use blocking::command_control_agent::{AgentRegistrationStatus, CommandControlAgent};
pub use session_state_machine::{
    NowMs, SessionAction, SessionEvent, SessionInit, SessionPhase, SessionStateMachine,
    SessionTimingConfig,
};
