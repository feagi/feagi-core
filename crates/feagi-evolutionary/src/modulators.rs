// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Genome modulator instances.
//!
//! An instance is a named copy of a built-in [`ModulatorKind`] plus the cortical
//! id of the 1x1x1 driver area that turns it on. Areas and mapping rules
//! subscribe by instance id.

use feagi_structures::genomic::cortical_area::CorticalID;
use feagi_structures::genomic::{
    validate_instance_fields, ModulatorKind, ModulatorValidationError,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::types::{EvoError, EvoResult};

/// One user-defined modulator and the driver area that owns it.
#[derive(Debug, Clone, PartialEq)]
pub struct ModulatorInstance {
    pub kind: ModulatorKind,
    pub magnitude_percent: f32,
    pub effect_duration_bursts: u16,
    pub rest_bursts: u16,
    pub graded: bool,
    pub full_scale_potential: Option<f32>,
    pub driver_cortical_id: CorticalID,
}

impl ModulatorInstance {
    /// Reject fields that cannot produce a defined signal.
    pub fn validate(&self) -> Result<(), ModulatorValidationError> {
        validate_instance_fields(
            self.magnitude_percent,
            self.effect_duration_bursts,
            self.graded,
            self.full_scale_potential,
        )
    }

    /// Hierarchical genome object for this instance.
    pub fn to_json(&self) -> Value {
        let mut object = serde_json::Map::new();
        object.insert("type".to_string(), json!(self.kind.as_str()));
        object.insert(
            "magnitude_percent".to_string(),
            json!(self.magnitude_percent),
        );
        object.insert(
            "effect_duration_bursts".to_string(),
            json!(self.effect_duration_bursts),
        );
        object.insert("rest_bursts".to_string(), json!(self.rest_bursts));
        object.insert("graded".to_string(), json!(self.graded));
        if let Some(scale) = self.full_scale_potential {
            object.insert("full_scale_potential".to_string(), json!(scale));
        }
        object.insert(
            "driver_cortical_id".to_string(),
            json!(self.driver_cortical_id.as_base_64()),
        );
        Value::Object(object)
    }
}

/// All modulator instances in a genome, keyed by instance id.
#[derive(Debug, Clone, Default)]
pub struct ModulatorRegistry {
    modulators: HashMap<String, ModulatorInstance>,
}

impl ModulatorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, id: String, instance: ModulatorInstance) {
        self.modulators.insert(id, instance);
    }

    pub fn get(&self, id: &str) -> Option<&ModulatorInstance> {
        self.modulators.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut ModulatorInstance> {
        self.modulators.get_mut(id)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.modulators.contains_key(id)
    }

    pub fn remove(&mut self, id: &str) -> Option<ModulatorInstance> {
        self.modulators.remove(id)
    }

    pub fn rename(&mut self, old_id: &str, new_id: String) -> bool {
        if let Some(instance) = self.modulators.remove(old_id) {
            self.modulators.insert(new_id, instance);
            true
        } else {
            false
        }
    }

    pub fn len(&self) -> usize {
        self.modulators.len()
    }

    pub fn is_empty(&self) -> bool {
        self.modulators.is_empty()
    }

    pub fn ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.modulators.keys().cloned().collect();
        ids.sort();
        ids
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &ModulatorInstance)> {
        self.modulators.iter()
    }

    /// Hierarchical `modulators` object.
    pub fn to_json(&self) -> Value {
        let mut object = serde_json::Map::new();
        let mut ids: Vec<&String> = self.modulators.keys().collect();
        ids.sort();
        for id in ids {
            if let Some(instance) = self.modulators.get(id) {
                object.insert(id.clone(), instance.to_json());
            }
        }
        Value::Object(object)
    }
}

/// Parse the top-level `modulators` object. An absent object is an empty registry.
pub fn parse_modulator_registry(value: Option<&Value>) -> EvoResult<ModulatorRegistry> {
    let Some(value) = value else {
        return Ok(ModulatorRegistry::new());
    };
    let Some(object) = value.as_object() else {
        return Err(EvoError::InvalidGenome(
            "modulators must be an object".to_string(),
        ));
    };
    let mut registry = ModulatorRegistry::new();
    for (id, body) in object {
        registry.insert(id.clone(), parse_modulator_instance(id, body)?);
    }
    Ok(registry)
}

fn parse_modulator_instance(id: &str, body: &Value) -> EvoResult<ModulatorInstance> {
    let object = body
        .as_object()
        .ok_or_else(|| EvoError::InvalidGenome(format!("modulator '{id}' must be an object")))?;
    let type_name = object
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| EvoError::InvalidGenome(format!("modulator '{id}' is missing type")))?;
    let kind = ModulatorKind::parse(type_name)
        .map_err(|err| EvoError::InvalidGenome(format!("modulator '{id}': {err}")))?;
    let magnitude_percent = object
        .get("magnitude_percent")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| {
            EvoError::InvalidGenome(format!("modulator '{id}' is missing magnitude_percent"))
        })? as f32;
    let effect_duration_bursts = object
        .get("effect_duration_bursts")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| {
            EvoError::InvalidGenome(format!(
                "modulator '{id}' is missing effect_duration_bursts"
            ))
        })?;
    if effect_duration_bursts > u16::MAX as u64 {
        return Err(EvoError::InvalidGenome(format!(
            "modulator '{id}' effect_duration_bursts exceeds u16"
        )));
    }
    let rest_bursts = object
        .get("rest_bursts")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if rest_bursts > u16::MAX as u64 {
        return Err(EvoError::InvalidGenome(format!(
            "modulator '{id}' rest_bursts exceeds u16"
        )));
    }
    let graded = object
        .get("graded")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let full_scale_potential = object
        .get("full_scale_potential")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32);
    let driver = object
        .get("driver_cortical_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            EvoError::InvalidGenome(format!("modulator '{id}' is missing driver_cortical_id"))
        })?;
    let driver_cortical_id = CorticalID::try_from_base_64(driver)
        .map_err(|err| EvoError::InvalidGenome(format!("modulator '{id}' driver id: {err}")))?;
    let instance = ModulatorInstance {
        kind,
        magnitude_percent,
        effect_duration_bursts: effect_duration_bursts as u16,
        rest_bursts: rest_bursts as u16,
        graded,
        full_scale_potential,
        driver_cortical_id,
    };
    instance
        .validate()
        .map_err(|err| EvoError::InvalidGenome(format!("modulator '{id}': {err}")))?;
    Ok(instance)
}

/// Locked neuron properties the instance writes onto its driver area.
pub fn driver_locked_properties(instance: &ModulatorInstance) -> HashMap<String, Value> {
    let mut properties = HashMap::new();
    properties.insert("refractory_period".to_string(), json!(0));
    properties.insert("spike_train".to_string(), json!(true));
    properties.insert(
        "consecutive_fire_cnt_max".to_string(),
        json!(instance.effect_duration_bursts),
    );
    properties.insert(
        "consecutive_fire_limit".to_string(),
        json!(instance.effect_duration_bursts),
    );
    properties.insert("snooze_length".to_string(), json!(instance.rest_bursts));
    properties.insert("snooze_period".to_string(), json!(instance.rest_bursts));
    properties.insert("mp_driven_psp".to_string(), json!(instance.graded));
    properties.insert("cortical_group".to_string(), json!("MODULATOR"));
    if let Some(scale) = instance.full_scale_potential {
        properties.insert("full_scale_potential".to_string(), json!(scale));
    }
    properties
}

/// Property names the API and Brain Visualizer must leave read-only on a driver.
pub fn driver_locked_property_names() -> &'static [&'static str] {
    &[
        "refractory_period",
        "spike_train",
        "consecutive_fire_cnt_max",
        "consecutive_fire_limit",
        "neuron_consecutive_fire_count",
        "consecutive_fire_count",
        "snooze_length",
        "snooze_period",
        "mp_driven_psp",
    ]
}

/// Request body shape shared by create and update. Serde is used by the API layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulatorWrite {
    #[serde(rename = "type")]
    pub kind: String,
    pub magnitude_percent: f32,
    pub effect_duration_bursts: u16,
    pub rest_bursts: u16,
    #[serde(default)]
    pub graded: bool,
    #[serde(default)]
    pub full_scale_potential: Option<f32>,
}

impl ModulatorWrite {
    pub fn into_instance(self, driver_cortical_id: CorticalID) -> EvoResult<ModulatorInstance> {
        let kind = ModulatorKind::parse(&self.kind).map_err(EvoError::InvalidGenome)?;
        let instance = ModulatorInstance {
            kind,
            magnitude_percent: self.magnitude_percent,
            effect_duration_bursts: self.effect_duration_bursts,
            rest_bursts: self.rest_bursts,
            graded: self.graded,
            full_scale_potential: self.full_scale_potential,
            driver_cortical_id,
        };
        instance
            .validate()
            .map_err(|err| EvoError::InvalidGenome(err.0))?;
        Ok(instance)
    }
}
