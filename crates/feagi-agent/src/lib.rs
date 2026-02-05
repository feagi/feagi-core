mod feagi_agent_client_error;
//#[cfg(feature = "sdk")]
//pub mod core;
//#[cfg(feature = "sdk")]
//pub use core::{AgentClient, AgentConfig, AgentType};
pub mod server;
pub mod clients;
pub mod registration;
//pub mod sdk;
mod feagi_agent_server_error;

pub use feagi_agent_client_error::FeagiAgentClientError;

///// Alias for SDK-facing errors (e.g. feagi-desktop).
//pub use FeagiAgentClientError as SdkError;