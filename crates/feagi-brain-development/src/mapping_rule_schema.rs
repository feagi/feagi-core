// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Single source of truth for the fields of a dict-format cortical mapping rule: type, range,
//! default, allowed values, and whether the field only configures plasticity.
//!
//! `ConnectomeManager` validation reads the ranges from here, and
//! `GET /v1/cortical_mapping/rule_schema` serves this table, so clients and the validator
//! cannot disagree.

use serde_json::{json, Map, Value};

/// JSON type of a rule field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleFieldType {
    Boolean,
    Integer,
    Number,
    String,
    Array,
    Object,
}

impl RuleFieldType {
    /// Name used in the schema reply.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Boolean => "boolean",
            Self::Integer => "integer",
            Self::Number => "number",
            Self::String => "string",
            Self::Array => "array",
            Self::Object => "object",
        }
    }
}

/// Default a field resolves to when the rule omits it (or sets null).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RuleDefault {
    /// No default: the field is optional with no effect, or required.
    None,
    Bool(bool),
    Int(i64),
    Num(f64),
    Str(&'static str),
}

/// One mapping-rule field.
#[derive(Debug, Clone, Copy)]
pub struct MappingRuleField {
    pub key: &'static str,
    pub kind: RuleFieldType,
    /// Lower bound, if any.
    pub min: Option<f64>,
    /// When true the lower bound itself is excluded (`value > min`).
    pub min_exclusive: bool,
    /// Upper bound (inclusive), if any.
    pub max: Option<f64>,
    pub default: RuleDefault,
    /// Allowed string values (case-insensitive), for enumerated fields.
    pub allowed: &'static [&'static str],
    /// True when changing the field never changes which synapses exist or their stored weights.
    pub plasticity_only: bool,
}

impl MappingRuleField {
    /// True when `value` is a finite-or-infinite non-NaN number inside the field's range.
    pub fn accepts_number(&self, value: f64) -> bool {
        if value.is_nan() {
            return false;
        }
        if let Some(min) = self.min {
            if value < min || (self.min_exclusive && value <= min) {
                return false;
            }
        }
        if let Some(max) = self.max {
            if value > max {
                return false;
            }
        }
        true
    }

    /// JSON entry served by the schema endpoint.
    pub fn to_schema_json(&self) -> Value {
        let default = match self.default {
            RuleDefault::None => Value::Null,
            RuleDefault::Bool(b) => json!(b),
            RuleDefault::Int(i) => json!(i),
            RuleDefault::Num(f) => json!(f),
            RuleDefault::Str(s) => json!(s),
        };
        let mut entry = Map::new();
        entry.insert("type".into(), json!(self.kind.as_str()));
        entry.insert("min".into(), self.min.map_or(Value::Null, |v| json!(v)));
        entry.insert("max".into(), self.max.map_or(Value::Null, |v| json!(v)));
        entry.insert("default".into(), default);
        if self.min_exclusive {
            entry.insert("exclusive_min".into(), json!(true));
        }
        if !self.allowed.is_empty() {
            entry.insert("enum".into(), json!(self.allowed));
        }
        entry.insert("plasticity_only".into(), json!(self.plasticity_only));
        Value::Object(entry)
    }
}

const fn field(key: &'static str, kind: RuleFieldType, plasticity_only: bool) -> MappingRuleField {
    MappingRuleField {
        key,
        kind,
        min: None,
        min_exclusive: false,
        max: None,
        default: RuleDefault::None,
        allowed: &[],
        plasticity_only,
    }
}

pub const PLASTICITY_FLAG: MappingRuleField = MappingRuleField {
    default: RuleDefault::Bool(false),
    ..field("plasticity_flag", RuleFieldType::Boolean, true)
};
pub const PLASTICITY_MODE: MappingRuleField = MappingRuleField {
    default: RuleDefault::Str("stdp"),
    allowed: &["off", "stdp", "rstdp"],
    ..field("plasticity_mode", RuleFieldType::String, true)
};
pub const PLASTICITY_WINDOW: MappingRuleField = MappingRuleField {
    min: Some(0.0),
    ..field("plasticity_window", RuleFieldType::Integer, true)
};
pub const PLASTICITY_CONSTANT: MappingRuleField =
    field("plasticity_constant", RuleFieldType::Integer, true);
/// LTP / LTD multipliers are stored in the NPU as `i8`.
pub const LTP_MULTIPLIER: MappingRuleField = MappingRuleField {
    min: Some(i8::MIN as f64),
    max: Some(i8::MAX as f64),
    ..field("ltp_multiplier", RuleFieldType::Integer, true)
};
pub const LTD_MULTIPLIER: MappingRuleField = MappingRuleField {
    min: Some(i8::MIN as f64),
    max: Some(i8::MAX as f64),
    ..field("ltd_multiplier", RuleFieldType::Integer, true)
};
pub const ELIGIBILITY_DECAY_BURSTS: MappingRuleField = MappingRuleField {
    min: Some(0.0),
    max: Some(u32::MAX as f64),
    default: RuleDefault::Int(0),
    ..field("eligibility_decay_bursts", RuleFieldType::Integer, true)
};
/// Learning-rate scale on the end-of-burst weight commit: finite and strictly positive.
pub const PLASTICITY_ETA: MappingRuleField = MappingRuleField {
    min: Some(0.0),
    min_exclusive: true,
    max: Some(f32::MAX as f64),
    default: RuleDefault::Num(1.0),
    ..field("plasticity_eta", RuleFieldType::Number, true)
};
/// Upper clamp on plastic weights: strictly positive; absent means no clamp.
pub const MAX_WEIGHT: MappingRuleField = MappingRuleField {
    min: Some(0.0),
    min_exclusive: true,
    ..field("max_weight", RuleFieldType::Number, true)
};
pub const ONE_SIDED_FIRING: MappingRuleField = MappingRuleField {
    default: RuleDefault::Str("decrement"),
    allowed: &["decrement", "ignore"],
    ..field("one_sided_firing", RuleFieldType::String, true)
};
pub const MODULATORS: MappingRuleField = field("modulators", RuleFieldType::Array, true);
pub const CLASSIFIER_INSTANCE_REWARD: MappingRuleField =
    field("classifier_instance_reward", RuleFieldType::String, true);
pub const REWARD_SOURCE_AREA: MappingRuleField =
    field("reward_source_area", RuleFieldType::String, true);
pub const PUNISHMENT_SOURCE_AREA: MappingRuleField =
    field("punishment_source_area", RuleFieldType::String, true);

/// Structural fields: they decide which synapses exist, so changing them rebuilds the mapping.
pub const MORPHOLOGY_ID: MappingRuleField = field("morphology_id", RuleFieldType::String, false);
pub const MORPHOLOGY_SCALAR: MappingRuleField =
    field("morphology_scalar", RuleFieldType::Array, false);
pub const POST_SYNAPTIC_CURRENT_MULTIPLIER: MappingRuleField = field(
    "postSynapticCurrent_multiplier",
    RuleFieldType::Number,
    false,
);
pub const SYNAPTIC_DELAY_BURSTS: MappingRuleField = MappingRuleField {
    min: Some(1.0),
    default: RuleDefault::Int(1),
    ..field("synaptic_delay_bursts", RuleFieldType::Integer, false)
};

/// Every field of a dict-format mapping rule.
pub const MAPPING_RULE_FIELDS: [MappingRuleField; 18] = [
    MORPHOLOGY_ID,
    MORPHOLOGY_SCALAR,
    POST_SYNAPTIC_CURRENT_MULTIPLIER,
    SYNAPTIC_DELAY_BURSTS,
    PLASTICITY_FLAG,
    PLASTICITY_MODE,
    PLASTICITY_WINDOW,
    PLASTICITY_CONSTANT,
    LTP_MULTIPLIER,
    LTD_MULTIPLIER,
    ELIGIBILITY_DECAY_BURSTS,
    PLASTICITY_ETA,
    MAX_WEIGHT,
    ONE_SIDED_FIRING,
    MODULATORS,
    CLASSIFIER_INSTANCE_REWARD,
    REWARD_SOURCE_AREA,
    PUNISHMENT_SOURCE_AREA,
];

/// True when `key` only configures plasticity (see [`MappingRuleField::plasticity_only`]).
pub fn is_plasticity_only_key(key: &str) -> bool {
    MAPPING_RULE_FIELDS
        .iter()
        .any(|f| f.plasticity_only && f.key == key)
}

/// The schema body: `{"fields": {<key>: {type, min, max, default, enum?, plasticity_only}}}`.
pub fn mapping_rule_schema_json() -> Value {
    let fields: Map<String, Value> = MAPPING_RULE_FIELDS
        .iter()
        .map(|f| (f.key.to_string(), f.to_schema_json()))
        .collect();
    json!({ "fields": fields })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_plasticity_only_key_is_flagged_in_the_schema() {
        let schema = mapping_rule_schema_json();
        let fields = schema["fields"].as_object().unwrap();
        // The keys that configure plasticity only (changing them never rebuilds synapses).
        const EXPECTED: [&str; 14] = [
            "plasticity_flag",
            "plasticity_mode",
            "plasticity_window",
            "plasticity_constant",
            "ltp_multiplier",
            "ltd_multiplier",
            "eligibility_decay_bursts",
            "plasticity_eta",
            "max_weight",
            "one_sided_firing",
            "modulators",
            "classifier_instance_reward",
            "reward_source_area",
            "punishment_source_area",
        ];
        let flagged = MAPPING_RULE_FIELDS
            .iter()
            .filter(|f| f.plasticity_only)
            .count();
        assert_eq!(flagged, EXPECTED.len());
        for key in EXPECTED {
            assert!(is_plasticity_only_key(key));
            assert_eq!(
                fields[key]["plasticity_only"],
                json!(true),
                "{key} must be plasticity_only in the schema"
            );
        }
        // Structural keys are not flagged.
        assert_eq!(fields["morphology_id"]["plasticity_only"], json!(false));
        assert_eq!(
            fields["postSynapticCurrent_multiplier"]["plasticity_only"],
            json!(false)
        );
    }

    #[test]
    fn schema_ranges_match_the_validator() {
        // The validator rejects a non-positive eta and max_weight; the schema excludes them too.
        let mut rule = serde_json::Map::new();
        rule.insert("plasticity_mode".into(), json!("stdp"));
        for (field, bad) in [(&PLASTICITY_ETA, 0.0), (&MAX_WEIGHT, -1.0)] {
            let entry = field.to_schema_json();
            assert_eq!(entry["min"], json!(0.0));
            assert_eq!(entry["exclusive_min"], json!(true));
            assert!(!field.accepts_number(bad));
            let mut r = rule.clone();
            r.insert(field.key.into(), json!(bad));
            let err = crate::connectome_manager::ConnectomeManager::validate_plastic_rule_options(
                &r, "src", "dst",
            )
            .unwrap_err();
            assert!(err.contains(field.key), "{err}");
            // An in-range value passes both.
            assert!(field.accepts_number(0.5));
            r.insert(field.key.into(), json!(0.5));
            assert!(
                crate::connectome_manager::ConnectomeManager::validate_plastic_rule_options(
                    &r, "src", "dst"
                )
                .is_ok()
            );
        }
        // i8 multipliers.
        assert!(!LTP_MULTIPLIER.accepts_number(128.0));
        assert!(LTP_MULTIPLIER.accepts_number(-128.0));
    }

    #[test]
    fn enum_values_are_accepted_by_the_runtime_parsers() {
        for v in ONE_SIDED_FIRING.allowed {
            assert!(feagi_npu_burst_engine::npu::OneSidedFiring::parse(v).is_some());
        }
        assert!(feagi_npu_burst_engine::npu::OneSidedFiring::parse("bogus").is_none());
    }
}
