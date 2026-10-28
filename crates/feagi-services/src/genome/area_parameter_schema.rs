// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Machine-readable schema of every cortical-area parameter the update path accepts.

Single source of truth: the key universe is [`CorticalChangeClassifier`] (the update path routes
by those sets, and the update path treats any other key as an unknown structural change). Each key
there needs exactly one spec here, and the unit tests fail when a key is missing from either side.
`update_class` and `live_updatable` are *derived* from the classifier at build time, never stated
twice. Hard bounds (`min`/`max`/`enum_values`) are the ones core validation enforces and are
checked by [`validate_hard_bounds`], which the update path calls; `suggested_min`/`suggested_max`
are advisory ranges for editors (they come from the Brain Visualizer editors) that core does not
enforce.
*/

use super::change_classifier::CorticalChangeClassifier;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Excitability is a fraction of bursts, not a percent. Shared by schema and validation.
pub const EXCITABILITY_MIN: f64 = 0.0;
pub const EXCITABILITY_MAX: f64 = 1.0;

pub use feagi_evolutionary::validator::{
    CONSECUTIVE_FIRE_COUNT_MAX, LEAK_FRACTION_MAX, LEAK_FRACTION_MIN,
};

const ALL_AREAS: &[&str] = &["IPU", "OPU", "CORE", "CUSTOM", "MEMORY"];
const MEMORY_ONLY: &[&str] = &["MEMORY"];
const IO_ONLY: &[&str] = &["IPU", "OPU"];

/// One parameter (or one field of an object parameter).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AreaParameterSchemaEntry {
    /// Canonical key to send to `update_cortical_area`.
    pub key: String,
    /// Other keys the update path accepts for the same parameter.
    pub aliases: Vec<String>,
    pub label: String,
    pub description: String,
    /// number | integer | boolean | enum | object | array | string
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// Hard lower bound enforced by core validation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// When true the lower bound itself is rejected (value must be > min).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub min_exclusive: bool,
    /// Hard upper bound enforced by core validation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// Advisory range used by editors; not enforced by core.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    /// Object parameters: their fields (flat, same entry shape).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<AreaParameterSchemaEntry>>,
    /// Editor section, in Brain Visualizer's Advanced Cortical Properties order: Summary,
    /// Neural Coding, Memory Tunables, Neuron Firing Parameters, Post Synaptic Potential
    /// Parameters, Cortical Area Monitoring, Connections, Advanced.
    pub category: String,
    /// Area groups the parameter applies to (IPU, OPU, CORE, CUSTOM, MEMORY).
    pub applies_to: Vec<String>,
    /// True when the change is applied without a synapse rebuild (parameter or metadata class).
    pub live_updatable: bool,
    /// parameter | metadata | structural | special (from the change classifier).
    pub update_class: String,
    /// True when `null` is accepted (clears the setting).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub nullable: bool,
    /// Value on the area when the schema was built for one. Present for every parameter then:
    /// the stored or effective value, else the schema default, else JSON `null`, which means
    /// the area has no setting for it (it is optional, memory-only data, or unset). Absent only
    /// when the schema was built without an area.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_value: Option<Value>,
}

impl AreaParameterSchemaEntry {
    fn new(key: &str, label: &str, value_type: &str, description: &str) -> Self {
        Self {
            key: key.to_string(),
            aliases: Vec::new(),
            label: label.to_string(),
            description: description.to_string(),
            value_type: value_type.to_string(),
            unit: None,
            min: None,
            min_exclusive: false,
            max: None,
            suggested_min: None,
            suggested_max: None,
            enum_values: None,
            default: None,
            fields: None,
            category: "Advanced".to_string(),
            applies_to: ALL_AREAS.iter().map(|s| s.to_string()).collect(),
            live_updatable: false,
            update_class: String::new(),
            nullable: false,
            current_value: None,
        }
    }
    fn alias(mut self, aliases: &[&str]) -> Self {
        self.aliases = aliases.iter().map(|s| s.to_string()).collect();
        self
    }
    fn unit(mut self, unit: &str) -> Self {
        self.unit = Some(unit.to_string());
        self
    }
    fn hard(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    fn exclusive_min(mut self) -> Self {
        self.min_exclusive = true;
        self
    }
    fn suggest(mut self, min: f64, max: f64) -> Self {
        self.suggested_min = Some(min);
        self.suggested_max = Some(max);
        self
    }
    fn values(mut self, values: &[&str]) -> Self {
        self.enum_values = Some(values.iter().map(|s| s.to_string()).collect());
        self
    }
    fn default_value(mut self, v: Value) -> Self {
        self.default = Some(v);
        self
    }
    fn fields(mut self, fields: Vec<AreaParameterSchemaEntry>) -> Self {
        self.fields = Some(fields);
        self
    }
    fn only(mut self, areas: &[&str]) -> Self {
        self.applies_to = areas.iter().map(|s| s.to_string()).collect();
        self
    }
    fn nullable(mut self) -> Self {
        self.nullable = true;
        self
    }

    /// All keys (canonical and aliases) this entry answers to.
    pub fn all_keys(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.key.as_str()).chain(self.aliases.iter().map(|s| s.as_str()))
    }
}

/// Fields of `rate_modulated_leak`. Bounds mirror `RateModulatedLeakConfig::validate`
/// (cross-checked by a unit test); `leak_min <= leak_max` is also enforced.
fn rate_modulated_leak_fields() -> Vec<AreaParameterSchemaEntry> {
    use AreaParameterSchemaEntry as E;
    vec![
        E::new("enabled", "Enabled", "boolean", "Turns the homeostatic leak on. When off, the other fields are ignored.")
            .default_value(json!(false)),
        E::new("target_mode", "Target mode", "enum", "Where each neuron's firing-rate set-point comes from: one value for the whole area, or each neuron's own rate measured during an arming phase.")
            .values(&["area", "per_neuron"])
            .default_value(json!("area")),
        E::new("target_firing_per_burst", "Target firing per burst", "number", "Area mode: expected fraction of bursts in which a neuron fires.")
            .unit("fraction of bursts").hard(Some(0.0), Some(1.0)).default_value(json!(0.1)),
        E::new("rate_ema_tau_bursts", "Rate EMA time constant", "number", "Time constant of the firing-rate moving average.")
            .unit("bursts").hard(Some(0.0), None).exclusive_min().suggest(1.0, 1_000_000.0).default_value(json!(50.0)),
        E::new("gain", "Gain", "number", "How strongly the leak follows the rate error.")
            .hard(Some(0.0), None).suggest(0.0, 10.0).default_value(json!(0.2)),
        E::new("leak_min", "Leak minimum", "number", "Lowest leak the homeostat may set. Must be <= leak_max.")
            .unit("fraction per burst").hard(Some(0.0), Some(1.0)).default_value(json!(0.02)),
        E::new("leak_max", "Leak maximum", "number", "Highest leak the homeostat may set. Must be >= leak_min.")
            .unit("fraction per burst").hard(Some(0.0), Some(1.0)).default_value(json!(0.5)),
        E::new("update_every_n_bursts", "Update interval", "integer", "The leak is adjusted every N bursts (0 is treated as 1).")
            .unit("bursts").suggest(1.0, 1_000_000.0).default_value(json!(1)),
        E::new("arm_bursts", "Arming length", "integer", "per_neuron only: bursts spent measuring each neuron's own rate before the targets freeze.")
            .unit("bursts").hard(Some(1.0), None).default_value(json!(500)),
        E::new("arm_tau_bursts", "Arming time constant", "number", "per_neuron only: time constant of the arming-phase rate average; absent means arm_bursts / 4.")
            .unit("bursts").hard(Some(0.0), None).exclusive_min().nullable(),
        E::new("min_target_firing_per_burst", "Minimum armed target", "number", "per_neuron only: neurons whose armed target is below this are treated as silent.")
            .unit("fraction of bursts").hard(Some(0.0), Some(1.0)).default_value(json!(0.001)),
        E::new("arm_epoch", "Arming epoch", "integer", "per_neuron only: changing this value restarts arming.")
            .default_value(json!(0)),
        E::new("silent_neurons", "Silent neurons", "enum", "per_neuron only: how neurons armed below the minimum target are handled. one_sided regulates them toward 0 with the leak floored at the base leak; unregulated leaves them at the base leak.")
            .values(&["one_sided", "unregulated"]).default_value(json!("one_sided")),
    ]
}

fn xyz_fields() -> Vec<AreaParameterSchemaEntry> {
    use AreaParameterSchemaEntry as E;
    ["x", "y", "z"]
        .iter()
        .map(|axis| {
            E::new(
                axis,
                &format!("{} gradient step", axis.to_uppercase()),
                "number",
                "Threshold increment per voxel along this axis.",
            )
        })
        .collect()
}

/// Editor sections, in display order. Anything not listed in [`category_for`] is "Advanced".
pub const CATEGORIES: [&str; 8] = [
    "Summary",
    "Neural Coding",
    "Memory Tunables",
    "Neuron Firing Parameters",
    "Post Synaptic Potential Parameters",
    "Cortical Area Monitoring",
    "Connections",
    "Advanced",
];

fn category_for(key: &str) -> &'static str {
    match key {
        "cortical_name"
        | "visible"
        | "coordinate_2d"
        | "coordinates_3d"
        | "cortical_dimensions"
        | "cortical_dimensions_per_device"
        | "per_voxel_neuron_cnt"
        | "region_id"
        | "cortical_type" => CATEGORIES[0],
        "coding_signage"
        | "coding_behavior"
        | "coding_type"
        | "new_cortical_id"
        | "group_id"
        | "sub_group_id"
        | "dev_count"
        | "sensory_hold_last_frame" => CATEGORIES[1],
        "neuron_init_lifespan"
        | "neuron_lifespan_growth_rate"
        | "neuron_longterm_mem_threshold"
        | "temporal_depth"
        | "mp_learning_enabled"
        | "mp_change_mode"
        | "mp_delta_quantization"
        | "mp_ratio_quantization"
        | "memory_neuron_cap"
        | "ltm_eviction"
        | "memory_twin_areas"
        | "is_mem_type" => CATEGORIES[2],
        "neuron_fire_threshold"
        | "neuron_firing_threshold_limit"
        | "neuron_fire_threshold_increment"
        | "firing_threshold_increment_x"
        | "firing_threshold_increment_y"
        | "firing_threshold_increment_z"
        | "neuron_refractory_period"
        | "neuron_leak_coefficient"
        | "neuron_leak_variability"
        | "neuron_consecutive_fire_count"
        | "neuron_snooze_period"
        | "neuron_excitability"
        | "neuron_mp_charge_accumulation"
        | "neuron_degeneracy_coefficient"
        | "burst_engine_active"
        | "rate_modulated_leak" => CATEGORIES[3],
        "neuron_post_synaptic_potential"
        | "neuron_post_synaptic_potential_max"
        | "neuron_psp_uniform_distribution"
        | "neuron_mp_driven_psp" => CATEGORIES[4],
        "visualization_voxel_granularity" | "visualization" => CATEGORIES[5],
        "cortical_mapping_dst" | "synapse_attractivity" => CATEGORIES[6],
        _ => CATEGORIES[7],
    }
}

/// Every parameter spec, before classifier-derived fields are filled in.
fn raw_specs() -> Vec<AreaParameterSchemaEntry> {
    use AreaParameterSchemaEntry as E;
    let mem = MEMORY_ONLY;
    vec![
        // ---- firing (parameter class) ----
        E::new("neuron_fire_threshold", "Fire threshold", "number", "Membrane potential a neuron must reach to fire.")
            .alias(&["firing_threshold"]).unit("membrane potential").suggest(0.0, 20.0),
        E::new("neuron_firing_threshold_limit", "Threshold limit", "number", "Upper cap on the firing threshold (0 means no cap).")
            .alias(&["firing_threshold_limit"]).unit("membrane potential").suggest(0.0, 100.0),
        E::new("neuron_fire_threshold_increment", "Threshold increment", "object", "Per-voxel gradient added to the firing threshold along x, y, z. Accepts {x,y,z} or [x,y,z].")
            .alias(&["firing_threshold_increment"]).fields(xyz_fields()),
        E::new("firing_threshold_increment_x", "Threshold increment X", "number", "Per-voxel firing-threshold gradient along x.").suggest(-20.0, 20.0),
        E::new("firing_threshold_increment_y", "Threshold increment Y", "number", "Per-voxel firing-threshold gradient along y.").suggest(-20.0, 20.0),
        E::new("firing_threshold_increment_z", "Threshold increment Z", "number", "Per-voxel firing-threshold gradient along z.").suggest(-20.0, 20.0),
        E::new("neuron_refractory_period", "Refractory period", "integer", "Bursts a neuron stays silent after it fires.")
            .alias(&["refractory_period", "refrac"]).unit("bursts").suggest(0.0, 64.0),
        E::new("neuron_leak_coefficient", "Leak", "number", "Fraction of the membrane potential removed each burst (0 to 1). Brain Visualizer shows it as a percent but sends the fraction.")
            .alias(&["leak_coefficient", "leak"]).unit("fraction per burst").hard(Some(LEAK_FRACTION_MIN), Some(LEAK_FRACTION_MAX)),
        E::new("neuron_leak_variability", "Leak variability", "number", "Random spread of the leak across neurons; changing it rebuilds neuron state.")
            .alias(&["leak_variability"]).unit("fraction").hard(Some(LEAK_FRACTION_MIN), Some(LEAK_FRACTION_MAX)),
        E::new("neuron_consecutive_fire_count", "Consecutive fire limit", "integer", "How many bursts in a row a neuron may fire before it is snoozed (0 means unlimited).")
            .alias(&["consecutive_fire_cnt_max", "consecutive_fire_count"]).unit("bursts").hard(Some(0.0), Some(CONSECUTIVE_FIRE_COUNT_MAX)).suggest(0.0, 64.0),
        E::new("neuron_snooze_period", "Snooze period", "integer", "Bursts a neuron waits after reaching the consecutive-fire limit.")
            .alias(&["snooze_length", "snooze_period"]).unit("bursts").suggest(0.0, 64.0),
        E::new("neuron_excitability", "Excitability", "number", "Fraction of bursts in which a neuron may fire (0 to 1, not percent). Out-of-range values are rejected.")
            .alias(&["excitability"]).unit("fraction").hard(Some(EXCITABILITY_MIN), Some(EXCITABILITY_MAX)),
        E::new("neuron_mp_charge_accumulation", "MP charge accumulation", "boolean", "Add incoming charge onto the current membrane potential instead of replacing it.")
            .alias(&["mp_charge_accumulation"]),
        E::new("burst_engine_active", "Burst engine active", "boolean", "Whether the burst engine processes this area."),
        E::new("rate_modulated_leak", "Rate-modulated leak", "object", "Opt-in homeostat that adjusts the leak to hold a firing-rate target. An object update merges into the stored one (given keys override, absent keys are kept, a null field deletes it); null clears the whole setting.")
            .fields(rate_modulated_leak_fields()).nullable().default_value(json!({"enabled": false})),
        // ---- postsynaptic ----
        E::new("neuron_post_synaptic_potential", "Postsynaptic potential", "number", "Membrane potential added to downstream neurons per spike.")
            .alias(&["postsynaptic_current"]).unit("membrane potential").suggest(-5.0, 5.0),
        E::new("neuron_post_synaptic_potential_max", "PSP max", "number", "Maximum postsynaptic potential.")
            .alias(&["postsynaptic_current_max"]).unit("membrane potential").suggest(0.0, 10000.0),
        E::new("neuron_psp_uniform_distribution", "PSP uniform distribution", "boolean", "Spread the postsynaptic potential uniformly over the target neurons.")
            .alias(&["psp_uniform_distribution"]),
        E::new("neuron_mp_driven_psp", "MP-driven PSP", "boolean", "Use the membrane potential as the postsynaptic potential.")
            .alias(&["mp_driven_psp"]),
        E::new("neuron_degeneracy_coefficient", "Degeneracy", "number", "Degeneracy coefficient of the area.")
            .alias(&["degeneration"]).suggest(0.0, 1.0),
        E::new("plasticity_constant", "Plasticity constant", "number", "Strength of synaptic plasticity on this area.")
            .alias(&["neuron_plasticity_constant"]),
        // ---- memory ----
        E::new("neuron_init_lifespan", "Initial lifespan", "integer", "Initial lifespan of a new memory neuron.")
            .alias(&["init_lifespan"]).unit("bursts").only(mem),
        E::new("neuron_lifespan_growth_rate", "Lifespan growth rate", "number", "How much a memory neuron's lifespan grows each time it is reactivated.")
            .alias(&["lifespan_growth_rate"]).only(mem),
        E::new("neuron_longterm_mem_threshold", "Long-term threshold", "integer", "Lifespan at which a memory neuron becomes long-term.")
            .alias(&["longterm_mem_threshold"]).only(mem),
        E::new("temporal_depth", "Temporal depth", "integer", "Number of bursts a memory pattern spans.").unit("bursts").only(mem),
        E::new("mp_learning_enabled", "MP learning", "boolean", "Learn from membrane-potential change. Mutually exclusive with a non-none mp_change_mode.").only(mem),
        E::new("mp_change_mode", "MP change mode", "enum", "How membrane-potential change is encoded for memory formation.")
            .values(&["none", "mp_differential", "mp_ratio"]).default_value(json!("none")).only(mem),
        E::new("mp_delta_quantization", "MP delta quantization", "number", "Quantization step of mp_differential encoding.")
            .hard(Some(0.0), None).exclusive_min().only(mem),
        E::new("mp_ratio_quantization", "MP ratio quantization", "number", "Quantization step of mp_ratio encoding.")
            .hard(Some(0.0), None).exclusive_min().only(mem),
        E::new("memory_neuron_cap", "Memory neuron cap", "integer", "Maximum memory neurons kept (0 means no cap).")
            .hard(Some(0.0), Some(u32::MAX as f64)).only(mem),
        E::new("ltm_eviction", "Long-term eviction", "enum", "Policy when the memory neuron cap is reached.")
            .values(&["lru", "none"]).only(mem),
        // ---- metadata ----
        E::new("cortical_name", "Name", "string", "Display name of the area.").alias(&["name"]),
        E::new("visible", "Visible", "boolean", "Whether the area is drawn in visualizers."),
        E::new("coordinate_2d", "2D position", "array", "Position on the 2D canvas, [x, y].").alias(&["coordinates_2d"]),
        E::new("coordinates_3d", "3D position", "array", "Position in 3D space, [x, y, z] integers.")
            .alias(&["coordinate_3d", "coordinates", "position"]),
        E::new("visualization_voxel_granularity", "Voxel granularity", "array", "Visualization-only voxel aggregation, [x, y, z] integers.").suggest(1.0, 1_000_000.0),
        E::new("memory_twin_areas", "Memory twin areas", "array", "Cortical ids of the memory areas that twin this area for classifier episodic scans."),
        E::new("modulators", "Modulators", "array", "Ids of area-level modulators (neuro.* and memory.formation) this area subscribes to; each must exist in the genome.").nullable(),
        E::new("spike_train", "Spike train", "boolean", "Mark the area as a spike-train source (limit comes with the modulator driver)."),
        E::new("sensory_hold_last_frame", "Hold last sensory frame", "boolean", "Keep re-injecting the last sensory frame between sensor updates.").only(IO_ONLY),
        E::new("coding_signage", "Coding signage", "string", "IO coding: signed or unsigned.").only(IO_ONLY),
        E::new("coding_behavior", "Coding behavior", "string", "IO coding: frame handling behavior.").only(IO_ONLY),
        E::new("coding_type", "Coding type", "string", "IO coding: value encoding type (changing it remaps the cortical id).").only(IO_ONLY),
        E::new("new_cortical_id", "New cortical id", "string", "Cortical id to remap the area to."),
        // ---- structural / special ----
        E::new("cortical_dimensions", "Dimensions", "array", "Area size [x, y, z]; changing it rebuilds neurons and synapses.").alias(&["dimensions"]),
        E::new("cortical_dimensions_per_device", "Dimensions per device", "array", "IO area size per device [x, y, z]; rebuilds neurons and synapses.").only(IO_ONLY),
        E::new("per_voxel_neuron_cnt", "Neurons per voxel", "integer", "Neurons per voxel; changing it rebuilds neurons and synapses.")
            .alias(&["cortical_neuron_per_vox_count", "neuron_density", "neurons_per_voxel"]).suggest(1.0, 1_000_000.0),
        E::new("cortical_type", "Cortical type", "string", "Area type; structural.").alias(&["area_type"]),
        E::new("cortical_mapping_dst", "Mapping destinations", "object", "Outgoing mappings keyed by destination id; prefer the mapping endpoints."),
        E::new("group_id", "Group id", "integer", "Unit index of an IO area; changing it remaps the cortical id."),
        E::new("sub_group_id", "Sub-group id", "integer", "Sub-unit index of an IO area.").only(IO_ONLY),
        E::new("region_id", "Brain region", "string", "Id of the brain region containing the area.").alias(&["brain_region_id", "parent_region_id"]),
        E::new("is_mem_type", "Is memory type", "boolean", "Marks the area as a memory area."),
        E::new("dev_count", "Device count", "integer", "Number of devices of an IO area.").only(IO_ONLY),
        E::new("synapse_attractivity", "Synapse attractivity", "number", "Attractivity of the area as a synapse target."),
        E::new("visualization", "Visualization flag", "boolean", "Whether the area takes part in visualization streams."),
        E::new("location_generation_type", "Location generation", "string", "How neuron locations are generated."),
    ]
}

/// Fill gaps in an area's stored properties with its effective values (`info`, the serialized
/// area info, which uses the schema's key names). Stored properties win; nothing is overwritten.
pub fn merge_effective_values(props: &mut HashMap<String, Value>, info: &Value) {
    if let Some(object) = info.as_object() {
        for (key, value) in object {
            if key != "properties" && !value.is_null() {
                props.entry(key.clone()).or_insert_with(|| value.clone());
            }
        }
    }
}

/// Value of one parameter on an area: a stored value under the key or an alias; the axis of the
/// threshold gradient for the per-axis keys; the schema default; else `null` (not set).
fn current_value_for(entry: &AreaParameterSchemaEntry, props: &HashMap<String, Value>) -> Value {
    if let Some(found) = entry.all_keys().find_map(|k| props.get(k)) {
        return found.clone();
    }
    if let Some(axis) = entry.key.strip_prefix("firing_threshold_increment_") {
        let index = ["x", "y", "z"].iter().position(|a| *a == axis);
        let gradient = props
            .get("neuron_fire_threshold_increment")
            .or_else(|| props.get("firing_threshold_increment"));
        let from = gradient.and_then(|g| match (g, index) {
            (Value::Array(items), Some(i)) => items.get(i).cloned(),
            (Value::Object(map), _) => map.get(axis).cloned(),
            _ => None,
        });
        if let Some(value) = from {
            return value;
        }
    }
    entry.default.clone().unwrap_or(Value::Null)
}

/// Build the schema. `props` fills `current_value`; `area_group` keeps only entries that apply to
/// that group (`IPU`, `OPU`, `CORE`, `CUSTOM`, `MEMORY`).
pub fn build_area_parameter_schema(
    area_group: Option<&str>,
    props: Option<&HashMap<String, Value>>,
) -> Vec<AreaParameterSchemaEntry> {
    let parameter = CorticalChangeClassifier::parameter_changes();
    let metadata = CorticalChangeClassifier::metadata_changes();
    let structural = CorticalChangeClassifier::structural_changes();
    let special = CorticalChangeClassifier::special_parameters();
    raw_specs()
        .into_iter()
        .filter(|e| match area_group {
            Some(g) => e.applies_to.iter().any(|a| a.eq_ignore_ascii_case(g)),
            None => true,
        })
        .map(|mut e| {
            let key = e.key.as_str();
            let (class, live) = if parameter.contains(key) {
                ("parameter", true)
            } else if metadata.contains(key) {
                ("metadata", true)
            } else if special.contains(key) {
                ("special", false)
            } else if structural.contains(key) {
                ("structural", false)
            } else {
                ("unclassified", false)
            };
            e.category = category_for(&e.key).to_string();
            e.update_class = class.to_string();
            e.live_updatable = live;
            if let Some(props) = props {
                e.current_value = Some(current_value_for(&e, props));
            }
            e
        })
        .collect()
}

/// Find the spec an update key (canonical or alias) belongs to.
pub fn find_parameter(key: &str) -> Option<AreaParameterSchemaEntry> {
    build_area_parameter_schema(None, None)
        .into_iter()
        .find(|e| e.all_keys().any(|k| k == key))
}

/// Enforce the hard bounds the schema states (numeric min/max, enum membership) on one value.
/// Keys without hard bounds, and `null` on nullable keys, pass. Used by the update path so the
/// schema and validation cannot disagree.
pub fn validate_hard_bounds(key: &str, value: &Value) -> Result<(), String> {
    let Some(entry) = find_parameter(key) else {
        return Ok(());
    };
    if entry.min.is_none() && entry.max.is_none() && entry.enum_values.is_none() {
        return Ok(());
    }
    if value.is_null() && entry.nullable {
        return Ok(());
    }
    if let Some(values) = &entry.enum_values {
        return match value.as_str() {
            Some(s) if values.iter().any(|v| v == s) => Ok(()),
            _ => Err(format!("{key} must be one of {values:?}; got {value}")),
        };
    }
    let n = value
        .as_f64()
        .filter(|n| n.is_finite())
        .ok_or_else(|| format!("{key} must be a number; got {value}"))?;
    if entry.value_type == "integer" && n.fract() != 0.0 {
        return Err(format!("{key} must be a whole number; got {n}"));
    }
    let unit = entry
        .unit
        .as_deref()
        .map(|u| format!(" ({u})"))
        .unwrap_or_default();
    let below = match entry.min {
        Some(min) => n < min || (entry.min_exclusive && n <= min),
        None => false,
    };
    let above = entry.max.is_some_and(|max| n > max);
    if below || above {
        let lo = entry
            .min
            .map(|m| format!("{}{m}", if entry.min_exclusive { "> " } else { ">= " }));
        let hi = entry.max.map(|m| format!("<= {m}"));
        let range = [lo, hi]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" and ");
        return Err(format!("{key} must be {range}{unit}; got {n}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn classifier_universe() -> BTreeSet<String> {
        [
            CorticalChangeClassifier::parameter_changes(),
            CorticalChangeClassifier::metadata_changes(),
            CorticalChangeClassifier::structural_changes(),
            CorticalChangeClassifier::special_parameters(),
        ]
        .iter()
        .flat_map(|s| s.iter().map(|k| k.to_string()))
        .collect()
    }

    #[test]
    fn schema_keys_equal_update_path_keys() {
        let schema: Vec<_> = build_area_parameter_schema(None, None);
        let mut seen = BTreeSet::new();
        for e in &schema {
            for k in e.all_keys() {
                assert!(
                    seen.insert(k.to_string()),
                    "key {k} listed twice in the schema"
                );
            }
            assert_ne!(
                e.update_class, "unclassified",
                "{} is not in the classifier",
                e.key
            );
        }
        let universe = classifier_universe();
        let missing: Vec<_> = universe.difference(&seen).collect();
        let extra: Vec<_> = seen.difference(&universe).collect();
        assert!(
            missing.is_empty(),
            "update-path keys missing from schema: {missing:?}"
        );
        assert!(
            extra.is_empty(),
            "schema keys the update path does not know: {extra:?}"
        );
    }

    #[test]
    fn every_entry_has_a_known_category() {
        let s = build_area_parameter_schema(None, None);
        assert!(s.iter().all(|e| CATEGORIES.contains(&e.category.as_str())));
        let by = |c: &str| s.iter().filter(|e| e.category == c).count();
        assert!(by("Neuron Firing Parameters") >= 10 && by("Summary") >= 5);
    }

    #[test]
    fn area_filter_and_current_value() {
        let io = build_area_parameter_schema(Some("CUSTOM"), None);
        assert!(io.iter().all(|e| e.key != "memory_neuron_cap"));
        let mem = build_area_parameter_schema(Some("memory"), None);
        assert!(mem.iter().any(|e| e.key == "memory_neuron_cap"));
        let props = HashMap::from([("leak_coefficient".to_string(), json!(0.25))]);
        let s = build_area_parameter_schema(None, Some(&props));
        let leak = s
            .iter()
            .find(|e| e.key == "neuron_leak_coefficient")
            .unwrap();
        assert_eq!(leak.current_value, Some(json!(0.25)));
        assert!(leak.live_updatable && leak.update_class == "parameter");
    }

    #[test]
    fn every_parameter_has_a_current_value_when_an_area_is_given() {
        // An area that stores almost nothing: the schema must still answer for every key,
        // with `null` meaning "no setting on this area".
        let props = HashMap::from([("name".to_string(), json!("a"))]);
        let with_area = build_area_parameter_schema(None, Some(&props));
        let missing: Vec<_> = with_area
            .iter()
            .filter(|e| e.current_value.is_none())
            .map(|e| &e.key)
            .collect();
        assert!(missing.is_empty(), "no current_value for {missing:?}");
        assert!(build_area_parameter_schema(None, None)
            .iter()
            .all(|e| e.current_value.is_none()));
        let by = |k: &str| {
            with_area
                .iter()
                .find(|e| e.key == k)
                .unwrap()
                .current_value
                .clone()
        };
        assert_eq!(by("cortical_name"), Some(json!("a")));
        assert_eq!(by("modulators"), Some(Value::Null));
        assert_eq!(by("mp_change_mode"), Some(json!("none")));
    }

    #[test]
    fn effective_values_fill_gaps_without_overwriting_and_split_the_gradient() {
        let mut props = HashMap::from([("neuron_snooze_period".to_string(), json!(7))]);
        let info = json!({
            "neuron_snooze_period": 3,
            "parent_region_id": "region-1",
            "neuron_fire_threshold_increment": [1.0, 2.0, 3.0],
            "properties": {"ignored": true},
        });
        merge_effective_values(&mut props, &info);
        assert!(!props.contains_key("properties"));
        let s = build_area_parameter_schema(None, Some(&props));
        let by = |k: &str| s.iter().find(|e| e.key == k).unwrap().current_value.clone();
        assert_eq!(by("neuron_snooze_period"), Some(json!(7)));
        assert_eq!(by("region_id"), Some(json!("region-1")));
        assert_eq!(by("firing_threshold_increment_y"), Some(json!(2.0)));
    }

    #[test]
    fn hard_bounds_match_schema_min_max() {
        for e in build_area_parameter_schema(None, None) {
            if e.enum_values.is_some() || (e.min.is_none() && e.max.is_none()) {
                continue;
            }
            if let Some(min) = e.min {
                assert_eq!(
                    validate_hard_bounds(&e.key, &json!(min)).is_ok(),
                    !e.min_exclusive,
                    "{} at min",
                    e.key
                );
                assert!(
                    validate_hard_bounds(&e.key, &json!(min - 1.0)).is_err(),
                    "{} below min",
                    e.key
                );
            }
            if let Some(max) = e.max {
                assert!(
                    validate_hard_bounds(&e.key, &json!(max)).is_ok(),
                    "{} at max",
                    e.key
                );
                assert!(
                    validate_hard_bounds(&e.key, &json!(max + 1.0)).is_err(),
                    "{} above max",
                    e.key
                );
            }
        }
        // The sample named in the brief: excitability is a fraction.
        assert!(validate_hard_bounds("neuron_excitability", &json!(0.25)).is_ok());
        assert!(validate_hard_bounds("excitability", &json!(25)).is_err());
        assert!(validate_hard_bounds("mp_change_mode", &json!("bogus")).is_err());
    }

    #[cfg(feature = "std")]
    #[test]
    fn leak_and_fire_count_bounds_match_validation() {
        for key in ["neuron_leak_coefficient", "neuron_leak_variability"] {
            let e = find_parameter(key).unwrap();
            assert_eq!((e.min, e.max), (Some(0.0), Some(1.0)), "{key}");
            assert!(validate_hard_bounds(key, &json!(1.0)).is_ok());
            let err = validate_hard_bounds(key, &json!(1.5)).unwrap_err();
            assert!(err.contains(key) && err.contains("<= 1"), "{err}");
            assert!(validate_hard_bounds(key, &json!(-0.01)).is_err());
        }
        let c = find_parameter("neuron_consecutive_fire_count").unwrap();
        assert_eq!(c.value_type, "integer");
        assert_eq!(c.max, Some(u16::MAX as f64));
        assert!(validate_hard_bounds("consecutive_fire_count", &json!(2.5)).is_err());
        assert!(validate_hard_bounds("consecutive_fire_count", &json!(0)).is_ok());
    }

    #[test]
    fn rate_modulated_leak_fields_match_burst_engine_validation() {
        use feagi_npu_burst_engine::rate_modulated_leak::RateModulatedLeakConfig as C;
        let rml = find_parameter("rate_modulated_leak").unwrap();
        let fields = rml.fields.unwrap();
        let int_field = |f: &AreaParameterSchemaEntry, x: f64| {
            if f.value_type == "integer" {
                json!(x as i64)
            } else {
                json!(x)
            }
        };
        let build = |field: &str, v: Value| {
            let mut o = json!({"enabled": true, "leak_min": 0.0, "leak_max": 1.0});
            o[field] = v;
            o
        };
        for f in &fields {
            if let Some(values) = &f.enum_values {
                for v in values {
                    assert!(
                        C::validate(&build(&f.key, json!(v))).is_ok(),
                        "{} = {v}",
                        f.key
                    );
                }
                assert!(
                    C::validate(&build(&f.key, json!("bogus"))).is_err(),
                    "{} bogus",
                    f.key
                );
                continue;
            }
            if let Some(min) = f.min {
                let at_min = C::validate(&build(&f.key, int_field(f, min))).is_ok();
                assert_eq!(at_min, !f.min_exclusive, "{} at min {min}", f.key);
                assert!(
                    C::validate(&build(&f.key, int_field(f, min - 1.0))).is_err(),
                    "{} below min",
                    f.key
                );
            }
            if let Some(max) = f.max {
                assert!(
                    C::validate(&build(&f.key, int_field(f, max))).is_ok(),
                    "{} at max {max}",
                    f.key
                );
                assert!(
                    C::validate(&build(&f.key, int_field(f, max + 1.0))).is_err(),
                    "{} above max",
                    f.key
                );
            }
            if let Some(d) = &f.default {
                assert!(
                    C::validate(&build(&f.key, d.clone())).is_ok(),
                    "{} default",
                    f.key
                );
            }
        }
        // Every field the config struct serializes is in the schema.
        let ser =
            serde_json::to_value(C::validate(&json!({"enabled": true})).unwrap().unwrap()).unwrap();
        for k in ser.as_object().unwrap().keys() {
            assert!(
                fields.iter().any(|f| &f.key == k),
                "rml field {k} missing from schema"
            );
        }
    }
}
