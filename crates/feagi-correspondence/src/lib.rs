pub mod legacy_code;

pub use legacy_code::feagi_agent_error::{is_transient_zmq_send_message, is_transient_zmq_send_would_block, FeagiAgentError};

pub use legacy_code::common::{AgentCapabilities, AgentDescriptor, AuthToken, FeagiApiVersion};

pub extern crate feagi_correspondence_io;