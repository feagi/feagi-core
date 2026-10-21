//! Connectome requests: inspecting, exporting and importing the neurons, synapses and cortical
//! areas of the running brain. Converted from the old `connectome` endpoints, for reference only

use std::collections::HashMap;

use feagi_basis::feagi_basis_proc_common::define_request_message_category;


/// All cortical areas of one cortical subtype
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectomeCorticalTypeGroup {
    /// Human readable name of the subtype
    pub title: String,
    /// Base64 cortical IDs of this subtype, sorted
    pub cortical_ids: Vec<String>,
    /// Group IDs used by this subtype, sorted
    pub group_ids: Vec<u8>,
}

define_request_message_category! {
    category_name: "Connectome",
    base_path: "v1/connectome",
    category_description: "Inspection, statistics, export and import of the connectome (neurons, synapses and cortical areas)",
    read: {
        "CorticalAreasListDetailed": { // move off serde_json
            path: "cortical_areas/list/detailed",
            description: "Get every cortical area with all its properties",
            alloc_only: true,
            response: {
                areas: HashMap<String, serde_json::Value>, "Serialized cortical area per cortical ID",
            },
        },
        "CorticalAreaListTypes": {
            path: "cortical_area/list/types",
            description: "Get the cortical areas grouped by cortical subtype, with their IDs and group IDs",
            alloc_only: true,
            response: {
                types: HashMap<String, ConnectomeCorticalTypeGroup>, "Group per cortical subtype (e.g. svi, mot)",
            },
        },
        "PropertiesMappings": {
            path: "properties/mappings",
            description: "Get all cortical area mappings",
            alloc_only: true,
            response: {
                mappings: HashMap<String, Vec<String>>, "Destination cortical IDs per source cortical ID",
            },
        },
        "Snapshot": {
            path: "snapshot",
            description: "Get a count of cortical areas and brain regions",
            response: {
                cortical_area_count: u64,
                brain_region_count: u64,
            },
        },
        "Stats": {
            path: "stats",
            description: "Get neuron and cortical area counts",
            response: {
                neuron_count: u64,
                cortical_area_count: u64,
            },
        },
        "NeuronCount": {
            path: "neuron_count",
            description: "Get the total number of neurons",
            response: {
                count: i64,
            },
        },
        "SynapseCount": {
            path: "synapse_count",
            description: "Get the total number of synapses",
            response: {
                count: i64,
            },
        },
        "CumulativeStats": {
            path: "cumulative_stats",
            description: "Get cumulative connectome statistics",
            response: {
                total_bursts: i64,
            },
        },
        "NeuronPropertiesById": { // cannot be implemented
            path: "neuron/:neuron_id<u64>/properties",
            path_item_descriptions: {
                neuron_id: "Neuron ID",
            },
            description: "Get the properties of a neuron, by ID in the path",
            alloc_only: true,
            response: {
                properties: HashMap<String, serde_json::Value>,
            },
        },
        "NeuronProperties": {
            path: "neuron_properties",
            description: "Get the properties of a neuron, by ID in the query",
            alloc_only: true,
            payload: {
                neuron_id: u64,
            },
            response: {
                properties: HashMap<String, serde_json::Value>,
            },
        },
        "MemoryNeuron": { // cannot be implemented
            path: "memory_neuron",
            description: "Get the plasticity lifecycle record and synapses of a memory neuron",
            alloc_only: true,
            payload: {
                neuron_id: u32, "Global memory neuron ID (reserved range, typically 50_000_000+)",
            },
            response: {
                neuron_id: u64,
                cortical_idx: u32,
                cortical_id: String,
                cortical_name: String,
                pattern_hash: Option<u64>,
                is_longterm_memory: bool,
                is_active: bool,
                lifespan_current: u32,
                lifespan_initial: u32,
                lifespan_growth_rate: f32,
                creation_burst: u64,
                last_activation_burst: u64,
                activation_count: u32,
                outgoing_synapse_count: u64,
                incoming_synapse_count: u64,
                outgoing_synapses: serde_json::Value,
                incoming_synapses: serde_json::Value,
            },
            error: {
                InvalidNeuronId: "The neuron ID is not in the memory neuron range",
                NeuronNotFound: "No memory neuron has that ID",
                IntrospectionUnavailable: "The NPU cannot report memory neuron details",
            },
        },
        "Download": { // cannot be implemented as is, instead return the struct
            path: "download",
            description: "Export the whole connectome",
            alloc_only: true,
            response: {
                connectome: serde_json::Value, "Serialized connectome snapshot",
            },
        },
    },
    create: {
        "Upload": {
            path: "upload", // cannot be implemented as is
            description: "Import a whole connectome snapshot, replacing the current one",
            alloc_only: true,
            payload: {
                connectome: serde_json::Value, "Serialized connectome snapshot",
            },
            response: {
                message: String,
            },
            error: {
                InvalidSnapshot: "The data is not a valid connectome snapshot",
            },
        },
    },
}

// NOTES:
// Most old handlers returned bare maps/arrays/numbers/tuples; each is wrapped in one named field.
// Many old handlers were stubs (paths, path, area_neurons, fire_queue, topology, rebuild, clear, optimize, batch ops, uploads of areas).
// Paths, Path, AreaNeurons: old query params were never parsed, so payloads are omitted (unknown).
// Batch ops / UploadCorticalArea bodies were untyped maps; operation shape unknown (Value).
// Upload / Download: connectome snapshot kept as serde_json::Value (ConnectomeSnapshot lives in feagi_services); old body was the bare snapshot, now wrapped.
// CorticalAreasListDetailed, neuron properties, synapse lists: untyped Value maps in the old code.
// AreaSynapses(:id/synapses) is a top-level path parameter and may clash with static routes in a router.
// usize fields changed to u64.
