//! Agent requests: registering, monitoring and configuring the external agents (sensors, motors,
//! visualizers) that connect to FEAGI. Converted from the old `agent` endpoints, for reference only

use std::collections::HashMap;

use feagi_basis::feagi_basis_proc_common::define_request_message_category;

/// Connection details for one transport an agent can use
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AgentTransportConfig {
    /// Transport name, "zmq" or "websocket"
    pub transport_type: String,
    pub enabled: bool,
    /// Port per stream ("registration", "sensory", "motor", "visualization")
    pub ports: HashMap<String, u16>,
    /// Host the agent should connect to
    pub host: String,
}

/// Liveness state of one registered agent
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AgentLivenessEntry {
    /// Base64 agent id, matching the values from the agent list
    pub agent_id: String,
    pub agent_name: String,
    pub manufacturer: String,
    pub agent_version: u32,
    /// Capability keys granted at registration (e.g. `receive_neuron_visualizations`)
    pub capabilities: Vec<String>,
    /// Seconds since anything (inbound command/control or outbound pushes) touched this agent
    pub last_activity_age_s: Option<f64>,
    /// Seconds since the agent itself last sent command/control traffic
    pub last_command_control_age_s: Option<f64>,
    /// Seconds remaining before the stale pruner deregisters this agent, clamped at zero
    pub prune_in_s: Option<f64>,
}

/// Capabilities of one agent, optionally with its device registrations
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AgentCapabilitiesSummary {
    pub agent_name: String,
    pub capabilities: HashMap<String, serde_json::Value>,
    /// Device registration JSON, only when requested
    pub device_registrations: Option<serde_json::Value>,
}

/// State of one fire queue sampler
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FqSamplerState {
    pub enabled: bool,
    /// Human readable reason for the enabled state
    pub reason: String,
    /// Agent IDs that need this sampler
    pub agents_requiring: Vec<String>,
    pub frequency_hz: f64,
}

/// Fire queue sampler coordination, per sampler kind
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FqSamplerCoordination {
    pub visualization_fq_sampler: FqSamplerState,
    pub motor_fq_sampler: FqSamplerState,
}

/// Summary of the agent registry
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AgentRegistrySummary {
    pub total_agents: u64,
    pub agent_ids: Vec<String>,
}

define_request_message_category! {
    category_name: "Agent",
    base_path: "v1/agent",
    category_description: "Registration, liveness, properties and device registrations of the external agents connected to FEAGI",
    read: {
        "List": {
            path: "list",
            description: "Get a list of all currently registered agent IDs",
            alloc_only: true,
            response: {
                agent_ids: Vec<String>, "Base64 IDs of every registered agent",
            },
        },
        "Liveness": {
            path: "liveness",
            description: "Report how close each registered agent is to being deregistered for inactivity",
            alloc_only: true,
            response: {
                heartbeat_timeout_s: f64, "Quiet period after which an agent is deregistered",
                stale_check_interval_s: f64, "How often the stale scan runs",
                agents: Vec<AgentLivenessEntry>, "One entry per registered agent, ordered by agent id",
                count: u64, "Number of agents",
            },
        },
        "Properties": {
            path: "properties",
            description: "Get agent properties including type, capabilities, version, and connection details, by agent ID query parameter",
            alloc_only: true,
            payload: {
                agent_id: String, "Agent ID to get properties for",
            },
            response: {
                agent_name: String,
                agent_type: String,
                agent_ip: String,
                agent_data_port: u16,
                agent_router_address: String,
                agent_version: String,
                controller_version: String,
                capabilities: HashMap<String, serde_json::Value>, "Capabilities declared at registration",
                chosen_transport: Option<String>, "Transport the agent chose (zmq, websocket, shm, ...)",
            },
            error: {
                AgentNotFound: "No agent has that ID",
            },
        },
        "PropertiesById": {
            path: "properties/:agent_id<String>",
            path_item_descriptions: {
                agent_id: "Agent ID",
            },
            description: "Get agent properties, with the agent ID in the path",
            alloc_only: true,
            response: {
                agent_name: String,
                agent_type: String,
                agent_ip: String,
                agent_data_port: u16,
                agent_router_address: String,
                agent_version: String,
                controller_version: String,
                capabilities: HashMap<String, serde_json::Value>, "Capabilities declared at registration",
                chosen_transport: Option<String>, "Transport the agent chose (zmq, websocket, shm, ...)",
            },
            error: {
                AgentNotFound: "No agent has that ID",
            },
        },
        "Info": {
            path: "info/:agent_id<String>",
            path_item_descriptions: {
                agent_id: "Agent ID",
            },
            description: "Get comprehensive agent information including status, capabilities, version, and connection details",
            alloc_only: true,
            response: {
                agent_id: String,
                agent_name: String,
                agent_type: String,
                agent_ip: String,
                agent_data_port: u16,
                capabilities: HashMap<String, serde_json::Value>,
                agent_version: String,
                controller_version: String,
                status: String, "Always \"active\"",
                chosen_transport: Option<String>,
            },
            error: {
                AgentNotFound: "No agent has that ID",
            },
        },
        "SharedMemory": {
            path: "shared_mem",
            description: "Get shared memory configuration and paths for all registered agents using shared memory transport",
            alloc_only: true,
            response: {
                agents: HashMap<String, HashMap<String, serde_json::Value>>, "Shared memory info per agent ID",
            },
        },
        "FqSamplerStatus": {
            path: "fq_sampler_status",
            description: "Get fire queue sampler coordination status including visualization and motor sampling configuration",
            alloc_only: true,
            response: {
                fq_sampler_coordination: FqSamplerCoordination,
                agent_registry: AgentRegistrySummary,
                system_status: String,
                fcl_sampler_consumer: serde_json::Value, "FCL sampler consumer as reported by the runtime",
            },
        },
        "Capabilities": {
            path: "capabilities",
            description: "Get the list of all supported agent types and capability types",
            alloc_only: true,
            response: {
                agent_types: Vec<String>, "e.g. sensory, motor, both, visualization, infrastructure",
                capability_types: Vec<String>, "e.g. vision, motor, visualization, sensory",
            },
        },
        "AllCapabilities": {
            path: "capabilities/all",
            description: "Get capabilities for all agents, with optional filtering",
            alloc_only: true,
            payload: {
                agent_type: Option<String>, "Filter by agent type (exact match)",
                capability: Option<String>, "Filter by capability key(s), comma-separated",
                include_device_registrations: Option<bool>, "Include device registration payloads per agent",
            },
            response: {
                agents: HashMap<String, AgentCapabilitiesSummary>, "Capabilities per agent ID",
            },
        },
        "DeviceRegistrations": {
            path: ":agent_id<String>/device_registrations",
            path_item_descriptions: {
                agent_id: "Agent ID",
            },
            description: "Export the device registrations (sensor/motor devices, encoder/decoder properties, feedbacks) of an agent",
            alloc_only: true,
            response: {
                device_registrations: serde_json::Value, "Device registration JSON, as from ConnectorAgent::get_device_registration_json",
                agent_id: String, "Agent ID this configuration belongs to",
            },
        },
    },
    create: {
        "Register": {
            path: "register",
            description: "Register a new agent with FEAGI and receive connection details including transport configuration and ports",
            alloc_only: true,
            payload: {
                agent_type: String, "Type of agent (e.g. brain_visualizer, video_agent)",
                agent_id: String, "Unique identifier for the agent",
                agent_data_port: u16, "Port the agent is listening on for data",
                agent_version: String, "Version of the agent software",
                controller_version: String, "Version of the controller",
                capabilities: HashMap<String, serde_json::Value>, "Agent capabilities (device_registrations, visualization.rate_hz, ...)",
                agent_ip: Option<String>, "Agent IP address, taken from the request if not provided",
                metadata: Option<HashMap<String, serde_json::Value>>, "Additional metadata",
                auth_token: Option<String>, "Base64 auth token",
                chosen_transport: Option<String>, "Transport the agent chose (zmq, websocket, shm, ...)",
            },
            response: {
                status: String,
                message: String,
                success: bool,
                transport: Option<HashMap<String, serde_json::Value>>, "Legacy, deprecated",
                rates: Option<HashMap<String, HashMap<String, f64>>>,
                transports: Option<Vec<AgentTransportConfig>>, "Available transports",
                recommended_transport: Option<String>,
                shm_paths: Option<HashMap<String, String>>,
                cortical_areas: serde_json::Value, "Cortical area availability status for agent operations",
            },
        },
        "Heartbeat": {
            path: "heartbeat",
            description: "Send a heartbeat to keep the agent registered and prevent timeout disconnection",
            alloc_only: true,
            payload: {
                agent_id: String,
            },
            response: {
                message: String,
                success: bool,
            },
            error: {
                AgentNotFound: "The agent is not in the registry",
            },
        },
        "ManualStimulation": {
            path: "manual_stimulation",
            description: "Manually stimulate neurons at specific coordinates across multiple cortical areas, for testing and debugging",
            alloc_only: true,
            payload: {
                stimulation_payload: HashMap<String, Vec<Vec<i32>>>, "Map of cortical area IDs to lists of [x, y, z] coordinates",
                mode: Option<String>, "\"candidate\" (default) or \"force_fire\"",
            },
            response: {
                success: bool,
                total_coordinates: u64,
                requested_coordinates: u64,
                matched_coordinates: u64,
                unique_neuron_ids: u64,
                mode: String,
                successful_areas: u64,
                failed_areas: Vec<String>,
                error: Option<String>,
            },
            error: {
                InvalidMode: "The stimulation mode is not candidate or force_fire",
            },
        },
        "Configure": {
            path: "configure",
            description: "Configure agent parameters and settings (not yet implemented)",
            alloc_only: true,
            payload: {
                config: HashMap<String, serde_json::Value>, "Configuration parameters",
            },
            response: {
                message: String,
                status: String,
            },
        },
        "DeviceRegistrations": {
            path: ":agent_id<String>/device_registrations",
            path_item_descriptions: {
                agent_id: "Agent ID",
            },
            description: "Import device registrations for an agent, wiping all existing registered devices first",
            alloc_only: true,
            payload: {
                device_registrations: serde_json::Value, "Must contain input_units_and_encoder_properties, output_units_and_decoder_properties and feedbacks",
            },
            response: {
                success: bool,
                message: String,
                agent_id: String,
            },
            error: {
                InvalidDeviceRegistrations: "The device registrations are not an object with the required fields",
            },
        },
    },
    delete: {
        "Deregister": {
            path: "deregister",
            description: "Deregister an agent from FEAGI and clean up its resources",
            alloc_only: true,
            payload: {
                agent_id: String,
            },
            response: {
                message: String,
                success: Option<bool>,
            },
        },
    },
}

// NOTES:
// List, SharedMemory, AllCapabilities: old responses were a bare array/map, wrapped in one field.
// Configure: old body was a bare map, wrapped in `config`; endpoint was a stub.
// FqSamplerStatus: old response was an untyped map; fields inferred from the handler. fcl_sampler_consumer type unknown (Value).
// DeviceRegistrations: device registration JSON kept as serde_json::Value (shape only partly known).
// Capabilities / metadata / transport maps are untyped Value maps in the old DTOs.
// Info: old response was an untyped map; fields inferred from the handler.
// Deregister: old DELETE took a JSON body; kept as payload.
// usize fields from the old DTOs changed to u64.
