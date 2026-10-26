//! Cortical area requests: listing, inspecting, creating, cloning, updating, resetting and
//! deleting cortical areas, plus IPU/OPU type metadata and per-voxel / memory-area introspection.
//!
//! Converted (for reference only) from the old `cortical_area` REST endpoints.

use feagi_basis::feagi_basis_proc_common::define_request_message_category;
use std::collections::HashMap;
use crate::basis::cortical_area::cortical_id::Base64EncodedCorticalID;

/// An x/y/z triple keyed by axis name (signed, used for positions)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CorticalXyzI32 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// An x/y/z triple keyed by axis name (unsigned, used for dimensions)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CorticalXyzU32 {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

/// Allowed IO coding options for an IPU/OPU cortical unit
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CorticalIOCodingOptions {
    pub signage_options: Vec<String>,
    pub behavior_options: Vec<String>,
    pub coding_type_options: Vec<String>,
}

/// Full properties of one cortical area, as serialized by the old service layer
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CorticalAreaInfo {
    /// Base64 cortical area ID
    pub cortical_id: Base64EncodedCorticalID,
    /// Human-readable ASCII form of the ID (e.g. "___power")
    pub cortical_id_s: String, // TODO Nuke This?
    /// Internal numerical index
    pub cortical_idx: u32,
    pub cortical_name: String,
    pub cortical_dimensions: [u32; 3],
    pub coordinates_3d: [i32; 3],
    /// "Sensory", "Motor", "Memory", "Custom"
    pub area_type: String,
    /// "IPU", "OPU", "CORE", "CUSTOM", "MEMORY"
    pub cortical_group: String,
    /// "sensory", "motor", "memory", "custom", "core"
    pub cortical_type: String,
    pub neuron_count: u64,
    pub synapse_count: u64,
    pub incoming_synapse_count: u64,
    pub outgoing_synapse_count: u64,
    pub visible: bool,
    pub cortical_sub_group: Option<String>,
    pub neurons_per_voxel: u32,
    pub neuron_post_synaptic_potential: f64,
    pub neuron_post_synaptic_potential_max: f64,
    pub neuron_plasticity_constant: f64,
    pub neuron_degeneracy_coefficient: f64,
    pub neuron_psp_uniform_distribution: bool,
    pub neuron_mp_driven_psp: bool,
    pub neuron_fire_threshold: f64,
    pub neuron_fire_threshold_increment: [f64; 3],
    pub neuron_firing_threshold_limit: f64,
    pub neuron_consecutive_fire_count: u32,
    pub neuron_snooze_period: u32,
    pub neuron_refractory_period: u32,
    pub neuron_leak_coefficient: f64,
    pub neuron_leak_variability: f64,
    pub neuron_mp_charge_accumulation: bool,
    pub neuron_excitability: f64,
    pub neuron_burst_engine_active: bool,
    pub neuron_init_lifespan: u32,
    pub neuron_lifespan_growth_rate: f64,
    pub neuron_longterm_mem_threshold: u32,
    /// Memory areas only
    pub temporal_depth: Option<u32>,
    /// Memory areas only
    pub mp_learning_enabled: Option<bool>,
    /// Free-form extra properties (cortical_mapping_dst, coordinate_2d, dev_count, ...)
    pub properties: HashMap<String, serde_json::Value>,
    /// IPU/OPU only: 4-character subtype (e.g. "isvi")
    pub cortical_subtype: Option<String>,
    /// IPU/OPU only: "Absolute" or "Incremental"
    pub encoding_type: Option<String>,
    /// IPU/OPU only: "Linear" or "Fractional"
    pub encoding_format: Option<String>,
    /// IPU/OPU only: cortical unit index (byte 7 of the ID)
    pub unit_id: Option<u8>,
    /// IPU/OPU only: cortical sub-unit index (byte 6 of the ID)
    pub subunit_id: Option<u8>,
    /// Legacy alias of unit_id
    pub group_id: Option<u8>,
    pub coding_signage: Option<String>,
    pub coding_behavior: Option<String>,
    pub coding_type: Option<String>,
    pub coding_options: Option<CorticalIOCodingOptions>,
    pub parent_region_id: Option<String>,
    /// IPU/OPU only: number of devices/channels
    pub dev_count: Option<u32>,
    /// IPU/OPU only: dimensions of a single device/channel
    pub cortical_dimensions_per_device: Option<[u32; 3]>,
    /// Voxel chunk size used by visualizers for very large areas
    pub visualization_voxel_granularity: Option<[u32; 3]>,
}

/// Geometry and neural parameters of one cortical area, in the shape the Brain Visualizer expects
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CorticalAreaGeometry {
    pub cortical_id: Base64EncodedCorticalID,
    pub cortical_name: String,
    pub cortical_group: String,
    pub cortical_type: String,
    /// Empty string when there is no sub group
    pub cortical_sub_group: String,
    pub coordinates_3d: [i32; 3],
    pub coordinates_2d: [i32; 2],
    pub cortical_dimensions: [u32; 3],
    pub cortical_neuron_per_vox_count: u32,
    pub visualization: bool,
    pub visible: bool,
    /// Same as cortical_dimensions, keyed by axis (backward compatibility)
    pub dimensions: CorticalXyzU32,
    /// Same as coordinates_3d, keyed by axis (backward compatibility)
    pub position: CorticalXyzI32,
    pub neuron_post_synaptic_potential: f64,
    pub neuron_fire_threshold: f64,
    pub neuron_firing_threshold_limit: f64,
    pub plasticity_constant: f64,
    pub degeneration: f64,
    pub leak_coefficient: f64,
    pub refractory_period: u32,
    pub snooze_period: u32,
    pub parent_region_id: Option<String>,
    pub visualization_voxel_granularity: Option<[u32; 3]>,
}

/// Default placement and size of one sub-unit of an IPU/OPU cortical unit
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CorticalUnitTopology {
    pub relative_position: [i32; 3],
    pub dimensions: [u32; 3],
}

/// Metadata describing one IPU or OPU cortical type
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CorticalTypeMetadata {
    /// Friendly name of the cortical unit
    pub description: String,
    /// Supported encodings, e.g. "absolute", "incremental"
    pub encodings: Vec<String>,
    /// Supported formats, e.g. "linear", "fractional" (empty when not applicable)
    pub formats: Vec<String>,
    /// Number of cortical areas the unit is made of
    pub units: u32,
    /// Default resolution [x, y, z]
    pub resolution: Vec<i32>,
    /// e.g. "asymmetric"
    pub structure: String,
    /// Default topology keyed by sub-unit index (stringified in JSON)
    pub unit_default_topology: HashMap<u32, CorticalUnitTopology>,
}

/// Genome memory parameters of a memory cortical area
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MemoryCorticalAreaParameters {
    pub temporal_depth: u32,
    pub longterm_mem_threshold: u32,
    pub lifespan_growth_rate: f32,
    pub init_lifespan: u32,
    pub mp_learning_enabled: bool,
}

/// Runtime state reset outcome for one cortical area
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CorticalAreaResetItem {
    pub cortical_idx: u32,
    pub neurons_reset: u64,
}

define_request_message_category! {
    category_name: "CorticalArea",
    base_path: "v1/cortical_area",
    category_description: "Cortical area management: listing, properties, creation, cloning, updates, runtime resets and deletion, plus IPU/OPU type metadata and voxel/memory introspection",
    read: {
        "Ipu": {
            path: "ipu",
            description: "List the IDs of all IPU (sensory) cortical areas",
            alloc_only: true,
            response: {
                cortical_ids: Vec<Base64EncodedCorticalID>, "IDs of all IPU cortical areas",
            },
        },
        "Opu": {
            path: "opu",
            description: "List the IDs of all OPU (motor) cortical areas",
            alloc_only: true,
            response: {
                cortical_ids: Vec<Base64EncodedCorticalID>, "IDs of all OPU cortical areas",
            },
        },
        "IpuTypes": {
            path: "ipu/types",
            description: "Get metadata (encodings, formats, units, topology) for every available IPU type",
            alloc_only: true,
            response: {
                types: HashMap<String, CorticalTypeMetadata>, "Metadata keyed by IPU type key (e.g. 'isvi')",
            },
        },
        "OpuTypes": {
            path: "opu/types",
            description: "Get metadata (encodings, formats, units, topology) for every available OPU type",
            alloc_only: true,
            response: {
                types: HashMap<String, CorticalTypeMetadata>, "Metadata keyed by OPU type key (e.g. 'omot')",
            },
        },
        "CorticalAreaIdList": {
            path: "cortical_area_id_list",
            description: "List the IDs of every cortical area in the genome (IPU, OPU, custom, memory and core)",
            alloc_only: true,
            response: {
                cortical_ids: Vec<Base64EncodedCorticalID>, "All cortical area IDs",
            },
        },
        "CorticalAreaNameList": {
            path: "cortical_area_name_list",
            description: "List the human-readable names of every cortical area",
            alloc_only: true,
            response: {
                cortical_area_name_list: Vec<String>, "All cortical area names",
            },
        },
        "CorticalIdNameMapping": {
            path: "cortical_id_name_mapping",
            description: "Get a map of cortical area IDs to their names",
            alloc_only: true,
            response: {
                mapping: HashMap<Base64EncodedCorticalID, String>, "Cortical area ID to name",
            },
        },
        "CorticalTypes": {
            path: "cortical_types",
            description: "Get the available cortical area types (sensory, motor, memory, custom)",
            alloc_only: true,
            response: {
                cortical_types: Vec<String>, "Available cortical area types",
            },
        },
        "CorticalMapDetailed": {
            path: "cortical_map_detailed",
            description: "Get detailed source-to-destination connectivity mappings with mapping rules, for every area that has outgoing mappings",
            alloc_only: true,
            response: {
                mappings: HashMap<Base64EncodedCorticalID, serde_json::Value>, "Source area ID to its cortical_mapping_dst object (destination ID to list of mapping rules)",
            },
        },
        "CorticalLocations2d": {
            path: "cortical_locations_2d",
            description: "Get the 2D visualization position of every cortical area",
            alloc_only: true,
            response: {
                locations: HashMap<Base64EncodedCorticalID, (i32, i32)>, "Cortical area ID to (x, y)",
            },
        },
        "CorticalAreaGeometry": {
            path: "cortical_area/geometry",
            description: "Get full geometry, neural parameters and metadata for every cortical area (used by the Brain Visualizer when loading a genome)",
            alloc_only: true,
            response: {
                areas: HashMap<Base64EncodedCorticalID, CorticalAreaGeometry>, "Cortical area ID to its geometry",
            },
        },
        "CorticalVisibility": {
            path: "cortical_visibility",
            description: "Get the visibility flag of every cortical area",
            alloc_only: true,
            response: {
                visibility: HashMap<Base64EncodedCorticalID, bool>, "Cortical area ID to visibility flag",
            },
        },
        "VoxelNeurons": { // cannot be implemented
            path: "voxel_neurons",
            description: "List every neuron at a voxel of a cortical area, with live properties and paginated synapse details (at most 50 outgoing and 50 incoming per page)",
            alloc_only: true,
            payload: {
                cortical_id: Base64EncodedCorticalID, "Base64 cortical area ID",
                x: u32, "Voxel X within the area",
                y: u32, "Voxel Y within the area",
                z: u32, "Voxel Z within the area",
                synapse_page: Option<u32>, "0-based synapse detail page (default 0)",
            },
            response: {
                cortical_id: Base64EncodedCorticalID, "Base64 cortical area ID",
                cortical_name: String, "Name of the cortical area",
                cortical_idx: u32, "Internal index of the cortical area",
                voxel_coordinate: [u32; 3], "The queried voxel",
                x: u32,
                y: u32,
                z: u32,
                synapse_page: u32, "Echo of the requested synapse page",
                neuron_count: u64, "Number of neurons at the voxel",
                neurons: Vec<serde_json::Value>, "Per-neuron property snapshots with outgoing_synapses / incoming_synapses",
            },
            error: {
                CorticalAreaNotFound: "No cortical area has that ID",
                IntrospectionUnavailable: "The engine cannot inspect neurons per voxel",
            },
        },
        "Memory": { // cannot be implemented as is
            path: "memory",
            description: "Get runtime stats, genome memory parameters, upstream wiring, synapse counts and paginated memory neuron IDs of a memory cortical area",
            alloc_only: true,
            payload: {
                cortical_id: Base64EncodedCorticalID, "Base64 ID of a memory cortical area",
                page: Option<u32>, "0-based page of memory_neuron_ids (default 0)",
                page_size: Option<u32>, "Page size for memory_neuron_ids (default 50, max 500)",
            },
            response: {
                cortical_id: Base64EncodedCorticalID,
                cortical_idx: u32,
                cortical_name: String,
                short_term_neuron_count: u64,
                long_term_neuron_count: u64,
                memory_parameters: MemoryCorticalAreaParameters, "Genome memory parameters",
                upstream_cortical_area_indices: Vec<u32>, "Indices of areas feeding pattern detection",
                upstream_cortical_area_count: u64,
                upstream_pattern_cache_size: u64, "Distinct temporal patterns cached for this area",
                incoming_synapse_count: u64,
                outgoing_synapse_count: u64,
                total_memory_neuron_ids: u64,
                page: u32,
                page_size: u32,
                memory_neuron_ids: Vec<u64>, "This page of memory neuron IDs",
                has_more: bool, "True when further pages exist",
            },
            error: {
                CorticalAreaNotFound: "No cortical area has that ID",
                NotAMemoryArea: "The cortical area is not a memory area",
                InvalidCorticalId: "The cortical ID is not valid base64",
                IntrospectionUnavailable: "The engine cannot provide memory runtime detail",
            },
        },
    },
    create: {
        "CorticalNameLocation": {
            path: "cortical_name_location",
            description: "Get the 2D location of a cortical area by name",
            alloc_only: true,
            payload: {
                cortical_name: String, "Name of the cortical area",
            },
            response: {
                locations: HashMap<Base64EncodedCorticalID, (i32, i32)>, "Single entry: cortical area ID to (x, y)",
            },
        },
        "CorticalAreaProperties": {
            path: "cortical_area_properties",
            description: "Get the full properties of a single cortical area",
            alloc_only: true,
            payload: {
                cortical_id: Base64EncodedCorticalID, "Base64 cortical area ID",
            },
            response: {
                properties: CorticalAreaInfo, "Full properties of the area",
            },
        },
        "MultiCorticalAreaProperties": {
            path: "multi/cortical_area_properties",
            description: "Get the full properties of several cortical areas (unknown IDs are skipped)",
            alloc_only: true,
            payload: {
                cortical_id_list: Vec<Base64EncodedCorticalID>, "Base64 cortical area IDs",
            },
            response: {
                areas: HashMap<Base64EncodedCorticalID, CorticalAreaInfo>, "Cortical area ID to its properties",
            },
        },
        "CorticalArea": {
            path: "cortical_area",
            description: "Create the cortical areas of an IPU (sensory) or OPU (motor) cortical unit, one per sub-unit, with the unit's default topology",
            alloc_only: true,
            payload: {
                cortical_id: Base64EncodedCorticalID, "IPU/OPU type key, e.g. 'isvi' or 'omot'",
                cortical_type: String, "'IPU' or 'OPU'",
                device_count: u32, "Number of devices; scales X of each sub-unit",
                coordinates_3d: [i32; 3], "Base 3D position of the unit",
                data_type_configs_by_subunit: HashMap<String, u16>, "Data type config per sub-unit index (keys are integers as strings)",
                group_id: Option<u8>, "Unit/group index (byte 7 of the ID), default 0",
                unit_id: Option<u8>, "Overrides group_id when set",
                neurons_per_voxel: Option<u32>, "Neurons per voxel, default 1",
                per_device_dimensions: Option<[u32; 3]>, "Override per-device dimensions [x, y, z]",
            },
            response: {
                message: String,
                cortical_id: Base64EncodedCorticalID, "ID of the first created area (backward compatibility)",
                cortical_ids: String, "Comma-separated IDs of all created areas",
                unit_count: u32, "Number of areas created",
                areas: Vec<CorticalAreaInfo>, "Full details of every created area",
            },
            error: {
                InvalidInput: "A required field is missing or malformed, or the IPU/OPU type is unknown",
            },
        },
        "CustomCorticalArea": {
            path: "custom_cortical_area",
            description: "Create a custom (or memory, when sub_group_id is MEMORY) cortical area inside a brain region",
            alloc_only: true,
            payload: {
                cortical_name: String, "Name of the new area",
                cortical_dimensions: [u32; 3], "Dimensions [x, y, z]",
                coordinates_3d: [i32; 3], "Position [x, y, z]",
                brain_region_id: String, "Brain region the area belongs to (required)",
                cortical_sub_group: Option<String>, "Optional sub group name",
                sub_group_id: Option<String>, "'MEMORY' to create a memory area instead of a custom one",
                cortical_group: Option<String>, "Sent by the Brain Visualizer (e.g. 'CUSTOM'), unused",
            },
            response: {
                message: String,
                cortical_id: Base64EncodedCorticalID, "Generated base64 ID of the new area",
                cortical_name: String,
            },
            error: {
                InvalidInput: "A required field is missing or malformed",
            },
        },
        "Clone": {
            path: "clone",
            description: "Clone a custom or memory cortical area, optionally reproducing its incoming and outgoing mappings",
            alloc_only: true,
            payload: {
                source_area_id: Base64EncodedCorticalID, "Base64 ID of the area to clone",
                new_name: String, "Name of the clone",
                coordinates_3d: [i32; 3], "3D position of the clone",
                coordinates_2d: [i32; 2], "2D visualization position of the clone",
                parent_region_id: Option<String>, "If set, must match the source area's region",
                clone_cortical_mapping: bool, "Also clone incoming and outgoing mappings",
            },
            response: {
                message: String,
                new_area_id: String, "Base64 ID of the clone",
            },
            error: {
                CorticalAreaNotFound: "No cortical area has the source ID",
                InvalidInput: "Source is not a custom/memory area, the ID is invalid, or parent_region_id mismatches",
            },
        },
        "VoxelNeurons": { // cannot be implemented as is
            path: "voxel_neurons",
            description: "Same as reading voxel_neurons, but with a JSON body",
            alloc_only: true,
            payload: {
                cortical_id: String, "Base64 cortical area ID",
                x: u32,
                y: u32,
                z: u32,
                synapse_page: Option<u32>, "0-based synapse detail page (default 0)",
            },
            response: {
                cortical_id: Base64EncodedCorticalID,
                cortical_name: String,
                cortical_idx: u32,
                voxel_coordinate: [u32; 3],
                x: u32,
                y: u32,
                z: u32,
                synapse_page: u32,
                neuron_count: u64,
                neurons: Vec<serde_json::Value>, "Per-neuron property snapshots with outgoing_synapses / incoming_synapses",
            },
            error: {
                CorticalAreaNotFound: "No cortical area has that ID",
                IntrospectionUnavailable: "The engine cannot inspect neurons per voxel",
            },
        },
        "Resize": {
            path: "resize",
            description: "Resize a cortical area by changing its dimensions (not yet implemented)",
            alloc_only: true,
            response: {
                message: String,
            },
        },
    },
    edit: {
        "CorticalArea": {
            path: "cortical_area",
            description: "Update properties of a cortical area (position, dimensions, neural parameters, ...); the area ID may change",
            alloc_only: true,
            payload: {
                cortical_id: Base64EncodedCorticalID, "Base64 ID of the area to update",
                changes: HashMap<String, serde_json::Value>, "Property name to new value",
            },
            response: {
                message: String,
                cortical_id: Base64EncodedCorticalID, "ID after the update",
                previous_cortical_id: String, "ID before the update",
            },
        },
        "MultiCorticalArea": {
            path: "multi/cortical_area",
            description: "Update several cortical areas at once: shared changes apply to all, per-area changes override them",
            alloc_only: true,
            payload: {
                cortical_id_list: Vec<Base64EncodedCorticalID>, "IDs of the areas to update (non-empty)",
                shared_changes: HashMap<String, serde_json::Value>, "Changes applied to every listed area",
                per_area_changes: Option<HashMap<String, HashMap<String, serde_json::Value>>>, "Area ID to changes overriding the shared ones",
            },
            response: {
                message: String,
                cortical_ids: String, "Comma-separated IDs of the updated areas",
            },
        },
        "Reset": {
            path: "reset",
            description: "Reset runtime neural state (membrane potentials, refractory counters, FCL candidates) of cortical areas; genome and connections are unchanged",
            alloc_only: true,
            payload: {
                area_list: Vec<String>, "IDs of the areas to reset (non-empty)",
            },
            response: {
                message: String,
                results: Vec<CorticalAreaResetItem>, "Neurons reset per area",
            },
            error: {
                CorticalAreaNotFound: "No cortical area has one of the IDs",
                EmptyAreaList: "area_list is empty",
            },
        },
        "Coord2d": {
            path: "coord_2d",
            description: "Update the 2D visualization coordinates of a cortical area (not yet implemented)",
            error: {
                NotImplemented: "This endpoint is not implemented",
            },
        },
        "SuppressCorticalVisibility": {
            path: "suppress_cortical_visibility",
            description: "Hide or show cortical areas in visualizations (not yet implemented)",
            error: {
                NotImplemented: "This endpoint is not implemented",
            },
        },
        "Coordinates3d": {
            path: "coordinates_3d",
            description: "Update the 3D position of a cortical area (not yet implemented)",
            alloc_only: true,
            response: {
                message: String,
            },
        },
        "Coord3d": {
            path: "coord_3d",
            description: "Update the 3D coordinates of a cortical area, alternative endpoint (not yet implemented)",
            alloc_only: true,
            response: {
                message: String,
            },
        },
    },
    delete: {
        "CorticalArea": {
            path: "cortical_area",
            description: "Delete a cortical area with all its neurons and synapses",
            alloc_only: true,
            payload: {
                cortical_id: Base64EncodedCorticalID, "Base64 ID of the area to delete",
            },
            response: {
                message: String,
            },
        },
        "MultiCorticalArea": {
            path: "multi/cortical_area",
            description: "Delete several cortical areas",
            alloc_only: true,
            payload: {
                cortical_ids: Vec<Base64EncodedCorticalID>, "IDs of the areas to delete (non-empty)",
            },
            response: {
                message: String,
                cortical_ids: String, "Comma-separated IDs of the deleted areas",
            },
        },
    },
}

// NOTES:
// - Bare array/map/scalar responses wrapped in one field: ipu, opu, ipu/opu types, id/name mappings, cortical_types, map_detailed, locations_2d, geometry, visibility, index_list, idx_mapping, neuron_count, cortical_name_location, (multi) properties, cortical_type_options.
// - Several old POSTs are really queries (cortical_name_location, cortical_area_properties, multi properties, voxel_neurons, cortical_type_options); kept under create to match the old methods.
// - multi/cortical_area_properties also accepted a bare ID array; only the {cortical_id_list} form is kept.
// - CorticalAreaInfo mirrors feagi-services' serialized CorticalAreaInfo; usize fields mapped to u32/u64.
// - cortical_map_detailed values and voxel neuron snapshots stay serde_json::Value (shape not fixed in old code).
// - geometry coordinates_2d came from free-form properties (default [0, 0]); typed as [i32; 2] by guess.
// - PUT cortical_area took a flat body of arbitrary property keys; moved into `changes` map (old was flat).
// - PUT multi/cortical_area mixed shared keys and per-ID objects in one flat body; split into shared_changes / per_area_changes.
// - DELETE multi/cortical_area took a bare ID array; wrapped as cortical_ids.
// - POST cortical_area: `cortical_id` is actually the IPU/OPU type key, not an area ID.
// - GET ipu/list: alias of GET ipu.
// - GET opu/list: alias of GET opu.
// - GET mapping_restrictions: always returned an empty map; old untyped query HashMap<String, String> omitted.
// - POST mapping_restrictions: always returned an empty map; old untyped ignored body HashMap<String, String> omitted.
// - POST mapping_restrictions_between_areas: always returned an empty map; old untyped ignored body omitted.
// - PUT coord_2d: stub, always returned a "Not yet implemented" error; untyped ignored body omitted.
// - PUT suppress_cortical_visibility: stub, always returned a "Not yet implemented" error; untyped ignored body omitted.
// - PUT coordinates_3d: stub, returned {message: "Not yet implemented"}; untyped ignored body omitted.
// - PUT coord_3d: stub, returned {message: "Not yet implemented"}; untyped ignored body omitted.
// - POST resize: stub, returned {message: "Not yet implemented"}; untyped ignored body omitted.
// - POST reposition: stub, returned {message: "Not yet implemented"}; untyped ignored body omitted.
// - POST batch_operations: stub, always returned processed = 0; bare array body wrapped as `operations`, items untyped.
// - DELETE bulk_delete: stub, always returned deleted_count = 0; bare ID array body wrapped as `cortical_ids`.
// - Errors other than the listed ones were generic internal errors in the old code.
