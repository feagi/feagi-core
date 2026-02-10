// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Agent registry - track connected agents
//!
//! Platform-specific locking implementation:
//! - std: parking_lot::RwLock (read-optimized)
//! - no_std: spin::RwLock
//! - wasm (single-threaded): RefCell (no locking needed)
//! - wasm-threaded: wasm_sync::Mutex

use crate::{Result, StateError};

// Platform-specific imports
#[cfg(all(feature = "std", not(target_family = "wasm")))]
use parking_lot::RwLock;

#[cfg(all(feature = "no_std", not(target_family = "wasm")))]
use spin::RwLock;

#[cfg(all(target_family = "wasm", not(feature = "wasm-threaded")))]
use std::cell::RefCell;

#[cfg(all(target_family = "wasm", feature = "wasm-threaded"))]
use wasm_sync::Mutex;

// Platform-specific collections
#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(feature = "no_std")]
use ahash::AHashMap as HashMap;

/// Agent type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentType {
    Sensory,
    Motor,
    Both,
    Visualization,
    Infrastructure,
}

impl AgentType {
    /// Parse agent type from string (case-insensitive)
    pub fn parse_from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "sensory" => AgentType::Sensory,
            "motor" => AgentType::Motor,
            "both" => AgentType::Both,
            "visualization" => AgentType::Visualization,
            "infrastructure" => AgentType::Infrastructure,
            _ => AgentType::Sensory,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AgentType::Sensory => "sensory",
            AgentType::Motor => "motor",
            AgentType::Both => "both",
            AgentType::Visualization => "visualization",
            AgentType::Infrastructure => "infrastructure",
        }
    }
}

/// Agent information
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub agent_id: String,
    pub agent_type: AgentType,
    pub registered_at: u64,
    pub last_seen: u64,
    pub metadata: Option<String>, // JSON string or other metadata
}

impl AgentInfo {
    pub fn new(agent_id: String, agent_type: AgentType) -> Self {
        Self {
            agent_id,
            agent_type,
            registered_at: Self::current_timestamp(),
            last_seen: Self::current_timestamp(),
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: String) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = Self::current_timestamp();
    }

    #[cfg(feature = "std")]
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    #[cfg(not(feature = "std"))]
    fn current_timestamp() -> u64 {
        // For no_std, use a counter (will be synchronized with external clock)
        0
    }
}

// ===== Platform-Specific Agent Registry Implementations =====

/// Agent registry for std platforms (parking_lot::RwLock)
#[cfg(all(feature = "std", not(target_family = "wasm")))]
pub struct AgentRegistry {
    agents: RwLock<HashMap<String, AgentInfo>>,
}

#[cfg(all(feature = "std", not(target_family = "wasm")))]
impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(HashMap::new()),
        }
    }

    pub fn register(&self, info: AgentInfo) -> Result<()> {
        let mut agents = self.agents.write();
        if agents.contains_key(&info.agent_id) {
            return Err(StateError::AgentAlreadyRegistered(info.agent_id.clone()));
        }
        agents.insert(info.agent_id.clone(), info);
        Ok(())
    }

    pub fn deregister(&self, agent_id: &str) -> Result<()> {
        let mut agents = self.agents.write();
        agents
            .remove(agent_id)
            .ok_or_else(|| StateError::AgentNotFound(agent_id.to_string()))?;
        Ok(())
    }

    pub fn get(&self, agent_id: &str) -> Option<AgentInfo> {
        let agents = self.agents.read();
        agents.get(agent_id).cloned()
    }

    pub fn get_all(&self) -> Vec<AgentInfo> {
        let agents = self.agents.read();
        agents.values().cloned().collect()
    }

    pub fn count(&self) -> usize {
        let agents = self.agents.read();
        agents.len()
    }

    pub fn update_heartbeat(&self, agent_id: &str) -> Result<()> {
        let mut agents = self.agents.write();
        agents
            .get_mut(agent_id)
            .map(|agent| agent.update_last_seen())
            .ok_or_else(|| StateError::AgentNotFound(agent_id.to_string()))
    }

    /// Alias for update_heartbeat for compatibility
    pub fn heartbeat(&self, agent_id: &str) -> Result<()> {
        self.update_heartbeat(agent_id)
    }
}

/// Agent registry for no_std platforms (spin::RwLock)
#[cfg(all(feature = "no_std", not(target_family = "wasm")))]
pub struct AgentRegistry {
    agents: RwLock<HashMap<String, AgentInfo>>,
}

#[cfg(all(feature = "no_std", not(target_family = "wasm")))]
impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(HashMap::default()),
        }
    }

    pub fn register(&self, info: AgentInfo) -> Result<()> {
        let mut agents = self.agents.write();
        if agents.contains_key(&info.agent_id) {
            return Err(StateError::AgentAlreadyRegistered(info.agent_id.clone()));
        }
        agents.insert(info.agent_id.clone(), info);
        Ok(())
    }

    pub fn deregister(&self, agent_id: &str) -> Result<()> {
        let mut agents = self.agents.write();
        agents
            .remove(agent_id)
            .ok_or_else(|| StateError::AgentNotFound(agent_id.to_string()))?;
        Ok(())
    }

    pub fn get(&self, agent_id: &str) -> Option<AgentInfo> {
        let agents = self.agents.read();
        agents.get(agent_id).cloned()
    }

    pub fn get_all(&self) -> Vec<AgentInfo> {
        let agents = self.agents.read();
        agents.values().cloned().collect()
    }

    pub fn count(&self) -> usize {
        let agents = self.agents.read();
        agents.len()
    }

    pub fn update_heartbeat(&self, agent_id: &str) -> Result<()> {
        let mut agents = self.agents.write();
        agents
            .get_mut(agent_id)
            .map(|agent| agent.update_last_seen())
            .ok_or_else(|| StateError::AgentNotFound(agent_id.to_string()))
    }

    /// Alias for update_heartbeat for compatibility
    pub fn heartbeat(&self, agent_id: &str) -> Result<()> {
        self.update_heartbeat(agent_id)
    }
}

/// Agent registry for single-threaded WASM (RefCell, no locking)
#[cfg(all(target_family = "wasm", not(feature = "wasm-threaded")))]
pub struct AgentRegistry {
    agents: RefCell<HashMap<String, AgentInfo>>,
}

#[cfg(all(target_family = "wasm", not(feature = "wasm-threaded")))]
impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: RefCell::new(HashMap::new()),
        }
    }

    pub fn register(&self, info: AgentInfo) -> Result<()> {
        let mut agents = self.agents.borrow_mut();
        if agents.contains_key(&info.agent_id) {
            return Err(StateError::AgentAlreadyRegistered(info.agent_id.clone()));
        }
        agents.insert(info.agent_id.clone(), info);
        Ok(())
    }

    pub fn deregister(&self, agent_id: &str) -> Result<()> {
        let mut agents = self.agents.borrow_mut();
        agents
            .remove(agent_id)
            .ok_or_else(|| StateError::AgentNotFound(agent_id.to_string()))?;
        Ok(())
    }

    pub fn get(&self, agent_id: &str) -> Option<AgentInfo> {
        let agents = self.agents.borrow();
        agents.get(agent_id).cloned()
    }

    pub fn get_all(&self) -> Vec<AgentInfo> {
        let agents = self.agents.borrow();
        agents.values().cloned().collect()
    }

    pub fn count(&self) -> usize {
        let agents = self.agents.borrow();
        agents.len()
    }

    pub fn update_heartbeat(&self, agent_id: &str) -> Result<()> {
        let mut agents = self.agents.borrow_mut();
        agents
            .get_mut(agent_id)
            .map(|agent| agent.update_last_seen())
            .ok_or_else(|| StateError::AgentNotFound(agent_id.to_string()))
    }

    /// Alias for update_heartbeat for compatibility
    pub fn heartbeat(&self, agent_id: &str) -> Result<()> {
        self.update_heartbeat(agent_id)
    }
}

/// Agent registry for multi-threaded WASM (wasm_sync::Mutex)
#[cfg(all(target_family = "wasm", feature = "wasm-threaded"))]
pub struct AgentRegistry {
    agents: Mutex<HashMap<String, AgentInfo>>,
}

#[cfg(all(target_family = "wasm", feature = "wasm-threaded"))]
impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: Mutex::new(HashMap::new()),
        }
    }

    pub fn register(&self, info: AgentInfo) -> Result<()> {
        let mut agents = self.agents.lock().unwrap();
        if agents.contains_key(&info.agent_id) {
            return Err(StateError::AgentAlreadyRegistered(info.agent_id.clone()));
        }
        agents.insert(info.agent_id.clone(), info);
        Ok(())
    }

    pub fn deregister(&self, agent_id: &str) -> Result<()> {
        let mut agents = self.agents.lock().unwrap();
        agents
            .remove(agent_id)
            .ok_or_else(|| StateError::AgentNotFound(agent_id.to_string()))?;
        Ok(())
    }

    pub fn get(&self, agent_id: &str) -> Option<AgentInfo> {
        let agents = self.agents.lock().unwrap();
        agents.get(agent_id).cloned()
    }

    pub fn get_all(&self) -> Vec<AgentInfo> {
        let agents = self.agents.lock().unwrap();
        agents.values().cloned().collect()
    }

    pub fn count(&self) -> usize {
        let agents = self.agents.lock().unwrap();
        agents.len()
    }

    pub fn update_heartbeat(&self, agent_id: &str) -> Result<()> {
        let mut agents = self.agents.lock().unwrap();
        agents
            .get_mut(agent_id)
            .map(|agent| agent.update_last_seen())
            .ok_or_else(|| StateError::AgentNotFound(agent_id.to_string()))
    }

    /// Alias for update_heartbeat for compatibility
    pub fn heartbeat(&self, agent_id: &str) -> Result<()> {
        self.update_heartbeat(agent_id)
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_registry_register() {
        let registry = AgentRegistry::new();
        let agent = AgentInfo::new("agent1".to_string(), AgentType::Sensory);

        assert!(registry.register(agent).is_ok());
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_agent_registry_duplicate() {
        let registry = AgentRegistry::new();
        let agent1 = AgentInfo::new("agent1".to_string(), AgentType::Sensory);
        let agent2 = AgentInfo::new("agent1".to_string(), AgentType::Motor);

        registry.register(agent1).unwrap();
        assert!(registry.register(agent2).is_err());
    }

    #[test]
    fn test_agent_registry_deregister() {
        let registry = AgentRegistry::new();
        let agent = AgentInfo::new("agent1".to_string(), AgentType::Sensory);

        registry.register(agent).unwrap();
        assert_eq!(registry.count(), 1);

        registry.deregister("agent1").unwrap();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_agent_registry_get() {
        let registry = AgentRegistry::new();
        let agent = AgentInfo::new("agent1".to_string(), AgentType::Sensory);

        registry.register(agent).unwrap();

        let retrieved = registry.get("agent1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().agent_id, "agent1");
    }

    #[test]
    fn test_agent_registry_get_all() {
        let registry = AgentRegistry::new();

        registry
            .register(AgentInfo::new("agent1".to_string(), AgentType::Sensory))
            .unwrap();
        registry
            .register(AgentInfo::new("agent2".to_string(), AgentType::Motor))
            .unwrap();

        let all = registry.get_all();
        assert_eq!(all.len(), 2);
    }
}
