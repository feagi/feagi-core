// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Agent management service trait
//!
//! This service manages agent registration, heartbeats, and properties.
//! It interfaces with the Registration Manager in feagi-pns for actual
//! agent lifecycle management and coordination.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result type for agent service operations
pub type AgentResult<T> = Result<T, AgentError>;

/// Errors that can occur during agent operations
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Agent not found: {0}")]
    NotFound(String),
    
    #[error("Registration service unavailable: {0}")]
    ServiceUnavailable(String),
    
    #[error("Registration failed: {0}")]
    RegistrationFailed(String),
    
    #[error("Invalid agent data: {0}")]
    InvalidData(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Agent registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistration {
    pub agent_id: String,
    pub agent_type: String,
    pub agent_data_port: u16,
    pub agent_version: String,
    pub controller_version: String,
    pub agent_ip: Option<String>,
    pub capabilities: HashMap<String, serde_json::Value>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Agent registration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistrationResponse {
    pub status: String,
    pub message: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rates: Option<HashMap<String, HashMap<String, f64>>>,
}

/// Agent properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProperties {
    pub agent_type: String,
    pub agent_ip: String,
    pub agent_data_port: u16,
    pub agent_router_address: String,
    pub agent_version: String,
    pub controller_version: String,
    pub capabilities: HashMap<String, serde_json::Value>,
}

/// Heartbeat request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub agent_id: String,
}

/// Service for managing agents (registration, heartbeats, properties)
#[async_trait]
pub trait AgentService: Send + Sync {
    /// Register a new agent
    async fn register_agent(
        &self,
        registration: AgentRegistration,
    ) -> AgentResult<AgentRegistrationResponse>;
    
    /// Record a heartbeat for an agent
    async fn heartbeat(&self, request: HeartbeatRequest) -> AgentResult<()>;
    
    /// List all registered agents
    async fn list_agents(&self) -> AgentResult<Vec<String>>;
    
    /// Get properties for a specific agent
    async fn get_agent_properties(&self, agent_id: &str) -> AgentResult<AgentProperties>;
    
    /// Get shared memory information for all agents
    async fn get_shared_memory_info(&self) -> AgentResult<HashMap<String, HashMap<String, serde_json::Value>>>;
    
    /// Deregister an agent
    async fn deregister_agent(&self, agent_id: &str) -> AgentResult<()>;
    
    /// Trigger manual stimulation for specific cortical areas
    async fn manual_stimulation(
        &self,
        stimulation_payload: HashMap<String, Vec<Vec<i32>>>,
    ) -> AgentResult<HashMap<String, serde_json::Value>>;
}



