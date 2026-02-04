//! Core agent client types (AgentClient, AgentConfig, AgentType). SDK surface for desktop/controllers.

pub use feagi_services::types::agent_registry::AgentType;

/// Configuration for an agent (id, type, and optional registration endpoint).
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub agent_id: String,
    pub agent_type: AgentType,
    pub registration_endpoint: String,
}

impl AgentConfig {
    pub fn new(agent_id: String, agent_type: AgentType) -> Self {
        Self {
            registration_endpoint: String::new(),
            agent_id,
            agent_type,
        }
    }

    pub fn with_registration_endpoint(mut self, endpoint: String) -> Self {
        self.registration_endpoint = endpoint;
        self
    }

    pub fn with_sensory_endpoint(self, _endpoint: String) -> Self {
        self
    }

    pub fn with_heartbeat_interval(self, _interval_s: f64) -> Self {
        self
    }

    pub fn with_connection_timeout_ms(self, _ms: u64) -> Self {
        self
    }

    pub fn with_registration_retries(self, _retries: u32) -> Self {
        self
    }

    pub fn with_sensory_capability(self, _tick_hz: f64, _capability: Option<()>) -> Self {
        self
    }
}

/// Placeholder agent client. Holds config; connect/run logic can be extended.
#[derive(Debug)]
pub struct AgentClient {
    pub config: AgentConfig,
    registered: std::cell::Cell<bool>,
}

impl AgentClient {
    pub fn new(config: AgentConfig) -> Result<Self, crate::FeagiAgentClientError> {
        Ok(Self {
            config,
            registered: std::cell::Cell::new(false),
        })
    }

    pub fn connect(&mut self) -> Result<(), crate::FeagiAgentClientError> {
        self.registered.set(true);
        Ok(())
    }

    pub fn is_registered(&self) -> bool {
        self.registered.get()
    }

    pub fn send_sensory_bytes(&mut self, _data: &[u8]) -> Result<(), crate::FeagiAgentClientError> {
        Ok(())
    }

    pub fn try_send_sensory_bytes(
        &mut self,
        data: &[u8],
    ) -> Result<bool, crate::FeagiAgentClientError> {
        self.send_sensory_bytes(data).map(|()| true)
    }

    pub fn reconnect_data_streams(&mut self) -> Result<(), crate::FeagiAgentClientError> {
        Ok(())
    }

    pub fn receive_motor_data(
        &mut self,
    ) -> Result<
        Option<feagi_structures::neuron_voxels::xyzp::CorticalMappedXYZPNeuronVoxels>,
        crate::FeagiAgentClientError,
    > {
        Ok(None)
    }
}
