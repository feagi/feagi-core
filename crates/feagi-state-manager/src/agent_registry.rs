//! Agent registry - track connected agents

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(feature = "no_std")]
use heapless::FnvIndexMap as HashMap;

/// Agent information
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub agent_id: String,
    pub agent_type: String,
    pub registered_at: u64,
    pub last_seen: u64,
}

/// Agent registry
pub struct AgentRegistry {
    // TODO: Implement
    _phantom: std::marker::PhantomData<()>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

