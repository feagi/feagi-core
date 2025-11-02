// Registration Handler - processes agent registration requests

use super::agent_registry::{
    AgentCapabilities, AgentInfo, AgentRegistry, AgentTransport, AgentType, MotorCapability,
    SensoryCapability, VisualizationCapability,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn, error};
use ahash::AHashSet;

/// Registration request from agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationRequest {
    pub agent_id: String,
    pub agent_type: String,
    pub capabilities: serde_json::Value, // Flexible JSON for different formats
}

/// Registration response to agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResponse {
    pub status: String,
    pub message: Option<String>,
    pub shm_paths: Option<HashMap<String, String>>, // capability_type -> shm_path
    pub zmq_ports: Option<HashMap<String, u16>>,
}

/// Type alias for registration callbacks
pub type RegistrationCallback =
    Arc<parking_lot::Mutex<Option<Box<dyn Fn(String, String, String) + Send + Sync>>>>;
pub type DeregistrationCallback =
    Arc<parking_lot::Mutex<Option<Box<dyn Fn(String) + Send + Sync>>>>;

/// Registration Handler
pub struct RegistrationHandler {
    agent_registry: Arc<RwLock<AgentRegistry>>,
    shm_base_path: String,
    /// Optional reference to burst engine's sensory agent manager for SHM I/O
    sensory_agent_manager:
        Arc<parking_lot::Mutex<Option<Arc<std::sync::Mutex<feagi_burst_engine::AgentManager>>>>>,
    /// Optional reference to burst loop runner for motor subscription tracking
    burst_runner:
        Arc<parking_lot::Mutex<Option<Arc<parking_lot::RwLock<feagi_burst_engine::BurstLoopRunner>>>>>,
    /// Actual ZMQ port numbers (from config, NOT hardcoded)
    motor_port: u16,
    viz_port: u16,
    /// Callbacks for Python integration
    on_agent_registered: RegistrationCallback,
    on_agent_deregistered: DeregistrationCallback,
}

impl RegistrationHandler {
    pub fn new(agent_registry: Arc<RwLock<AgentRegistry>>, motor_port: u16, viz_port: u16) -> Self {
        Self {
            agent_registry,
            shm_base_path: "/tmp".to_string(),
            sensory_agent_manager: Arc::new(parking_lot::Mutex::new(None)),
            burst_runner: Arc::new(parking_lot::Mutex::new(None)),
            motor_port,
            viz_port,
            on_agent_registered: Arc::new(parking_lot::Mutex::new(None)),
            on_agent_deregistered: Arc::new(parking_lot::Mutex::new(None)),
        }
    }
    
    /// Set burst runner reference (for motor subscription tracking)
    pub fn set_burst_runner(
        &self,
        runner: Arc<parking_lot::RwLock<feagi_burst_engine::BurstLoopRunner>>,
    ) {
        *self.burst_runner.lock() = Some(runner);
        info!("🦀 [REGISTRATION] Burst runner connected for motor subscriptions");
    }

    /// Set the sensory agent manager (for SHM I/O coordination)
    pub fn set_sensory_agent_manager(
        &self,
        manager: Arc<std::sync::Mutex<feagi_burst_engine::AgentManager>>,
    ) {
        *self.sensory_agent_manager.lock() = Some(manager);
        info!("🦀 [REGISTRATION] Sensory agent manager connected");
    }

    /// Set callback for agent registration events (for Python integration)
    pub fn set_on_agent_registered<F>(&self, callback: F)
    where
        F: Fn(String, String, String) + Send + Sync + 'static,
    {
        *self.on_agent_registered.lock() = Some(Box::new(callback));
        info!("🦀 [REGISTRATION] Agent registration callback set");
    }

    /// Set callback for agent deregistration events (for Python integration)
    pub fn set_on_agent_deregistered<F>(&self, callback: F)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        *self.on_agent_deregistered.lock() = Some(Box::new(callback));
        info!("🦀 [REGISTRATION] Agent deregistration callback set");
    }

    /// Process a registration request
    pub fn process_registration(
        &self,
        request: RegistrationRequest,
    ) -> Result<RegistrationResponse, String> {
        let total_start = std::time::Instant::now();
        info!(
            "🦀 [REGISTRATION] 🔍 [LOCK-TRACE] Processing registration for agent: {} (type: {})",
            request.agent_id, request.agent_type
        );

        // Parse capabilities
        let capabilities = self.parse_capabilities(&request.capabilities)?;

        // Allocate SHM paths if needed
        let mut shm_paths = HashMap::new();
        let mut allocated_capabilities = capabilities.clone();

        if let Some(ref mut sensory) = allocated_capabilities.sensory {
            let shm_path = format!(
                "{}/feagi-shm-{}-sensory.bin",
                self.shm_base_path, request.agent_id
            );
            sensory.shm_path = Some(shm_path.clone());
            shm_paths.insert("sensory".to_string(), shm_path);
        }

        if allocated_capabilities.motor.is_some() {
            let shm_path = format!(
                "{}/feagi-shm-{}-motor.bin",
                self.shm_base_path, request.agent_id
            );
            shm_paths.insert("motor".to_string(), shm_path);
        }

        if allocated_capabilities.visualization.is_some() {
            let shm_path = format!(
                "{}/feagi-shared-mem-visualization_stream.bin",
                self.shm_base_path
            );
            shm_paths.insert("visualization".to_string(), shm_path);
        }

        // Determine transport
        let transport = if !shm_paths.is_empty() {
            AgentTransport::Hybrid
        } else {
            AgentTransport::Zmq
        };

        // Parse agent type string to enum
        let agent_type_enum = match request.agent_type.to_lowercase().as_str() {
            "sensory" => AgentType::Sensory,
            "motor" => AgentType::Motor,
            "both" => AgentType::Both,
            "visualization" => AgentType::Visualization,
            "infrastructure" => AgentType::Infrastructure,
            _ => return Err(format!("Invalid agent type: {}", request.agent_type)),
        };

        // Create agent info using the new constructor
        let agent_info = AgentInfo::new(
            request.agent_id.clone(),
            agent_type_enum,
            allocated_capabilities,
            transport,
        );

        // Register in registry
        info!("🦀 [REGISTRATION] 🔍 Registering agent '{}' in AgentRegistry...", request.agent_id);
        self.agent_registry
            .write()
            .register(agent_info.clone())
            .map_err(|e| format!("Failed to register agent: {}", e))?;
        
        // Verify registration
        let registry_count = self.agent_registry.read().get_all().len();
        let all_agents: Vec<String> = self.agent_registry.read().get_all().iter().map(|a| a.agent_id.clone()).collect();
        info!("🦀 [REGISTRATION] ✅ Agent '{}' registered in AgentRegistry (total agents: {})", request.agent_id, registry_count);
        info!("🦀 [REGISTRATION] Registry contents: {:?}", all_agents);
        info!("🦀 [REGISTRATION] Registry pointer: {:p}", &*self.agent_registry as *const _);

        // Register with burst engine's sensory agent manager (if sensory capability exists)
        if let Some(ref sensory) = agent_info.capabilities.sensory {
            if let Some(sensory_mgr_lock) = self.sensory_agent_manager.lock().as_ref() {
                if let Some(shm_path) = &sensory.shm_path {
                    info!(
                        "🦀 [REGISTRATION] 🔍 [LOCK-TRACE] Registering {} with burst engine: {} @ {}Hz",
                        request.agent_id, shm_path, sensory.rate_hz
                    );

                    let burst_start = std::time::Instant::now();
                    let sensory_mgr = sensory_mgr_lock.lock().unwrap();
                    let burst_lock_duration = burst_start.elapsed();
                    info!(
                        "🦀 [REGISTRATION] 🔍 [LOCK-TRACE] Burst engine lock acquired in {:?}",
                        burst_lock_duration
                    );
                    
                    let config = feagi_burst_engine::AgentConfig {
                        agent_id: request.agent_id.clone(),
                        shm_path: std::path::PathBuf::from(shm_path),
                        rate_hz: sensory.rate_hz,
                        area_mapping: sensory.cortical_mappings.clone(),
                    };
                    
                    let register_start = std::time::Instant::now();
                    sensory_mgr
                        .register_agent(config)
                        .map_err(|e| format!("Failed to register with burst engine: {}", e))?;
                    let register_duration = register_start.elapsed();

                    info!(
                        "🦀 [REGISTRATION] 🔍 [LOCK-TRACE] ✅ Agent {} registered with burst engine in {:?}",
                        request.agent_id, register_duration
                    );
                } else {
                    warn!("🦀 [REGISTRATION] ⚠️  Sensory capability exists but no SHM path");
                }
            } else {
                warn!("🦀 [REGISTRATION] ⚠️  Sensory agent manager not connected - skipping burst engine registration");
            }
        }

        // Register motor subscriptions with burst engine (if motor capability exists)
        if let Some(ref motor) = agent_info.capabilities.motor {
            if let Some(burst_runner_lock) = self.burst_runner.lock().as_ref() {
                info!(
                    "🦀 [REGISTRATION] 🎮 Agent {} has motor capability with {} cortical areas: {:?}",
                    request.agent_id, motor.source_cortical_areas.len(), motor.source_cortical_areas
                );
                
                // Store cortical_id strings (matching sensory stream pattern)
                // Resolution to area_idx happens during encoding, just like sensory
                let cortical_ids: AHashSet<String> = motor.source_cortical_areas.iter().cloned().collect();
                
                burst_runner_lock.read().register_motor_subscriptions(
                    request.agent_id.clone(),
                    cortical_ids.clone(),
                );
                
                info!("🦀 [REGISTRATION] ✅ Motor subscriptions registered for '{}' with cortical_ids: {:?}", 
                      request.agent_id, cortical_ids);
            } else {
                info!("🦀 [REGISTRATION] 🎮 Agent {} has motor capability but burst runner not connected yet", request.agent_id);
            }
        }

        // Invoke Python callback if set
        if let Some(ref callback) = *self.on_agent_registered.lock() {
            // Serialize capabilities to JSON string for Python
            let caps_json =
                serde_json::to_string(&request.capabilities).unwrap_or_else(|_| "{}".to_string());

            info!(
                "🦀 [REGISTRATION] 🔍 [LOCK-TRACE] Invoking Python callback for agent: {}",
                request.agent_id
            );
            let callback_start = std::time::Instant::now();
            callback(
                request.agent_id.clone(),
                request.agent_type.clone(),
                caps_json,
            );
            let callback_duration = callback_start.elapsed();
            info!(
                "🦀 [REGISTRATION] 🔍 [LOCK-TRACE] Python callback completed in {:?}",
                callback_duration
            );
        }

        // Return success response
        let total_duration = total_start.elapsed();
        info!(
            "🦀 [REGISTRATION] 🔍 [LOCK-TRACE] ✅ Total registration completed in {:?} for agent: {}",
            total_duration, request.agent_id
        );
        
        Ok(RegistrationResponse {
            status: "success".to_string(),
            message: Some(format!(
                "Agent {} registered successfully",
                request.agent_id
            )),
            shm_paths: if shm_paths.is_empty() {
                None
            } else {
                Some(shm_paths)
            },
            zmq_ports: Some(HashMap::from([
                ("motor".to_string(), self.motor_port),
                ("visualization".to_string(), self.viz_port),
            ])),
        })
    }

    /// Parse capabilities from JSON
    fn parse_capabilities(
        &self,
        caps_json: &serde_json::Value,
    ) -> Result<AgentCapabilities, String> {
        // Try to deserialize directly from JSON first (handles new agent SDK format)
        if let Ok(capabilities) = serde_json::from_value::<AgentCapabilities>(caps_json.clone()) {
            return Ok(capabilities);
        }

        // Fall back to manual parsing for legacy format
        let mut capabilities = AgentCapabilities::default();

        // Parse legacy sensory capability
        if let Some(sensory) = caps_json.get("sensory") {
            if let Some(rate_hz) = sensory.get("rate_hz").and_then(|v| v.as_f64()) {
                capabilities.sensory = Some(SensoryCapability {
                    rate_hz,
                    shm_path: None,
                    cortical_mappings: HashMap::new(),
                });
            }
        }

        // Parse motor capability (support both legacy and new format)
        if let Some(motor) = caps_json.get("motor") {
            if motor
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                capabilities.motor = Some(MotorCapability {
                    modality: motor
                        .get("modality")
                        .and_then(|v| v.as_str())
                        .unwrap_or("generic")
                        .to_string(),
                    output_count: motor
                        .get("output_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(1) as usize,
                    source_cortical_areas: vec![],
                });
            }
        }

        // Parse visualization capability
        if let Some(viz) = caps_json.get("visualization") {
            if viz
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                capabilities.visualization = Some(VisualizationCapability {
                    visualization_type: viz
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("generic")
                        .to_string(),
                    resolution: None,
                    refresh_rate: viz.get("rate_hz").and_then(|v| v.as_f64()),
                    bridge_proxy: viz
                        .get("bridge_proxy")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                });
            }
        }

        Ok(capabilities)
    }

    /// Process deregistration request
    pub fn process_deregistration(&self, agent_id: &str) -> Result<String, String> {
        // Deregister from burst engine first
        if let Some(sensory_mgr_lock) = self.sensory_agent_manager.lock().as_ref() {
            let sensory_mgr = sensory_mgr_lock.lock().unwrap();
            if let Err(e) = sensory_mgr.deregister_agent(agent_id) {
                error!(
                    "🦀 [REGISTRATION] ⚠️  Failed to deregister {} from burst engine: {}",
                    agent_id, e
                );
            } else {
                info!(
                    "🦀 [REGISTRATION] ✅ Agent {} deregistered from burst engine",
                    agent_id
                );
            }
        }

        // Deregister from registry
        let result = self
            .agent_registry
            .write()
            .deregister(agent_id)
            .map(|_| format!("Agent {} deregistered", agent_id));

        // Invoke Python callback if deregistration was successful
        if result.is_ok() {
            if let Some(ref callback) = *self.on_agent_deregistered.lock() {
                info!(
                    "🦀 [REGISTRATION] Invoking Python deregistration callback for agent: {}",
                    agent_id
                );
                callback(agent_id.to_string());
            }
        }

        result
    }

    /// Process heartbeat
    pub fn process_heartbeat(&self, agent_id: &str) -> Result<String, String> {
        self.agent_registry
            .write()
            .heartbeat(agent_id)
            .map(|_| format!("Heartbeat recorded for {}", agent_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registration_handler() {
        let registry = Arc::new(RwLock::new(AgentRegistry::with_defaults()));
        let handler = RegistrationHandler::new(registry.clone());

        let request = RegistrationRequest {
            agent_id: "test-agent".to_string(),
            agent_type: "both".to_string(),
            capabilities: serde_json::json!({
                "sensory": {"rate_hz": 30.0},
                "motor": {"enabled": true, "rate_hz": 20.0, "modality": "servo", "output_count": 2}
            }),
        };

        let response = handler.process_registration(request).unwrap();
        assert_eq!(response.status, "success");
        assert!(response.shm_paths.is_some());

        assert_eq!(registry.read().count(), 1);
    }
}
