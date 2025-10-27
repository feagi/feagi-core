// Heartbeat Tracker - monitors agent health and deregisters stale agents

use super::agent_registry::AgentRegistry;
use parking_lot::RwLock;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Heartbeat Tracker
pub struct HeartbeatTracker {
    running: Arc<RwLock<bool>>,
    thread_handle: Option<thread::JoinHandle<()>>,
    timeout: Duration,
    poll_interval: Duration,
}

impl HeartbeatTracker {
    pub fn new() -> Self {
        Self {
            running: Arc::new(RwLock::new(false)),
            thread_handle: None,
            timeout: Duration::from_secs(68), // Increased by 50% from 45s (was timing out prematurely)
            poll_interval: Duration::from_secs(10),
        }
    }

    /// Start heartbeat monitoring
    pub fn start(&mut self, agent_registry: Arc<RwLock<AgentRegistry>>) {
        if *self.running.read() {
            return;
        }

        *self.running.write() = true;
        let running = Arc::clone(&self.running);
        let timeout = self.timeout;
        let poll_interval = self.poll_interval;

        let handle = thread::spawn(move || {
            println!("🦀 [HEARTBEAT] Monitoring started (timeout: {:?})", timeout);

            while *running.read() {
                thread::sleep(poll_interval);

                if !*running.read() {
                    break;
                }

                // Check for stale agents
                let stale_agents = agent_registry.read().get_stale_agents();

                if !stale_agents.is_empty() {
                    println!("🦀 [HEARTBEAT] Found {} stale agent(s)", stale_agents.len());

                    // Deregister stale agents
                    for agent_id in stale_agents {
                        println!("🦀 [HEARTBEAT] Deregistering stale agent: {}", agent_id);
                        if let Err(e) = agent_registry.write().deregister(&agent_id) {
                            eprintln!(
                                "🦀 [HEARTBEAT] [ERR] Failed to deregister {}: {}",
                                agent_id, e
                            );
                        }
                    }
                }
            }

            println!("🦀 [HEARTBEAT] Monitoring stopped");
        });

        self.thread_handle = Some(handle);
    }

    /// Stop heartbeat monitoring
    pub fn stop(&mut self) {
        *self.running.write() = false;

        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }

    /// Set heartbeat timeout
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Set poll interval for checking agent heartbeats (useful for tests)
    pub fn set_poll_interval(&mut self, interval: Duration) {
        self.poll_interval = interval;
    }
}

impl Drop for HeartbeatTracker {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::agent_registry::{
        AgentCapabilities, AgentInfo, AgentTransport, AgentType, VisualizationCapability,
    };

    #[test]
    fn test_heartbeat_tracker() {
        let registry = Arc::new(RwLock::new(AgentRegistry::new(100, 50)));
        let mut tracker = HeartbeatTracker::new();
        tracker.set_timeout(Duration::from_millis(100));
        tracker.set_poll_interval(Duration::from_millis(20));

        // Register a test agent
        let mut capabilities = AgentCapabilities::default();
        capabilities.visualization = Some(VisualizationCapability {
            visualization_type: "test_viz".to_string(),
            resolution: None,
            refresh_rate: Some(30.0),
            bridge_proxy: false,
        });

        let mut agent_info = AgentInfo::new(
            "test-agent".to_string(),
            AgentType::Visualization,
            capabilities,
            AgentTransport::Zmq,
        );

        // Make the agent stale by rewinding timestamps
        let stale_offset_ms = 1_000;
        agent_info.last_seen = agent_info.last_seen.saturating_sub(stale_offset_ms);
        agent_info.registered_at = agent_info.registered_at.saturating_sub(stale_offset_ms);

        registry.write().register(agent_info).unwrap();
        assert_eq!(registry.read().count(), 1);

        // Start tracker (should deregister stale agent)
        tracker.start(Arc::clone(&registry));
        thread::sleep(Duration::from_millis(200));
        tracker.stop();

        // Agent should be deregistered
        assert_eq!(registry.read().count(), 0);
    }
}
