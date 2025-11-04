// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Agent service implementation
//!
//! This implementation delegates all agent management to the PNS AgentRegistry,
//! ensuring centralized coordination consistent with the Python implementation.

use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn, error};

use crate::traits::agent_service::*;
use feagi_bdu::ConnectomeManager;
use feagi_pns::{
    AgentRegistry, AgentInfo, AgentType, AgentCapabilities, AgentTransport,
    SensoryCapability, VisualizationCapability, MotorCapability,
};

/// Implementation of the Agent service
pub struct AgentServiceImpl {
    connectome_manager: Arc<RwLock<ConnectomeManager<f32>>>,
    agent_registry: Arc<RwLock<AgentRegistry>>,
}

impl AgentServiceImpl {
    pub fn new(
        connectome_manager: Arc<RwLock<ConnectomeManager<f32>>>,
        agent_registry: Arc<RwLock<AgentRegistry>>,
    ) -> Self {
        Self {
            connectome_manager,
            agent_registry,
        }
    }
}

#[async_trait]
impl AgentService for AgentServiceImpl {
    async fn register_agent(
        &self,
        registration: AgentRegistration,
    ) -> AgentResult<AgentRegistrationResponse> {
        info!("🦀 [AGENT-SERVICE] Registering agent: {} (type: {})",
              registration.agent_id, registration.agent_type);
        
        // Parse agent type
        let agent_type = match registration.agent_type.as_str() {
            "visualization" | "brain_visualizer" => AgentType::Visualization,
            "sensory" | "video_agent" | "camera_agent" => AgentType::Sensory,
            "motor" | "motor_agent" => AgentType::Motor,
            "both" | "sensorimotor" => AgentType::Both,
            "infrastructure" | "bridge" | "proxy" => AgentType::Infrastructure,
            other => {
                warn!("Unknown agent type '{}', defaulting to Sensory", other);
                AgentType::Sensory
            }
        };
        
        // Convert capabilities to PNS format
        let mut capabilities = AgentCapabilities::default();
        
        // Populate structured capabilities based on agent type to satisfy validation
        match agent_type {
            AgentType::Visualization => {
                // Brain Visualizer requires visualization capability
                capabilities.visualization = Some(VisualizationCapability {
                    visualization_type: "3d_brain".to_string(),
                    resolution: None,
                    refresh_rate: None,
                    bridge_proxy: false,
                });
            }
            AgentType::Sensory => {
                // Sensory agents require sensory capability
                capabilities.sensory = Some(SensoryCapability {
                    rate_hz: 30.0,
                    shm_path: None,
                    cortical_mappings: std::collections::HashMap::new(),
                });
            }
            AgentType::Motor => {
                // Motor agents require motor capability
                capabilities.motor = Some(MotorCapability {
                    modality: "generic".to_string(),
                    output_count: 0,
                    source_cortical_areas: vec![],
                });
            }
            AgentType::Both => {
                // Sensorimotor agents need both sensory and motor capabilities
                capabilities.sensory = Some(SensoryCapability {
                    rate_hz: 30.0,
                    shm_path: None,
                    cortical_mappings: std::collections::HashMap::new(),
                });
                capabilities.motor = Some(MotorCapability {
                    modality: "generic".to_string(),
                    output_count: 0,
                    source_cortical_areas: vec![],
                });
            }
            AgentType::Infrastructure => {
                // Infrastructure agents can proxy any type, use visualization as default
                capabilities.visualization = Some(VisualizationCapability {
                    visualization_type: "bridge".to_string(),
                    resolution: None,
                    refresh_rate: None,
                    bridge_proxy: true,
                });
            }
        }
        
        // Store all raw capabilities in custom field
        capabilities.custom = registration.capabilities.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        
        // Create agent info
        let mut agent_info = AgentInfo::new(
            registration.agent_id.clone(),
            agent_type,
            capabilities,
            AgentTransport::Zmq, // Default to ZMQ
        );
        
        // Store metadata
        if let Some(meta) = registration.metadata {
            agent_info.metadata.extend(meta.iter().map(|(k, v)| (k.clone(), v.clone())));
        }
        if let Some(ip) = registration.agent_ip {
            agent_info.metadata.insert("agent_ip".to_string(), serde_json::json!(ip));
        }
        agent_info.metadata.insert("agent_data_port".to_string(), serde_json::json!(registration.agent_data_port));
        agent_info.metadata.insert("agent_version".to_string(), serde_json::json!(registration.agent_version));
        agent_info.metadata.insert("controller_version".to_string(), serde_json::json!(registration.controller_version));
        
        // Register in agent registry
        info!("📝 [AGENT-SERVICE] Registering in AgentRegistry: {}", registration.agent_id);
        self.agent_registry.write().register(agent_info)
            .map_err(|e| {
                error!("❌ [AGENT-SERVICE] Registration failed: {}", e);
                AgentError::RegistrationFailed(e)
            })?;
        
        // Verify registration worked
        let count = self.agent_registry.read().get_all().len();
        info!("✅ [AGENT-SERVICE] Agent '{}' registered successfully (total agents: {})", 
            registration.agent_id, count);
        
        Ok(AgentRegistrationResponse {
            status: "success".to_string(),
            message: format!("Agent {} registered successfully", registration.agent_id),
            success: true,
            transport: None, // TODO: Add transport negotiation
            rates: None,     // TODO: Add rate negotiation
        })
    }
    
    async fn heartbeat(&self, request: HeartbeatRequest) -> AgentResult<()> {
        self.agent_registry.write().heartbeat(&request.agent_id)
            .map_err(|e| AgentError::NotFound(e))?;
        Ok(())
    }
    
    async fn list_agents(&self) -> AgentResult<Vec<String>> {
        tracing::info!(target: "feagi-services", "📋 list_agents() called - acquiring registry read lock...");
        let registry = self.agent_registry.read();
        let agents = registry.get_all();
        let agent_ids: Vec<String> = agents.iter().map(|a| a.agent_id.clone()).collect();
        
        tracing::info!(target: "feagi-services", 
            "📋 list_agents() found {} agents: {:?}", agent_ids.len(), agent_ids);
        tracing::info!(target: "feagi-services",
            "📋 Registry pointer: {:p}", &*self.agent_registry as *const _);
        
        Ok(agent_ids)
    }
    
    async fn get_agent_properties(&self, agent_id: &str) -> AgentResult<AgentProperties> {
        let registry = self.agent_registry.read();
        let agent = registry.get(agent_id)
            .ok_or_else(|| AgentError::NotFound(format!("Agent {} not found", agent_id)))?;
        
        // Extract properties from agent info
        let agent_ip = agent.metadata.get("agent_ip")
            .and_then(|v| v.as_str())
            .unwrap_or("127.0.0.1")
            .to_string();
        
        let agent_data_port = agent.metadata.get("agent_data_port")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u16;
        
        let agent_version = agent.metadata.get("agent_version")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        let controller_version = agent.metadata.get("controller_version")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        let agent_router_address = format!("tcp://{}:{}", agent_ip, agent_data_port);
        
        // Build full capabilities map (including vision, motor, viz, not just custom)
        let mut capabilities = HashMap::new();
        
        // Add vision capability if present
        if let Some(ref vision) = agent.capabilities.vision {
            capabilities.insert(
                "vision".to_string(),
                serde_json::to_value(vision).unwrap_or(serde_json::Value::Null)
            );
        }
        
        // Add motor capability if present
        if let Some(ref motor) = agent.capabilities.motor {
            capabilities.insert(
                "motor".to_string(),
                serde_json::to_value(motor).unwrap_or(serde_json::Value::Null)
            );
        }
        
        // Add visualization capability if present
        if let Some(ref viz) = agent.capabilities.visualization {
            capabilities.insert(
                "visualization".to_string(),
                serde_json::to_value(viz).unwrap_or(serde_json::Value::Null)
            );
        }
        
        // Add sensory capability if present
        if let Some(ref sensory) = agent.capabilities.sensory {
            capabilities.insert(
                "sensory".to_string(),
                serde_json::to_value(sensory).unwrap_or(serde_json::Value::Null)
            );
        }
        
        // Add custom capabilities
        for (k, v) in agent.capabilities.custom.iter() {
            capabilities.insert(k.clone(), v.clone());
        }
        
        Ok(AgentProperties {
            agent_type: agent.agent_type.to_string(),
            agent_ip,
            agent_data_port,
            agent_router_address,
            agent_version,
            controller_version,
            capabilities,
        })
    }
    
    async fn get_shared_memory_info(&self) -> AgentResult<HashMap<String, HashMap<String, serde_json::Value>>> {
        // TODO: Implement shared memory tracking in agent registry
        Ok(HashMap::new())
    }
    
    async fn deregister_agent(&self, agent_id: &str) -> AgentResult<()> {
        self.agent_registry.write().deregister(agent_id)
            .map_err(|e| AgentError::NotFound(e))?;
        
        info!("✅ [AGENT-SERVICE] Agent '{}' deregistered successfully", agent_id);
        Ok(())
    }
    
    async fn manual_stimulation(
        &self,
        stimulation_payload: HashMap<String, Vec<Vec<i32>>>,
    ) -> AgentResult<HashMap<String, serde_json::Value>> {
        // This delegates to ConnectomeManager for actual neuron stimulation
        let manager = self.connectome_manager.read();
        
        let mut result = HashMap::new();
        let mut total_stimulated = 0;
        let mut successful_areas = 0;
        let mut failed_areas = Vec::new();
        
        for (cortical_id, coordinates) in stimulation_payload.iter() {
            match manager.get_cortical_area(cortical_id) {
                Some(_area) => {
                    // For each coordinate, we would trigger neuron activation
                    // This requires NPU integration for setting fire candidates
                    // For now, just count the coordinates
                    total_stimulated += coordinates.len();
                    successful_areas += 1;
                    
                    // TODO: Actual implementation would call:
                    // - manager.get_neuron_by_coordinates(cortical_id, x, y, z)
                    // - npu.add_to_fire_candidates(neuron_id)
                },
                None => {
                    failed_areas.push(cortical_id.clone());
                }
            }
        }
        
        result.insert("success".to_string(), serde_json::json!(failed_areas.is_empty()));
        result.insert("total_coordinates".to_string(), serde_json::json!(total_stimulated));
        result.insert("successful_areas".to_string(), serde_json::json!(successful_areas));
        result.insert("failed_areas".to_string(), serde_json::json!(failed_areas));
        
        if !failed_areas.is_empty() {
            result.insert("error".to_string(), serde_json::json!(format!(
                "Some cortical areas not found: {:?}",
                failed_areas
            )));
        }
        
        Ok(result)
    }
}



