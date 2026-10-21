//! Connectome requests: inspecting, exporting and importing the neurons, synapses and cortical
//! areas of the running brain. Converted from the old `connectome` endpoints, for reference only

use std::collections::HashMap;

use feagi_basis::feagi_basis_proc_common::define_request_message_category;

/// ID and name of one cortical area
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectomeCorticalAreaSummary {
    pub cortical_id: String,
    pub cortical_name: String,
}

/// Details of one cortical area, as returned by the area details request
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectomeAreaDetails {
    pub cortical_id: String,
}

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
        "CorticalAreasListDetailed": {
            path: "cortical_areas/list/detailed",
            description: "Get every cortical area with all its properties",
            alloc_only: true,
            response: {
                areas: HashMap<String, serde_json::Value>, "Serialized cortical area per cortical ID",
            },
        },
        "CorticalAreasListSummary": {
            path: "cortical_areas/list/summary",
            description: "Get the ID and name of every cortical area",
            alloc_only: true,
            response: {
                areas: Vec<ConnectomeCorticalAreaSummary>,
            },
        },
        "CorticalAreasListTransforming": {
            path: "cortical_areas/list/transforming",
            description: "Get the cortical areas currently being transformed",
            alloc_only: true,
            response: {
                areas: Vec<String>, "Cortical IDs",
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
        "CorticalAreaNeurons": {
            path: "cortical_area/:cortical_id<String>/neurons",
            path_item_descriptions: {
                cortical_id: "Cortical area ID",
            },
            description: "Get the IDs of every neuron in a cortical area",
            alloc_only: true,
            response: {
                neuron_ids: Vec<u64>,
            },
        },
        "PropertiesDimensions": {
            path: "properties/dimensions",
            description: "Get the maximum dimensions of the connectome",
            response: {
                dimensions: [u64; 3],
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
        "StatsCorticalCumulative": {
            path: "stats/cortical_area/cumulative/:cortical_area<String>",
            path_item_descriptions: {
                cortical_area: "Cortical area ID",
            },
            description: "Get cumulative firing statistics of a cortical area",
            response: {
                total_fires: i64,
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
        "Paths": {
            path: "paths",
            description: "Get the paths between cortical areas",
            alloc_only: true,
            response: {
                paths: Vec<Vec<String>>, "Each path as a list of cortical IDs",
            },
        },
        "Path": {
            path: "path",
            description: "Get a path between cortical areas",
            alloc_only: true,
            response: {
                paths: Vec<Vec<String>>, "Each path as a list of cortical IDs",
            },
        },
        "CumulativeStats": {
            path: "cumulative_stats",
            description: "Get cumulative connectome statistics",
            response: {
                total_bursts: i64,
            },
        },
        "AreaDetails": {
            path: "area_details",
            description: "Get details of several cortical areas",
            alloc_only: true,
            payload: {
                area_ids: String, "Comma-separated cortical IDs",
            },
            response: {
                areas: HashMap<String, ConnectomeAreaDetails>, "Details per cortical ID (unknown IDs are skipped)",
            },
        },
        "Structure": {
            path: "structure",
            description: "Get the structure of the connectome",
            response: {
                cortical_areas: u64, "Number of cortical areas",
            },
        },
        "Validation": {
            path: "validation",
            description: "Validate the connectome",
            response: {
                valid: bool,
            },
        },
        "Topology": {
            path: "topology",
            description: "Get the topology of the connectome",
            response: {
                layers: u64,
            },
        },
        "ConnectivityMatrix": {
            path: "connectivity_matrix",
            description: "Get the connectivity matrix between cortical areas",
            alloc_only: true,
            response: {
                matrix: Vec<Vec<i32>>,
            },
        },
        "AreaSynapses": {
            path: ":cortical_area_id<String>/synapses",
            path_item_descriptions: {
                cortical_area_id: "Cortical area ID",
            },
            description: "Get the outgoing synapses whose source neuron is in a cortical area",
            alloc_only: true,
            response: {
                synapses: Vec<HashMap<String, serde_json::Value>>,
            },
            error: {
                AreaNotFound: "No cortical area has that ID",
                IntrospectionUnavailable: "The NPU cannot enumerate synapses",
            },
        },
        "AreaSynapsesIncoming": {
            path: ":cortical_area_id<String>/synapses/incoming",
            path_item_descriptions: {
                cortical_area_id: "Cortical area ID",
            },
            description: "Get the incoming synapses whose target neuron is in a cortical area",
            alloc_only: true,
            response: {
                synapses: Vec<HashMap<String, serde_json::Value>>,
            },
            error: {
                AreaNotFound: "No cortical area has that ID",
                IntrospectionUnavailable: "The NPU cannot enumerate synapses",
            },
        },
        "CorticalInfo": {
            path: "cortical_info/:cortical_area<String>",
            path_item_descriptions: {
                cortical_area: "Cortical area ID",
            },
            description: "Get the ID and name of a cortical area",
            alloc_only: true,
            response: {
                cortical_id: String,
                cortical_name: String,
            },
            error: {
                AreaNotFound: "No cortical area has that ID",
            },
        },
        "NeuronPropertiesById": {
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
        "NeuronPropertiesAt": {
            path: "neuron_properties_at",
            description: "Get the live properties of the neuron at a coordinate of a cortical area, for clients without neuron IDs",
            alloc_only: true,
            payload: {
                cortical_id: String, "Cortical area ID (base64)",
                x: u32,
                y: u32,
                z: u32,
            },
            response: {
                properties: HashMap<String, serde_json::Value>,
            },
            error: {
                AreaNotFound: "No cortical area has that ID",
                IntrospectionUnavailable: "The NPU cannot resolve coordinates to neurons",
            },
        },
        "AreaNeurons": {
            path: "area_neurons",
            description: "Get the IDs of the neurons in a cortical area",
            alloc_only: true,
            response: {
                neuron_ids: Vec<u64>,
            },
        },
        "FireQueueArea": {
            path: "fire_queue/:cortical_area<String>",
            path_item_descriptions: {
                cortical_area: "Cortical area ID",
            },
            description: "Get the fire queue of a cortical area",
            alloc_only: true,
            response: {
                fire_queue: HashMap<String, serde_json::Value>,
            },
        },
        "Plasticity": {
            path: "plasticity",
            description: "Get whether plasticity is enabled",
            response: {
                enabled: bool,
            },
        },
        "MemoryNeuron": {
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
        "Download": {
            path: "download",
            description: "Export the whole connectome",
            alloc_only: true,
            response: {
                connectome: serde_json::Value, "Serialized connectome snapshot",
            },
        },
        "DownloadCorticalArea": {
            path: "download-cortical_area-area/:cortical_area<String>",
            path_item_descriptions: {
                cortical_area: "Cortical area ID",
            },
            description: "Export one cortical area of the connectome",
            alloc_only: true,
            response: {
                data: HashMap<String, serde_json::Value>,
            },
        },
    },
    create: {
        "BatchNeuronOperations": {
            path: "batch_neuron_operations",
            description: "Apply a batch of neuron operations",
            alloc_only: true,
            payload: {
                operations: Vec<HashMap<String, serde_json::Value>>,
            },
            response: {
                processed: i32, "Number of operations processed",
            },
        },
        "BatchSynapseOperations": {
            path: "batch_synapse_operations",
            description: "Apply a batch of synapse operations",
            alloc_only: true,
            payload: {
                operations: Vec<HashMap<String, serde_json::Value>>,
            },
            response: {
                processed: i32, "Number of operations processed",
            },
        },
        "NeuronsBatch": {
            path: "neurons/batch",
            description: "Apply a batch of neuron operations",
            alloc_only: true,
            payload: {
                operations: Vec<HashMap<String, serde_json::Value>>,
            },
            response: {
                processed: i32, "Number of operations processed",
            },
        },
        "SynapsesBatch": {
            path: "synapses/batch",
            description: "Apply a batch of synapse operations",
            alloc_only: true,
            payload: {
                operations: Vec<HashMap<String, serde_json::Value>>,
            },
            response: {
                processed: i32, "Number of operations processed",
            },
        },
        "Rebuild": {
            path: "rebuild",
            description: "Rebuild the connectome (not yet implemented)",
            alloc_only: true,
            response: {
                message: String,
            },
        },
        "Clear": {
            path: "clear",
            description: "Clear the connectome (not yet implemented)",
            alloc_only: true,
            response: {
                message: String,
            },
        },
        "Optimize": {
            path: "optimize",
            description: "Optimize the connectome (not yet implemented)",
            alloc_only: true,
            response: {
                message: String,
            },
        },
        "Upload": {
            path: "upload",
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
        "UploadCorticalArea": {
            path: "upload-cortical_area-area",
            description: "Import one cortical area (not yet implemented)",
            alloc_only: true,
            payload: {
                data: HashMap<String, serde_json::Value>,
            },
            response: {
                message: String,
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
