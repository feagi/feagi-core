// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Agent registry types - shared between feagi-services and feagi-io

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Type of agent based on I/O direction and purpose
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    /// Agent provides sensory input to FEAGI
    Sensory,
    /// Agent receives motor output from FEAGI
    Motor,
    /// Agent both sends and receives data
    Both,
    /// Agent consumes visualization stream only (e.g., Brain Visualizer clients)
    Visualization,
    /// Infrastructure agent (e.g., bridges, proxies) - needs viz + control streams
    Infrastructure,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::Sensory => write!(f, "sensory"),
            AgentType::Motor => write!(f, "motor"),
            AgentType::Both => write!(f, "both"),
            AgentType::Visualization => write!(f, "visualization"),
            AgentType::Infrastructure => write!(f, "infrastructure"),
        }
    }
}

impl std::str::FromStr for AgentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sensory" => Ok(AgentType::Sensory),
            "motor" => Ok(AgentType::Motor),
            "both" => Ok(AgentType::Both),
            "visualization" => Ok(AgentType::Visualization),
            "infrastructure" => Ok(AgentType::Infrastructure),
            _ => Err(format!("Invalid agent type: {}", s)),
        }
    }
}

/// Vision input capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionCapability {
    /// Type of vision sensor (camera, lidar, depth, etc.)
    pub modality: String,
    /// Frame dimensions [width, height]
    pub dimensions: (usize, usize),
    /// Number of channels (1=grayscale, 3=RGB, 4=RGBA)
    pub channels: usize,
    /// Target cortical area ID
    pub target_cortical_area: String,
    /// Semantic unit identifier (preferred).
    ///
    /// When set, the backend can derive cortical IDs without requiring agents to know
    /// internal 3-letter unit designators (e.g., "svi"/"seg").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<SensoryUnit>,
    /// Cortical unit index (group) for the selected unit (preferred).
    ///
    /// FEAGI encodes the group in the cortical ID. This keeps the wire contract
    /// language-agnostic and avoids leaking internal byte layouts to SDK users.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<u8>,
}

/// Motor output capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorCapability {
    /// Type of motor (servo, stepper, dc, etc.)
    pub modality: String,
    /// Number of motor outputs
    pub output_count: usize,
    /// Source cortical area IDs
    pub source_cortical_areas: Vec<String>,
    /// Semantic unit identifier (preferred).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<MotorUnit>,
    /// Cortical unit index (group) for the selected unit (preferred).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<u8>,
    /// Multiple semantic motor unit sources (preferred for multi-OPU agents).
    ///
    /// This supports agents that subscribe to multiple motor cortical unit types
    /// (e.g., object_segmentation + simple_vision_output + text_english_output).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_units: Option<Vec<MotorUnitSpec>>,
}

/// Motor unit + group pair for registration contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MotorUnitSpec {
    pub unit: MotorUnit,
    pub group: u8,
}

/// Language-agnostic sensory unit identifiers for registration contracts.
///
/// These are **not** the same as the internal 3-letter unit designators. They are intended
/// to be stable, human-readable, and suitable for auto-generated SDKs in other languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensoryUnit {
    Infrared,
    Proximity,
    Shock,
    Battery,
    Servo,
    AnalogGpio,
    DigitalGpio,
    MiscData,
    TextEnglishInput,
    CountInput,
    Vision,
    SegmentedVision,
    Accelerometer,
    Gyroscope,
}

/// Language-agnostic motor unit identifiers for registration contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MotorUnit {
    RotaryMotor,
    PositionalServo,
    Gaze,
    MiscData,
    TextEnglishOutput,
    CountOutput,
    ObjectSegmentation,
    SimpleVisionOutput,
}

/// Visualization capability for agents that consume neural activity stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationCapability {
    /// Type of visualization (3d_brain, 2d_plot, neural_graph, etc.)
    pub visualization_type: String,
    /// Display resolution if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<(usize, usize)>,
    /// Refresh rate in Hz if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_rate: Option<f64>,
    /// Whether this is a bridge/proxy agent (vs direct consumer)
    #[serde(default)]
    pub bridge_proxy: bool,
}

/// Sensory capability for non-vision sensory modalities (text, audio, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensoryCapability {
    pub rate_hz: f64,
    pub shm_path: Option<String>,
}

/// Agent capabilities describing what data it can provide/consume
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentCapabilities {
    /// Vision input capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vision: Option<VisionCapability>,

    /// Motor output capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motor: Option<MotorCapability>,

    /// Visualization capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visualization: Option<VisualizationCapability>,

    /// Legacy sensory capability (for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensory: Option<SensoryCapability>,

    /// Custom capabilities (extensible for audio, tactile, etc.)
    #[serde(flatten)]
    pub custom: serde_json::Map<String, serde_json::Value>,
}

/// Transport mechanism for agent communication
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentTransport {
    Zmq,
    Shm,
    Hybrid, // Uses both ZMQ and SHM
}

/// Complete agent information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Unique agent identifier
    pub agent_id: String,

    /// Agent type (sensory, motor, or both)
    pub agent_type: AgentType,

    /// Agent capabilities
    pub capabilities: AgentCapabilities,

    /// Transport method the agent chose to use
    pub chosen_transport: Option<String>, // "zmq", "websocket", "shm", or "hybrid"

    /// Registration timestamp (Unix epoch milliseconds)
    pub registered_at: u64,

    /// Last activity timestamp (Unix epoch milliseconds)
    pub last_seen: u64,

    /// Transport mechanism
    pub transport: AgentTransport,

    /// Metadata (client version, hostname, etc.)
    #[serde(default)]
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

impl AgentInfo {
    /// Create a new agent info with current timestamp
    pub fn new(
        agent_id: String,
        agent_type: AgentType,
        capabilities: AgentCapabilities,
        transport: AgentTransport,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            agent_id,
            agent_type,
            capabilities,
            chosen_transport: None, // Set later when agent reports back
            registered_at: now,
            last_seen: now,
            transport,
            metadata: serde_json::Map::new(),
        }
    }

    /// Update last_seen timestamp to current time
    pub fn update_activity(&mut self) {
        self.last_seen = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
    }

    /// Check if agent has been inactive for more than timeout_ms
    pub fn is_inactive(&self, timeout_ms: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        now - self.last_seen > timeout_ms
    }
}

/// Agent Registry - single source of truth for agent management
pub struct AgentRegistry {
    agents: std::collections::HashMap<String, AgentInfo>,
    max_agents: usize,
    timeout_ms: u64,
}

impl AgentRegistry {
    /// Create a new agent registry
    ///
    /// # Arguments
    /// * `max_agents` - Maximum number of concurrent agents (default: 100)
    /// * `timeout_ms` - Inactivity timeout in milliseconds (default: 60000)
    pub fn new(max_agents: usize, timeout_ms: u64) -> Self {
        tracing::info!(
            "🦀 [REGISTRY] Initialized (max_agents={}, timeout_ms={})",
            max_agents,
            timeout_ms
        );
        Self {
            agents: std::collections::HashMap::new(),
            max_agents,
            timeout_ms,
        }
    }

    /// Create registry with default settings (100 agents, 60s timeout)
    pub fn with_defaults() -> Self {
        Self::new(100, 60000)
    }

    /// Register a new agent
    pub fn register(&mut self, agent_info: AgentInfo) -> Result<(), String> {
        let agent_id = agent_info.agent_id.clone();

        // Validate configuration
        self.validate_agent_config(&agent_id, &agent_info.agent_type, &agent_info.capabilities)?;

        // Check if already registered (allow re-registration)
        let is_reregistration = self.agents.contains_key(&agent_id);
        if is_reregistration {
            tracing::warn!(
                "⚠️ [REGISTRY] Agent re-registering (updating existing entry): {}",
                agent_id
            );
        } else {
            // Check capacity only for new registrations
            if self.agents.len() >= self.max_agents {
                return Err(format!(
                    "Registry full ({}/{})",
                    self.agents.len(),
                    self.max_agents
                ));
            }
        }

        tracing::info!(
            "🦀 [REGISTRY] Registered agent: {} (type: {}, total: {})",
            agent_id,
            agent_info.agent_type,
            self.agents.len() + if is_reregistration { 0 } else { 1 }
        );
        self.agents.insert(agent_id, agent_info);
        self.refresh_agent_data_hash();
        Ok(())
    }

    /// Deregister an agent
    pub fn deregister(&mut self, agent_id: &str) -> Result<(), String> {
        if self.agents.remove(agent_id).is_some() {
            tracing::info!(
                "🦀 [REGISTRY] Deregistered agent: {} (total: {})",
                agent_id,
                self.agents.len()
            );
            self.refresh_agent_data_hash();
            Ok(())
        } else {
            Err(format!("Agent {} not found", agent_id))
        }
    }

    /// Update heartbeat for an agent
    pub fn heartbeat(&mut self, agent_id: &str) -> Result<(), String> {
        use tracing::debug;

        if let Some(agent) = self.agents.get_mut(agent_id) {
            let old_last_seen = agent.last_seen;
            agent.update_activity();
            let new_last_seen = agent.last_seen;

            debug!(
                "💓 [REGISTRY] Heartbeat updated for '{}': last_seen {} -> {} (+{}ms)",
                agent_id,
                old_last_seen,
                new_last_seen,
                new_last_seen.saturating_sub(old_last_seen)
            );

            Ok(())
        } else {
            Err(format!("Agent {} not found", agent_id))
        }
    }

    /// Get agent info
    pub fn get(&self, agent_id: &str) -> Option<&AgentInfo> {
        self.agents.get(agent_id)
    }

    /// Get all agents
    pub fn get_all(&self) -> Vec<&AgentInfo> {
        self.agents.values().collect()
    }

    /// Get agents with stale heartbeats (older than configured timeout)
    pub fn get_stale_agents(&self) -> Vec<String> {
        self.agents
            .iter()
            .filter(|(_, info)| info.is_inactive(self.timeout_ms))
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Prune inactive agents
    ///
    /// # Returns
    /// Number of agents pruned
    pub fn prune_inactive_agents(&mut self) -> usize {
        let inactive: Vec<String> = self
            .agents
            .iter()
            .filter(|(_, info)| info.is_inactive(self.timeout_ms))
            .map(|(id, _)| id.clone())
            .collect();

        let count = inactive.len();
        for agent_id in &inactive {
            self.agents.remove(agent_id);
            tracing::info!("🦀 [REGISTRY] Pruned inactive agent: {}", agent_id);
        }

        if count > 0 {
            tracing::info!(
                "🦀 [REGISTRY] Pruned {} inactive agents (total: {})",
                count,
                self.agents.len()
            );
            self.refresh_agent_data_hash();
        }

        count
    }

    /// Get number of registered agents
    pub fn count(&self) -> usize {
        self.agents.len()
    }

    fn refresh_agent_data_hash(&self) {
        #[cfg(feature = "std")]
        {
            use feagi_state_manager::StateManager;
            let mut agent_ids: Vec<&String> = self.agents.keys().collect();
            agent_ids.sort();
            let mut hasher = DefaultHasher::new();
            for agent_id in agent_ids {
                agent_id.hash(&mut hasher);
                if let Some(agent) = self.agents.get(agent_id) {
                    agent.agent_id.hash(&mut hasher);
                    agent.agent_type.hash(&mut hasher);
                    agent.transport.hash(&mut hasher);
                    hash_optional_string(&agent.chosen_transport, &mut hasher);
                    hash_json_value(&serde_json::Value::Object(agent.metadata.clone()), &mut hasher);
                    if let Ok(value) = serde_json::to_value(&agent.capabilities) {
                        hash_json_value(&value, &mut hasher);
                    }
                }
            }
            let hash_value = hasher.finish() & AGENT_HASH_SAFE_MASK;
            if let Some(state_manager) = StateManager::instance().try_write() {
                state_manager.set_agent_data_hash(hash_value);
            }
        }
    }

    /// Check if any agent has sensory capability (for stream gating)
    pub fn has_sensory_agents(&self) -> bool {
        self.agents.values().any(|agent| {
            agent.capabilities.sensory.is_some() || agent.capabilities.vision.is_some()
        })
    }

    /// Check if any agent has motor capability (for stream gating)
    pub fn has_motor_agents(&self) -> bool {
        self.agents
            .values()
            .any(|agent| agent.capabilities.motor.is_some())
    }

    /// Check if any agent has visualization capability (for stream gating)
    pub fn has_visualization_agents(&self) -> bool {
        self.agents
            .values()
            .any(|agent| agent.capabilities.visualization.is_some())
    }

    /// Get count of agents with sensory capability
    pub fn count_sensory_agents(&self) -> usize {
        self.agents
            .values()
            .filter(|agent| {
                agent.capabilities.sensory.is_some() || agent.capabilities.vision.is_some()
            })
            .count()
    }

    /// Get count of agents with motor capability
    pub fn count_motor_agents(&self) -> usize {
        self.agents
            .values()
            .filter(|agent| agent.capabilities.motor.is_some())
            .count()
    }

    /// Get count of agents with visualization capability
    pub fn count_visualization_agents(&self) -> usize {
        self.agents
            .values()
            .filter(|agent| agent.capabilities.visualization.is_some())
            .count()
    }

    /// Get the configured timeout threshold in milliseconds
    pub fn get_timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    /// Validate agent configuration
    fn validate_agent_config(
        &self,
        agent_id: &str,
        agent_type: &AgentType,
        capabilities: &AgentCapabilities,
    ) -> Result<(), String> {
        // Agent ID must not be empty
        if agent_id.is_empty() {
            return Err("Agent ID cannot be empty".to_string());
        }

        // Validate capabilities match agent type
        match agent_type {
            AgentType::Sensory => {
                if capabilities.vision.is_none()
                    && capabilities.sensory.is_none()
                    && capabilities.custom.is_empty()
                {
                    return Err("Sensory agent must have at least one input capability".to_string());
                }
            }
            AgentType::Motor => {
                if capabilities.motor.is_none() {
                    return Err("Motor agent must have motor capability".to_string());
                }
            }
            AgentType::Both => {
                // Both requires at least one capability of each type
                if (capabilities.vision.is_none()
                    && capabilities.sensory.is_none()
                    && capabilities.custom.is_empty())
                    || capabilities.motor.is_none()
                {
                    return Err(
                        "Bidirectional agent must have both input and output capabilities"
                            .to_string(),
                    );
                }
            }
            AgentType::Visualization => {
                if capabilities.visualization.is_none() {
                    return Err(
                        "Visualization agent must have visualization capability".to_string()
                    );
                }
            }
            AgentType::Infrastructure => {
                // Infrastructure agents are flexible - they can proxy any type
                // Just require at least one capability to be declared
                if capabilities.vision.is_none()
                    && capabilities.sensory.is_none()
                    && capabilities.motor.is_none()
                    && capabilities.visualization.is_none()
                    && capabilities.custom.is_empty()
                {
                    return Err(
                        "Infrastructure agent must declare at least one capability".to_string()
                    );
                }
            }
        }

        Ok(())
    }
}

const AGENT_HASH_SAFE_MASK: u64 = (1u64 << 53) - 1;

fn hash_optional_string(value: &Option<String>, hasher: &mut DefaultHasher) {
    match value {
        Some(text) => {
            hasher.write_u8(1);
            text.hash(hasher);
        }
        None => {
            hasher.write_u8(0);
        }
    }
}

fn hash_json_value(value: &serde_json::Value, hasher: &mut DefaultHasher) {
    match value {
        serde_json::Value::Null => {
            hasher.write_u8(0);
        }
        serde_json::Value::Bool(val) => {
            hasher.write_u8(1);
            hasher.write_u8(*val as u8);
        }
        serde_json::Value::Number(num) => {
            hasher.write_u8(2);
            num.to_string().hash(hasher);
        }
        serde_json::Value::String(text) => {
            hasher.write_u8(3);
            text.hash(hasher);
        }
        serde_json::Value::Array(values) => {
            hasher.write_u8(4);
            for item in values {
                hash_json_value(item, hasher);
            }
        }
        serde_json::Value::Object(map) => {
            hasher.write_u8(5);
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            for key in keys {
                key.hash(hasher);
                if let Some(item) = map.get(key) {
                    hash_json_value(item, hasher);
                } else {
                    hasher.write_u8(0);
                }
            }
        }
    }
}
