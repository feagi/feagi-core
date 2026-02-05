//! Client-side registration and data channels.
//!
//! Flow: connect to the registration endpoint (ZMQ or WebSocket) -> register -> disconnect
//! registration -> connect to returned data endpoints (sensory, motor, optional visualization).
//! Sensory data must be sent as FeagiByteContainer bytes with the session_id set.
//!
//! Use `ConnectorAgent::connect` for ZMQ or `ConnectorAgent::connect_ws` for WebSocket.

mod blocking;

pub use blocking::embodiment_agent::{DeviceRegistrations, EmbodimentAgent};
pub use blocking::registration_agent::RegistrationAgent;