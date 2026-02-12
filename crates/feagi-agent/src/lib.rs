extern crate core;
mod feagi_agent_error;
mod common;
pub mod command_and_control;
pub mod server;
pub mod clients;

pub use feagi_agent_error::FeagiAgentError;

pub use common::{
    generate_agent_id_base64, AgentCapabilities, AgentDescriptor, AuthToken, FeagiApiVersion,
};
