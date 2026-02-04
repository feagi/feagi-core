use feagi_serialization::SessionID;
use crate::registration::{AgentCapabilities, AgentDescriptor};





#[derive(Debug, Clone)]
pub struct AgentRegisteredEvent {
    agent_descriptor: AgentDescriptor,
    requested_capabilities: Vec<AgentCapabilities>,
}