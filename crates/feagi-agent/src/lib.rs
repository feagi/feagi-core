extern crate core;
pub mod clients;
pub mod command_and_control;
mod common;
mod feagi_agent_error;
pub mod server;

pub use feagi_agent_error::{
    is_transient_zmq_send_message, is_transient_zmq_send_would_block, FeagiAgentError,
};

pub use common::{AgentCapabilities, AgentDescriptor, AuthToken, FeagiApiVersion};
