extern crate core;
pub mod clients;
pub mod command_and_control;
mod common;
mod feagi_agent_error;
pub mod server;

pub use feagi_agent_error::FeagiAgentError;

pub use common::{
    generate_agent_id_base64, AgentCapabilities, AgentDescriptor, AuthToken, FeagiApiVersion,
};
