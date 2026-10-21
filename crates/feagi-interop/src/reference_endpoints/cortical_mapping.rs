//! Cortical mappings: the synaptic connection rules between a source and a destination cortical
//! area. Converted from the old `cortical_mapping` endpoints.

use feagi_basis::feagi_basis_proc_common::define_request_message_category;
use std::collections::HashMap;

/// One connection rule of a mapping from a source to a destination cortical area
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MappingRule {
    /// ID of the morphology defining the connection pattern
    pub morphology_id: String,
    /// Scale applied to the morphology, per axis
    pub morphology_scalar: [i32; 3],
    /// Multiplier on the post synaptic current
    #[serde(rename = "postSynapticCurrent_multiplier")]
    pub post_synaptic_current_multiplier: i64,
    /// Whether plasticity is enabled (superseded by `plasticity_mode`)
    pub plasticity_flag: bool,
    /// Plasticity constant
    pub plasticity_constant: i64,
    /// Long term potentiation multiplier
    pub ltp_multiplier: i8,
    /// Long term depression multiplier
    pub ltd_multiplier: i8,
    /// Plasticity window
    pub plasticity_window: i64,
    /// Synaptic delay in bursts, at least 1 (defaults to 1)
    #[serde(default = "default_synaptic_delay_bursts")]
    pub synaptic_delay_bursts: u64,
    /// Plasticity mode (e.g. "rstdp"); when absent, derived from `plasticity_flag`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plasticity_mode: Option<String>,
    /// R-STDP eligibility trace decay in bursts
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eligibility_decay_bursts: Option<u64>,
    /// R-STDP reward source cortical area ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reward_source_area: Option<String>,
    /// R-STDP punishment source cortical area ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub punishment_source_area: Option<String>,
    /// STDP / R-STDP weight ceiling
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_weight: Option<f64>,
    /// Plasticity learning rate
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plasticity_eta: Option<f64>,
}

fn default_synaptic_delay_bursts() -> u64 {
    1
}

define_request_message_category! {
    category_name: "CorticalMapping",
    base_path: "v1/cortical_mapping",
    category_description: "Synaptic connection rules between a source and a destination cortical area",
    read: {
        "Mapping": {
            path: "mapping",
            description: "Get the raw stored connection rules from one cortical area to another",
            alloc_only: true,
            payload: {
                src_cortical_area: String, "Source cortical area ID",
                dst_cortical_area: String, "Destination cortical area ID",
            },
            response: {
                connections: Vec<serde_json::Value>, "Stored rules, in array or object form",
            },
            error: {
                CorticalAreaNotFound: "The source cortical area does not exist",
            },
        },
        "MappingList": {
            path: "mapping_list",
            description: "Get every mapped [source, destination] cortical area pair",
            alloc_only: true,
            response: {
                mappings: Vec<[String; 2]>, "[source area ID, destination area ID] pairs",
            },
        },
    },
    create: {
        "MappingProperties": {
            path: "mapping_properties",
            description: "Get the normalized connection rules from one cortical area to another",
            alloc_only: true,
            payload: {
                src_cortical_area: String, "Source cortical area ID",
                dst_cortical_area: String, "Destination cortical area ID",
            },
            response: {
                rules: Vec<MappingRule>, "Connection rules; empty if the areas are not mapped",
            },
            error: {
                CorticalAreaNotFound: "The source cortical area does not exist",
                InvalidStoredRule: "A stored rule is malformed",
            },
        },
        "Afferents": {
            path: "afferents",
            description: "Get the cortical areas mapping into a cortical area",
            alloc_only: true,
            response: {
                cortical_areas: Vec<String>, "Cortical area IDs",
            },
        },
        "Efferents": {
            path: "efferents",
            description: "Get the cortical areas a cortical area maps to",
            alloc_only: true,
            response: {
                cortical_areas: Vec<String>, "Cortical area IDs",
            },
        },
        "BatchUpdate": {
            path: "batch_update",
            description: "Batch update multiple cortical mappings",
            alloc_only: true,
            payload: {
                updates: Vec<HashMap<String, String>>, "One entry per mapping update",
            },
            response: {
                message: String,
                updated_count: u64, "Number of mappings updated",
            },
        },
        "Mapping": {
            path: "mapping",
            description: "Create a cortical mapping",
            alloc_only: true,
            response: {
                message: String,
            },
        },
    },
    edit: {
        "MappingProperties": {
            path: "mapping_properties",
            description: "Replace the connection rules from one cortical area to another and regenerate synapses",
            alloc_only: true,
            payload: {
                src_cortical_area: String, "Source cortical area ID",
                dst_cortical_area: String, "Destination cortical area ID",
                mapping_string: Vec<MappingRule>, "New connection rules; empty removes the mapping",
            },
            response: {
                message: String,
                synapse_count: u64, "Number of synapses created",
                src_region: Option<String>, "Always None for now",
                dst_region: Option<String>, "Always None for now",
            },
            error: {
                CorticalAreaNotFound: "A cortical area does not exist",
                InvalidMapping: "The mapping rules are invalid",
                MappingConflict: "The mapping conflicts with the current connectome",
            },
        },
        "Mapping": {
            path: "mapping",
            description: "Update a cortical mapping",
            alloc_only: true,
            response: {
                message: String,
            },
        },
    },
    delete: {
        "Mapping": {
            path: "mapping",
            description: "Delete the mapping from one cortical area to another, pruning its synapses",
            alloc_only: true,
            payload: {
                src_cortical_area: String, "Source cortical area ID",
                dst_cortical_area: String, "Destination cortical area ID",
            },
            response: {
                message: String,
                src_cortical_area: String,
                dst_cortical_area: String,
                synapse_count: u64, "Synapses remaining between the areas",
            },
            error: {
                CorticalAreaNotFound: "A cortical area does not exist",
                MappingConflict: "The deletion conflicts with the current connectome",
            },
        },
    },
}

// NOTES:
// MappingProperties (POST) is really a read; it lives in `create` because the old endpoint was POST.
// MappingProperties: old response was a bare array; wrapped in `rules`.
// MappingList: old response was a bare Vec<Vec<String>>; wrapped in `mappings` and typed as pairs.
// read Mapping: `connections` left untyped; stored rules may be arrays or objects. Returned {} (no field) if no mappings.
// MappingRule.morphology_scalar typed [i32; 3] (guessed; old code passed the JSON value through).
// edit MappingProperties: old code also accepted rules in positional array form; only object form is modelled.
// delete Mapping: src/dst were URL query params in the old DELETE; now payload fields.
// synapse_count typed u64 (old type was usize).
// POST afferents: unimplemented stub, payload unknown; old response was a bare Vec<String>, wrapped in `cortical_areas`.
// POST efferents: unimplemented stub, payload unknown; old response was a bare Vec<String>, wrapped in `cortical_areas`.
// POST batch_update: unimplemented stub; old payload was a bare array of string maps, wrapped in `updates`.
// POST mapping: unimplemented stub, payload unknown; response only had `message`.
// PUT mapping: unimplemented stub, payload unknown; response only had `message`.
