//! Registration endpoint for handling agent registration requests.
//!
//! This module provides the `RegistrationEndpoint` struct which manages
//! the registration of agents (clients) connecting to the FEAGI server.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use feagi_io::next::traits_and_enums::server::server_shared::{FeagiServerBindState, FeagiServerBindStateChange};
use feagi_io::next::traits_and_enums::server::{FeagiServer, FeagiServerRouter, FeagiServerRouterProperties};
use feagi_io::next::FeagiNetworkError;
use crate::next::client::communication::auth_request::AuthRequest;
use crate::next::client::communication::registration_request::RegistrationRequest;
use crate::next::common::{AgentCapabilities, AgentDescriptor, ConnectionId, FeagiAgentError};
use crate::next::server::communication::{Phase1Response, Phase2Response};

/// Data stored for a phase 1 (initial) registration.
#[derive(Debug, Clone)]
pub struct Phase1RegistrationData {
    /// The agent descriptor from the registration request.
    pub agent_descriptor: AgentDescriptor,
    /// When this registration was created.
    pub registered_at: Instant,
}

impl Phase1RegistrationData {
    /// Create new phase 1 registration data.
    pub fn new(agent_descriptor: AgentDescriptor) -> Self {
        Self {
            agent_descriptor,
            registered_at: Instant::now(),
        }
    }
}

/// Data stored for a fully registered agent (phase 2 complete).
#[derive(Debug, Clone)]
pub struct RegisteredAgentData {
    /// The agent descriptor from the registration.
    pub agent_descriptor: AgentDescriptor,
    /// When this agent completed registration.
    pub registered_at: Instant,
    /// The capabilities this agent registered for.
    pub capabilities: Vec<AgentCapabilities>,
}

impl RegisteredAgentData {
    /// Create new registered agent data.
    pub fn new(agent_descriptor: AgentDescriptor, capabilities: Vec<AgentCapabilities>) -> Self {
        Self {
            agent_descriptor,
            registered_at: Instant::now(),
            capabilities,
        }
    }
}

/// Endpoint for handling agent registration requests.
///
/// This struct manages a `FeagiServerRouter` to receive registration requests
/// from agents and respond with registration confirmations.
pub struct RegistrationEndpoint {
    router: Box<dyn FeagiServerRouter>,
    router_state: Arc<Mutex<FeagiServerBindState>>,
    /// Phase 1 registrations: agents that have sent an AuthRequest but haven't completed full registration.
    phase1_registrations: HashMap<ConnectionId, Phase1RegistrationData>,
    /// Fully registered agents (phase 2 complete).
    registered_agents: HashMap<ConnectionId, RegisteredAgentData>,
}

impl RegistrationEndpoint {
    /// Create a new registration endpoint from router properties.
    pub fn new(properties: Box<dyn FeagiServerRouterProperties>) -> Result<Self, FeagiNetworkError>
    {
        let router_state = Arc::new(Mutex::new(FeagiServerBindState::Inactive));
        let state_ref = Arc::clone(&router_state);
        
        // Build the router with our internal state change handler
        let mut router = properties.build(Box::new(move |state_change: FeagiServerBindStateChange| {
            Self::handle_state_change(&state_ref, state_change);
        }));
        
        router.start()?;
        
        Ok(Self { 
            router, 
            router_state,
            phase1_registrations: HashMap::new(),
            registered_agents: HashMap::new(),
        })
    }

    /// Internal handler for router state changes.
    fn handle_state_change(
        state: &Arc<Mutex<FeagiServerBindState>>,
        state_change: FeagiServerBindStateChange,
    ) {
        if let Ok(mut current_state) = state.lock() {
            *current_state = state_change.get_now();
        }
        // TODO: Add logging or additional handling as needed
    }

    /// Get the current router state.
    pub fn get_router_state(&self) -> FeagiServerBindState {
        self.router_state.lock().map(|s| *s).unwrap_or(FeagiServerBindState::Inactive)
    }

    /// Get the number of phase 1 registrations.
    pub fn phase1_registration_count(&self) -> usize {
        self.phase1_registrations.len()
    }

    /// Get the number of fully registered agents.
    pub fn registered_agent_count(&self) -> usize {
        self.registered_agents.len()
    }

    /// Process incoming data and determine which phase/request type it is.
    fn process_request(&mut self, data: &[u8]) -> Result<Vec<u8>, FeagiAgentError> {
        // Try to parse as JSON first
        let json_value: serde_json::Value = serde_json::from_slice(data)
            .map_err(|e| FeagiAgentError::GeneralFailure(format!("Invalid JSON: {}", e)))?;

        // Check if this is a phase 1 AuthRequest (has agent_descriptor and auth_token)
        if json_value.get("agent_descriptor").is_some() && json_value.get("auth_token").is_some() {
            return self.handle_phase1_auth_request(&json_value);
        }

        // Check if this is a phase 2 RegistrationRequest (has connection_id and capabilities)
        if json_value.get("connection_id").is_some() && json_value.get("capabilities").is_some() {
            return self.handle_phase2_registration_request(&json_value);
        }

        // TODO: Add more request types here as needed
        
        Err(FeagiAgentError::GeneralFailure("Unknown request type".to_string()))
    }

    /// Handle phase 1: AuthRequest -> Generate ConnectionId and store registration.
    fn handle_phase1_auth_request(&mut self, json: &serde_json::Value) -> Result<Vec<u8>, FeagiAgentError> {
        // Parse the AuthRequest
        let auth_request = AuthRequest::from_json(json)?;
        
        // Get the agent descriptor
        let agent_descriptor = auth_request.agent_descriptor()?;
        
        // TODO: CRITICAL - Validate the auth token properly!
        // Currently we accept ANY token without validation.
        let _auth_token = auth_request.auth_token()?;
        
        // Generate a new connection ID
        let connection_id = ConnectionId::generate();
        
        // Store the phase 1 registration
        let registration_data = Phase1RegistrationData::new(agent_descriptor);
        self.phase1_registrations.insert(connection_id.clone(), registration_data);
        
        // Create and serialize the response
        let response = Phase1Response::new(&connection_id);
        Ok(response.to_json_bytes())
    }

    /// Handle phase 2: RegistrationRequest -> Complete registration and return endpoints.
    fn handle_phase2_registration_request(&mut self, json: &serde_json::Value) -> Result<Vec<u8>, FeagiAgentError> {
        // Parse the RegistrationRequest
        let registration_request = RegistrationRequest::from_json(json)?;
        
        // Get the connection ID
        let connection_id = registration_request.connection_id()?;
        
        // Check if this connection ID exists in phase 1 registrations
        let phase1_data = self.phase1_registrations.remove(&connection_id)
            .ok_or_else(|| FeagiAgentError::AuthenticationFailed(
                "Invalid or expired connection_id. Please complete phase 1 first.".to_string()
            ))?;
        
        // Create the fully registered agent data
        let registered_data = RegisteredAgentData::new(
            phase1_data.agent_descriptor,
            registration_request.capabilities.clone(),
        );
        
        // Store in registered agents
        self.registered_agents.insert(connection_id, registered_data);
        
        // Create response with endpoint addresses (placeholders for now)
        let response = Phase2Response::new(&registration_request.capabilities);
        Ok(response.to_json_bytes())
    }

    /// Run the registration endpoint loop.
    ///
    /// This async loop polls for incoming registration requests and processes them.
    /// Uses `spawn_blocking` for the polling to avoid blocking the tokio runtime.
    #[cfg(feature = "async")]
    pub async fn run(&mut self) -> Result<(), FeagiNetworkError> {
        loop {
            // Poll for incoming requests (non-blocking poll)
            // Copy data to owned Vec to avoid borrow conflict with process_request
            let received = self.router.try_poll_receive()?
                .map(|(client_id, data)| (client_id, data.to_vec()));
            
            if let Some((client_id, data)) = received {
                // Process the request
                let response_data = match self.process_request(&data) {
                    Ok(response) => response,
                    Err(e) => {
                        // Send error response as JSON
                        let error_response = serde_json::json!({
                            "error": e.to_string()
                        });
                        serde_json::to_vec(&error_response).unwrap_or_default()
                    }
                };
                
                self.router.send_response(client_id, &response_data)?;
            }
            
            // Yield to allow other async tasks to run
            tokio::task::yield_now().await;
        }
    }

    /// Run the registration endpoint loop (blocking version).
    ///
    /// This loop polls for incoming registration requests and processes them.
    pub fn run_blocking(&mut self) -> Result<(), FeagiNetworkError> {
        loop {
            // Poll for incoming requests
            // Copy data to owned Vec to avoid borrow conflict with process_request
            let received = self.router.try_poll_receive()?
                .map(|(client_id, data)| (client_id, data.to_vec()));
            
            if let Some((client_id, data)) = received {
                // Process the request
                let response_data = match self.process_request(&data) {
                    Ok(response) => response,
                    Err(e) => {
                        // Send error response as JSON
                        let error_response = serde_json::json!({
                            "error": e.to_string()
                        });
                        serde_json::to_vec(&error_response).unwrap_or_default()
                    }
                };
                
                self.router.send_response(client_id, &response_data)?;
            }
            
            // Small sleep to avoid busy-waiting
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// Stop the registration endpoint.
    pub fn stop(&mut self) -> Result<(), FeagiNetworkError> {
        self.router.stop()
    }
}
