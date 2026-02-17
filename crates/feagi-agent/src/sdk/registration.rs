//! REST registration client (FeagiApiConfig, AgentRegistrar).

use std::time::Duration;

use crate::FeagiAgentClientError;

/// Configuration for the FEAGI REST API (host, port, timeout).
#[derive(Debug, Clone)]
pub struct FeagiApiConfig {
    pub host: String,
    pub port: u16,
    pub timeout: Duration,
}

impl FeagiApiConfig {
    pub fn new(host: String, port: u16, timeout: Duration) -> Self {
        Self {
            host,
            port,
            timeout,
        }
    }

    fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

/// Result of syncing device registrations to FEAGI (counts of created units/feedbacks).
#[derive(Debug, Clone)]
pub struct DeviceRegistrationCounts {
    pub input_units: u32,
    pub output_units: u32,
    pub feedbacks: u32,
}

/// Client for agent registration and topology over the FEAGI REST API.
#[derive(Clone)]
pub struct AgentRegistrar {
    config: FeagiApiConfig,
}

impl AgentRegistrar {
    pub fn new(config: FeagiApiConfig) -> Result<Self, FeagiAgentClientError> {
        Ok(Self { config })
    }

    /// Sync device registrations to FEAGI (stub: returns zero counts).
    #[cfg(feature = "sdk-io")]
    pub async fn sync_device_registrations(
        &self,
        _device_registrations: serde_json::Value,
        _agent_id: &str,
    ) -> Result<DeviceRegistrationCounts, FeagiAgentClientError> {
        Ok(DeviceRegistrationCounts {
            input_units: 0,
            output_units: 0,
            feedbacks: 0,
        })
    }

    /// Returns a client that can fetch topology from GET /v1/connectome/topology.
    #[cfg(feature = "sdk-io")]
    pub fn topology_cache(
        &self,
    ) -> Result<crate::sdk::base::TopologyClient, FeagiAgentClientError> {
        Ok(crate::sdk::base::TopologyClient::new(
            self.config.base_url(),
        ))
    }
}
