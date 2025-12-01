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
use crate::traits::RuntimeService as RuntimeServiceTrait;
use feagi_data_structures::genomic::cortical_area::CorticalID;
use feagi_bdu::ConnectomeManager;
use feagi_pns::{
    AgentRegistry, AgentInfo, AgentType, AgentCapabilities, AgentTransport,
    SensoryCapability, VisualizationCapability, MotorCapability,
    RegistrationHandler, RegistrationRequest,
};
use parking_lot::Mutex;

/// Implementation of the Agent service
pub struct AgentServiceImpl {
    connectome_manager: Arc<RwLock<ConnectomeManager>>,
    agent_registry: Arc<RwLock<AgentRegistry>>,
    registration_handler: Option<Arc<Mutex<RegistrationHandler>>>,
    runtime_service: Arc<RwLock<Option<Arc<dyn RuntimeServiceTrait + Send + Sync>>>>,
}

impl AgentServiceImpl {
    pub fn new(
        connectome_manager: Arc<RwLock<ConnectomeManager>>,
        agent_registry: Arc<RwLock<AgentRegistry>>,
    ) -> Self {
        Self {
            connectome_manager,
            agent_registry,
            registration_handler: None,
            runtime_service: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Create AgentServiceImpl with runtime service
    pub fn new_with_runtime(
        connectome_manager: Arc<RwLock<ConnectomeManager>>,
        agent_registry: Arc<RwLock<AgentRegistry>>,
        runtime_service: Arc<dyn RuntimeServiceTrait + Send + Sync>,
    ) -> Self {
        Self {
            connectome_manager,
            agent_registry,
            registration_handler: None,
            runtime_service: Arc::new(RwLock::new(Some(runtime_service))),
        }
    }
    
    /// Set the PNS registration handler for full transport negotiation
    pub fn set_registration_handler(&mut self, handler: Arc<Mutex<RegistrationHandler>>) {
        self.registration_handler = Some(handler);
        info!("🦀 [AGENT-SERVICE] Registration handler connected");
    }
    
    /// Set the runtime service for sensory injection (thread-safe, can be called after Arc wrapping)
    pub fn set_runtime_service(&self, runtime_service: Arc<dyn RuntimeServiceTrait + Send + Sync>) {
        *self.runtime_service.write() = Some(runtime_service);
        info!("🦀 [AGENT-SERVICE] Runtime service connected");
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
        
        // If we have a registration handler, use it (gets full transport info)
        if let Some(handler) = &self.registration_handler {
            info!("📝 [AGENT-SERVICE] Using PNS registration handler for full transport negotiation");
            
            // Build PNS registration request
            let pns_request = RegistrationRequest {
                agent_id: registration.agent_id.clone(),
                agent_type: registration.agent_type.clone(),
                capabilities: serde_json::to_value(&registration.capabilities)
                    .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
                chosen_transport: registration.chosen_transport.clone(), // Pass through the agent's transport choice
            };
            
            // Call PNS registration handler
            let pns_response = handler.lock().process_registration(pns_request)
                .map_err(|e| AgentError::RegistrationFailed(e))?;
            
            // Convert PNS transport configs to service transport configs
            let transports = pns_response.transports.map(|ts| {
                ts.into_iter().map(|t| {
                    TransportConfig {
                        transport_type: t.transport_type,
                        enabled: t.enabled,
                        ports: t.ports,
                        host: t.host,
                    }
                }).collect()
            });
            
            return Ok(AgentRegistrationResponse {
                status: pns_response.status,
                message: pns_response.message.unwrap_or_else(|| "Success".to_string()),
                success: true,
                transport: None,  // Legacy
                rates: None,      // TODO: Calculate rates
                transports,       // NEW: Full transport info!
                recommended_transport: pns_response.recommended_transport,
                zmq_ports: pns_response.zmq_ports,
                shm_paths: pns_response.shm_paths,
            });
        }
        
        // Fallback: Register directly in registry (legacy path without transport info)
        warn!("⚠️ [AGENT-SERVICE] No registration handler - using fallback path (no transport info)");
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
            transport: None,
            rates: None,
            transports: None,      // No transport info in fallback path
            recommended_transport: None,
            zmq_ports: None,
            shm_paths: None,
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
            chosen_transport: agent.chosen_transport.clone(),
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
        // Use RuntimeService for sensory injection (service layer, not direct NPU access)
        let runtime_service = self.runtime_service.read()
            .as_ref()
            .ok_or_else(|| AgentError::Internal("Runtime service not available - cannot inject stimuli".to_string()))?
            .clone();
        
        let mut result = HashMap::new();
        let mut total_stimulated = 0;
        let mut successful_areas = 0;
        let mut failed_areas = Vec::new();
        let mut coordinates_not_found = 0;
        
        // Default potential for manual stimulation (high enough to trigger firing)
        const DEFAULT_POTENTIAL: f32 = 100.0;
        
        // First pass: validate all cortical areas and build injection data
        let mut injection_requests: Vec<(String, Vec<(u32, u32, u32, f32)>)> = Vec::new();
        
        {
            let manager = self.connectome_manager.read();
            
            for (cortical_id, coordinates) in stimulation_payload.iter() {
                let cortical_id_typed = match CorticalID::try_from_base_64(cortical_id) {
                    Ok(id) => id,
                    Err(e) => {
                        failed_areas.push(cortical_id.clone());
                        warn!("Invalid cortical ID '{}': {}", cortical_id, e);
                        continue;
                    }
                };
                
                match manager.get_cortical_area(&cortical_id_typed) {
                    Some(_area) => {
                        // Build xyzp_data for this cortical area (coordinates with potential)
                        let mut xyzp_data = Vec::new();
                        
                        for coord in coordinates {
                            if coord.len() != 3 {
                                warn!("Invalid coordinate format: {:?} (expected [x, y, z])", coord);
                                coordinates_not_found += 1;
                                continue;
                            }
                            
                            let x = coord[0] as u32;
                            let y = coord[1] as u32;
                            let z = coord[2] as u32;
                            
                            // Add coordinate with default potential
                            xyzp_data.push((x, y, z, DEFAULT_POTENTIAL));
                        }
                        
                        if !xyzp_data.is_empty() {
                            injection_requests.push((cortical_id.clone(), xyzp_data));
                            successful_areas += 1;
                        }
                    },
                    None => {
                        failed_areas.push(cortical_id.clone());
                    }
                }
            }
        } // Drop manager lock here before await
        
        // Second pass: perform injections (no locks held)
        for (cortical_id, xyzp_data) in injection_requests {
            match runtime_service.inject_sensory_by_coordinates(&cortical_id, &xyzp_data).await {
                Ok(injected_count) => {
                    total_stimulated += injected_count;
                    if injected_count < xyzp_data.len() {
                        coordinates_not_found += xyzp_data.len() - injected_count;
                    }
                },
                Err(e) => {
                    error!("❌ [MANUAL-STIMULATION] Failed to inject for area {}: {}", cortical_id, e);
                    coordinates_not_found += xyzp_data.len();
                }
            }
        }
        
        result.insert("success".to_string(), serde_json::json!(failed_areas.is_empty() && coordinates_not_found == 0));
        result.insert("total_coordinates".to_string(), serde_json::json!(total_stimulated));
        result.insert("successful_areas".to_string(), serde_json::json!(successful_areas));
        result.insert("failed_areas".to_string(), serde_json::json!(failed_areas));
        
        if coordinates_not_found > 0 {
            result.insert("coordinates_not_found".to_string(), serde_json::json!(coordinates_not_found));
        }
        
        if !failed_areas.is_empty() {
            result.insert("error".to_string(), serde_json::json!(format!(
                "Some cortical areas not found: {:?}",
                failed_areas
            )));
        }
        
        Ok(result)
    }
    
    fn try_set_runtime_service(&self, runtime_service: Arc<dyn RuntimeServiceTrait + Send + Sync>) {
        self.set_runtime_service(runtime_service);
    }
}



