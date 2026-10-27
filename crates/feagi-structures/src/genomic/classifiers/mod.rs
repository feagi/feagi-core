// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
First-class genome classifier assembly.

A classifier is packaging around two memory areas joined by `associative_memory`.
It is stored under the top-level genome key `classifiers`, parallel to
`brain_regions`. It is not a brain region and is not exportable as a circuit.

External inputs are ordinary episodic connections selected by connectivity rule:

- `classifier_sample` lands on the sample memory
- `classifier_class` lands on the class memory and creates one class output
  whose shape matches that class input

The sample memory accepts any number of those inputs, including a mix of
memory and interconnect areas. The class memory does the same. Pattern identity
is the memory area's own episodic hash. The classifier does not add a second
input rule on top of that.
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Connectivity rule: episodic input into the classifier sample memory.
pub const CLASSIFIER_SAMPLE_MORPHOLOGY: &str = "classifier_sample";
/// Connectivity rule: episodic input into the classifier class memory.
pub const CLASSIFIER_CLASS_MORPHOLOGY: &str = "classifier_class";
/// Morphology used for sample-memory → class-memory bind.
pub const CLASSIFIER_ASSOCIATIVE_MORPHOLOGY: &str = "associative_memory";

/// True when a mapping rule is an episodic encode into a memory area.
///
/// Classifier sample and class rules are episodic encodes. A plain
/// `episodic_memory` rule remains one too, so a classifier memory area still
/// accepts the same upstream set a standalone memory area accepts.
pub fn morphology_is_episodic_input(morphology_id: &str) -> bool {
    matches!(
        morphology_id,
        "episodic_memory" | CLASSIFIER_SAMPLE_MORPHOLOGY | CLASSIFIER_CLASS_MORPHOLOGY
    )
}

/// Result of recording one mapping edit on a classifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifierMappingEffect {
    pub applied: bool,
    pub removed_output_id: Option<String>,
}

/// Directed mapping owned by a classifier assembly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifierMapping {
    pub src_area_id: String,
    pub dst_area_id: String,
    pub morphology_id: String,
}

/// One class input and the output area created for it.
///
/// The output shape matches the class input. Several bindings may each have
/// one firing neuron. That combo is one class, not an error.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClassifierClassBinding {
    pub class_area_id: String,
    pub class_output_area_id: String,
}

/// First-class classifier record persisted in the genome.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Classifier {
    /// UUID string, genome map key.
    pub classifier_id: String,
    pub name: String,
    /// Circuit region that contains this assembly.
    ///
    /// That region is a child of the region the user added the classifier to,
    /// and that parent may be root. The memory and error areas belong to this
    /// circuit, not to root.
    pub parent_region_id: String,
    pub coordinates_3d: [i32; 3],
    /// Episodic sample inputs on the sample memory. Order is not significant.
    #[serde(default)]
    pub sample_area_ids: Vec<String>,
    /// Episodic class inputs and the output created for each one.
    #[serde(default)]
    pub class_bindings: Vec<ClassifierClassBinding>,
    /// Owned sample memory. Deleting it deletes the classifier.
    pub sample_memory_id: String,
    /// Owned class memory. Deleting it deletes the classifier.
    pub class_memory_id: String,
    /// Owned 1x1x1 area. Fires when any class output mismatches its class input
    /// or has more than one firing neuron.
    pub error_area_id: String,
    /// Owned 1x1x1 area. Fires when every firing class input is answered by its class
    /// output and no output raises the error. Empty on classifiers created before it existed.
    #[serde(default)]
    pub correct_area_id: String,
    /// Per-pattern pain and pleasure on the associative mapping.
    #[serde(default)]
    pub reward_training: bool,
    /// Correct-answer area. Empty until the user selects one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer_feedback_area_id: Option<String>,
    /// `synaptic.reward` modulators the associative mapping learns from (R-STDP).
    /// The classifier's own right-answer and wrong-answer modulators are listed here
    /// like any other source.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reward_modulator_ids: Vec<String>,
    /// Bursts between a decision and the answer that grades it. Zero grades the same burst.
    #[serde(default)]
    pub answer_latency_bursts: u32,
    /// When set, pain and pleasure run only on bursts this area fires.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub learn_area_id: Option<String>,
    /// OPU that receives one surplus value per class channel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence_area_id: Option<String>,
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

impl Classifier {
    /// Sample memory and class memory. Deleting either deletes the assembly.
    pub fn assembly_core_ids(&self) -> Vec<String> {
        vec![self.sample_memory_id.clone(), self.class_memory_id.clone()]
    }

    /// Owned internals deleted with the classifier.
    pub fn owned_area_ids(&self) -> Vec<String> {
        let mut owned = self.assembly_core_ids();
        if !self.error_area_id.is_empty() {
            owned.push(self.error_area_id.clone());
        }
        if !self.correct_area_id.is_empty() {
            owned.push(self.correct_area_id.clone());
        }
        for binding in &self.class_bindings {
            if !binding.class_output_area_id.is_empty() {
                owned.push(binding.class_output_area_id.clone());
            }
        }
        owned
    }

    /// Referenced input areas that may be cleared independently.
    pub fn input_area_ids(&self) -> Vec<String> {
        let mut inputs = self.sample_area_ids.clone();
        for binding in &self.class_bindings {
            inputs.push(binding.class_area_id.clone());
        }
        inputs
    }

    /// Forget a deleted error or correct area. Returns true when `area_id` was one of them.
    /// The next reward update creates the area again.
    pub fn clear_answer_area(&mut self, area_id: &str) -> bool {
        if area_id.is_empty() {
            return false;
        }
        if self.error_area_id == area_id {
            self.error_area_id.clear();
            return true;
        }
        if self.correct_area_id == area_id {
            self.correct_area_id.clear();
            return true;
        }
        false
    }

    pub fn owns_assembly_core(&self, area_id: &str) -> bool {
        self.sample_memory_id == area_id || self.class_memory_id == area_id
    }

    /// True when `area_id` is sample memory, class memory, the error area, or a class output.
    pub fn owns_area(&self, area_id: &str) -> bool {
        self.owns_assembly_core(area_id)
            || self.error_area_id == area_id
            || (!self.correct_area_id.is_empty() && self.correct_area_id == area_id)
            || self.binding_for_output(area_id).is_some()
    }

    pub fn binding_for_class(&self, class_area_id: &str) -> Option<&ClassifierClassBinding> {
        self.class_bindings
            .iter()
            .find(|binding| binding.class_area_id == class_area_id)
    }

    pub fn binding_for_output(&self, output_area_id: &str) -> Option<&ClassifierClassBinding> {
        self.class_bindings
            .iter()
            .find(|binding| binding.class_output_area_id == output_area_id)
    }

    /// True when `area_id` is a referenced sample or class input of this classifier.
    pub fn references_input(&self, area_id: &str) -> bool {
        self.sample_area_ids.iter().any(|id| id == area_id)
            || self.binding_for_class(area_id).is_some()
    }

    /// Apply classifier-level metadata. Does not change owned internals or inputs.
    pub fn apply_metadata_update(
        &mut self,
        name: Option<String>,
        coordinates_3d: Option<[i32; 3]>,
    ) -> Result<(), String> {
        self.apply_assembly_update(name, coordinates_3d, None)
    }

    /// Apply classifier-level edit. Does not replace owned internals or input bindings.
    pub fn apply_assembly_update(
        &mut self,
        name: Option<String>,
        coordinates_3d: Option<[i32; 3]>,
        parent_region_id: Option<String>,
    ) -> Result<(), String> {
        if let Some(name) = name {
            let trimmed = name.trim();
            if trimmed.is_empty() {
                return Err("Classifier name cannot be blank".to_string());
            }
            self.name = trimmed.to_string();
        }
        if let Some(coordinates_3d) = coordinates_3d {
            self.coordinates_3d = coordinates_3d;
        }
        if let Some(parent_region_id) = parent_region_id {
            let trimmed = parent_region_id.trim();
            if trimmed.is_empty() {
                return Err("parent_region_id cannot be blank".to_string());
            }
            self.parent_region_id = trimmed.to_string();
        }
        Ok(())
    }

    /// Mappings this classifier requires given its current inputs.
    pub fn required_mappings(&self) -> Vec<ClassifierMapping> {
        let mut mappings = Vec::new();
        for sample_area_id in &self.sample_area_ids {
            mappings.push(ClassifierMapping {
                src_area_id: sample_area_id.clone(),
                dst_area_id: self.sample_memory_id.clone(),
                morphology_id: CLASSIFIER_SAMPLE_MORPHOLOGY.to_string(),
            });
        }
        for binding in &self.class_bindings {
            mappings.push(ClassifierMapping {
                src_area_id: binding.class_area_id.clone(),
                dst_area_id: self.class_memory_id.clone(),
                morphology_id: CLASSIFIER_CLASS_MORPHOLOGY.to_string(),
            });
        }
        mappings.push(ClassifierMapping {
            src_area_id: self.sample_memory_id.clone(),
            dst_area_id: self.class_memory_id.clone(),
            morphology_id: CLASSIFIER_ASSOCIATIVE_MORPHOLOGY.to_string(),
        });
        mappings
    }

    /// Drop an input reference when that area is deleted.
    ///
    /// Returns the class output id that must be deleted with a removed class input.
    pub fn clear_input(&mut self, area_id: &str) -> Option<String> {
        self.sample_area_ids.retain(|id| id != area_id);
        if self.answer_feedback_area_id.as_deref() == Some(area_id) {
            self.answer_feedback_area_id = None;
        }
        if self.learn_area_id.as_deref() == Some(area_id) {
            self.learn_area_id = None;
        }
        if self.confidence_area_id.as_deref() == Some(area_id) {
            self.confidence_area_id = None;
        }
        self.detach_class(area_id)
    }

    /// Record one sample input. A second call for the same area is a no-op success.
    pub fn attach_sample(&mut self, sample_area_id: String) -> Result<(), String> {
        let sample_area_id = required_area_id(sample_area_id, "sample_area_id")?;
        if !self.sample_area_ids.iter().any(|id| id == &sample_area_id) {
            self.sample_area_ids.push(sample_area_id);
        }
        Ok(())
    }

    /// Remove one sample input.
    pub fn detach_sample(&mut self, sample_area_id: &str) -> bool {
        let before = self.sample_area_ids.len();
        self.sample_area_ids.retain(|id| id != sample_area_id);
        self.sample_area_ids.len() != before
    }

    /// Record one class input and its output. The output id may be filled after the area exists.
    pub fn attach_class(
        &mut self,
        class_area_id: String,
        class_output_area_id: String,
    ) -> Result<(), String> {
        let class_area_id = required_area_id(class_area_id, "class_area_id")?;
        if self.binding_for_class(&class_area_id).is_some() {
            return Err(format!(
                "class_area_id {class_area_id} is already mapped to this classifier"
            ));
        }
        self.class_bindings.push(ClassifierClassBinding {
            class_area_id,
            class_output_area_id,
        });
        Ok(())
    }

    /// Fill the output id for a class input that was recorded before the area existed.
    pub fn set_class_output(
        &mut self,
        class_area_id: &str,
        class_output_area_id: String,
    ) -> Result<(), String> {
        let class_output_area_id = required_area_id(class_output_area_id, "class_output_area_id")?;
        let binding = self
            .class_bindings
            .iter_mut()
            .find(|binding| binding.class_area_id == class_area_id)
            .ok_or_else(|| format!("class_area_id {class_area_id} is not mapped"))?;
        binding.class_output_area_id = class_output_area_id;
        Ok(())
    }

    /// Remove one class binding. Returns the output area id when one was recorded.
    pub fn detach_class(&mut self, class_area_id: &str) -> Option<String> {
        let position = self
            .class_bindings
            .iter()
            .position(|binding| binding.class_area_id == class_area_id)?;
        let removed = self.class_bindings.remove(position);
        if removed.class_output_area_id.is_empty() {
            None
        } else {
            Some(removed.class_output_area_id)
        }
    }

    /// Bind or clear sample and class inputs from a mapping change.
    ///
    /// `applied` is false when the edge does not belong to this classifier.
    /// `removed_output_id` is set when a class input mapping is removed.
    pub fn apply_mapping_change(
        &mut self,
        src_area_id: &str,
        dst_area_id: &str,
        morphology_id: &str,
        removed: bool,
    ) -> Result<ClassifierMappingEffect, String> {
        if dst_area_id == self.sample_memory_id && morphology_id == CLASSIFIER_SAMPLE_MORPHOLOGY {
            if removed {
                self.detach_sample(src_area_id);
            } else {
                self.attach_sample(src_area_id.to_string())?;
            }
            return Ok(ClassifierMappingEffect {
                applied: true,
                removed_output_id: None,
            });
        }
        if dst_area_id == self.class_memory_id && morphology_id == CLASSIFIER_CLASS_MORPHOLOGY {
            if removed {
                return Ok(ClassifierMappingEffect {
                    applied: true,
                    removed_output_id: self.detach_class(src_area_id),
                });
            }
            if self.binding_for_class(src_area_id).is_none() {
                self.attach_class(src_area_id.to_string(), String::new())?;
            }
            return Ok(ClassifierMappingEffect {
                applied: true,
                removed_output_id: None,
            });
        }
        Ok(ClassifierMappingEffect {
            applied: false,
            removed_output_id: None,
        })
    }
}

/// One class input/output pair observed on a single burst.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassPairFiring {
    pub input_coords: Vec<(u32, u32, u32)>,
    pub output_coords: Vec<(u32, u32, u32)>,
}

/// True when this class output should raise the classifier error.
///
/// More than one firing neuron is counted inside this output only. Other class
/// outputs may each have one firing neuron in the same burst. A silent class
/// input is not a mismatch. A firing class input mismatches when its coordinates
/// differ from this output.
pub fn class_output_reports_error(pair: &ClassPairFiring) -> bool {
    if pair.output_coords.len() > 1 {
        return true;
    }
    if pair.input_coords.is_empty() {
        return false;
    }
    let mut input = pair.input_coords.clone();
    let mut output = pair.output_coords.clone();
    input.sort_unstable();
    output.sort_unstable();
    input.dedup();
    output.dedup();
    input != output
}

/// True when any class output of this classifier raises the error.
pub fn classifier_reports_error(pairs: &[ClassPairFiring]) -> bool {
    pairs.iter().any(class_output_reports_error)
}

/// True when at least one class input fired and every class output answered it without error.
pub fn classifier_reports_correct(pairs: &[ClassPairFiring]) -> bool {
    pairs.iter().any(|pair| !pair.input_coords.is_empty()) && !classifier_reports_error(pairs)
}

/// Bursts between a memory area receiving its upstream input and its memory neuron firing.
///
/// Pattern detection runs after the input arrives, and the memory neuron is injected into the
/// next burst's FCL (the memory-neuron injection path in the burst engine, `npu.rs`, and
/// plasticity `inject_memory_neuron_to_fcl`), so it fires one burst later.
pub const MEMORY_NEURON_FIRING_LATENCY_BURSTS: u32 = 1;

/// Bursts from the class-memory replay neuron firing to the class output firing.
pub const CLASS_MEMORY_REPLAY_BURSTS: u32 = 1;

/// Bursts from a class input firing to its class output answering it: the hop into class
/// memory (at least one burst), plus the memory neuron's firing latency, plus the replay
/// onto the output. With the default one-burst rule delay this is 3.
pub fn classifier_answer_delay_bursts(input_to_memory_delay_bursts: u32) -> u32 {
    input_to_memory_delay_bursts.max(1)
        + MEMORY_NEURON_FIRING_LATENCY_BURSTS
        + CLASS_MEMORY_REPLAY_BURSTS
}

/// Bursts from co-activity on the associative mapping until the answer reaches it as reward:
/// the answer delay, one burst for the correct or error area to fire, and one for the
/// modulator to commit its signal.
pub fn classifier_reward_eligibility_bursts(answer_delay_bursts: u32) -> u32 {
    answer_delay_bursts + 2
}

/// Visible name of one class output.
pub fn class_output_area_name(classifier_name: &str, class_area_name: &str) -> String {
    format!("{classifier_name} {class_area_name} class output")
}

/// Mapping rule written for one classifier-owned edge.
///
/// Only `associative_memory` is plastic. Sample and class rules are episodic
/// encodes and create no synapses.
pub fn classifier_mapping_rule(morphology_id: &str, associative_window: u32) -> serde_json::Value {
    let is_associative = morphology_id == CLASSIFIER_ASSOCIATIVE_MORPHOLOGY;
    let plasticity_value = if is_associative { 1 } else { 0 };
    serde_json::json!({
        "morphology_id": morphology_id,
        "morphology_scalar": [1, 1, 1],
        "postSynapticCurrent_multiplier": 1,
        "plasticity_flag": is_associative,
        "plasticity_constant": plasticity_value,
        "ltp_multiplier": plasticity_value,
        "ltd_multiplier": plasticity_value,
        "plasticity_window": if is_associative { associative_window } else { 0 },
    })
}

fn required_area_id(area_id: String, field: &str) -> Result<String, String> {
    let trimmed = area_id.trim();
    if trimmed.is_empty() {
        return Err(format!("{field} cannot be blank"));
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Classifier {
        let mut classifier = Classifier {
            classifier_id: "clf-1".to_string(),
            name: "demo".to_string(),
            parent_region_id: "region".to_string(),
            coordinates_3d: [1, 2, 3],
            sample_area_ids: vec!["sample-a".to_string()],
            class_bindings: Vec::new(),
            sample_memory_id: "smem".to_string(),
            class_memory_id: "cmem".to_string(),
            error_area_id: "err".to_string(),
            reward_training: false,
            answer_feedback_area_id: None,
            correct_area_id: String::new(),
            reward_modulator_ids: Vec::new(),
            answer_latency_bursts: 0,
            learn_area_id: None,
            confidence_area_id: None,
            properties: HashMap::new(),
        };
        classifier
            .attach_class("class-a".to_string(), "out-a".to_string())
            .expect("class");
        classifier
    }

    #[test]
    fn deleted_answer_area_is_forgotten_and_no_longer_owned() {
        let mut classifier = sample();
        classifier.correct_area_id = "ok".to_string();
        assert!(classifier.owns_area("ok"));
        assert!(classifier.clear_answer_area("ok"));
        assert!(classifier.correct_area_id.is_empty());
        assert!(!classifier.owned_area_ids().contains(&"ok".to_string()));
        assert!(classifier.clear_answer_area("err"));
        assert!(classifier.error_area_id.is_empty());
        assert!(
            !classifier.clear_answer_area("smem"),
            "only answer areas are cleared"
        );
        assert!(!classifier.clear_answer_area(""));
    }

    #[test]
    fn answer_delay_and_eligibility_follow_the_circuit() {
        assert_eq!(classifier_answer_delay_bursts(1), 3);
        assert_eq!(
            classifier_answer_delay_bursts(0),
            3,
            "a zero rule delay still takes one burst"
        );
        assert_eq!(classifier_answer_delay_bursts(3), 5);
        assert_eq!(classifier_reward_eligibility_bursts(3), 5);
    }

    #[test]
    fn correct_needs_an_answered_input_and_no_error() {
        let answered = ClassPairFiring {
            input_coords: vec![(0, 0, 0)],
            output_coords: vec![(0, 0, 0)],
        };
        let silent = ClassPairFiring {
            input_coords: Vec::new(),
            output_coords: Vec::new(),
        };
        let wrong = ClassPairFiring {
            input_coords: vec![(0, 0, 0)],
            output_coords: vec![(1, 0, 0)],
        };
        assert!(classifier_reports_correct(&[
            answered.clone(),
            silent.clone()
        ]));
        assert!(!classifier_reports_correct(&[silent]));
        assert!(!classifier_reports_correct(&[answered, wrong]));
    }

    #[test]
    fn legacy_affect_fields_are_ignored() {
        let json = serde_json::json!({
            "classifier_id": "clf-1",
            "name": "demo",
            "parent_region_id": "region",
            "coordinates_3d": [0, 0, 0],
            "sample_memory_id": "smem",
            "class_memory_id": "cmem",
            "error_area_id": "err",
            "pain_area_id": "legacy-pain",
            "properties": {}
        });
        let classifier: Classifier = serde_json::from_value(json).expect("classifier");
        assert!(classifier.correct_area_id.is_empty());
        assert!(classifier.reward_modulator_ids.is_empty());
        assert!(!classifier
            .owned_area_ids()
            .contains(&"legacy-pain".to_string()));
    }

    #[test]
    fn required_mappings_cover_each_sample_class_and_the_associative_edge() {
        let mut classifier = sample();
        classifier
            .attach_sample("sample-b".to_string())
            .expect("second sample");
        classifier
            .attach_class("class-b".to_string(), "out-b".to_string())
            .expect("second class");
        let mappings = classifier.required_mappings();
        assert_eq!(mappings.len(), 5);
        assert!(mappings.iter().any(|mapping| {
            mapping.src_area_id == "sample-b"
                && mapping.dst_area_id == "smem"
                && mapping.morphology_id == CLASSIFIER_SAMPLE_MORPHOLOGY
        }));
        assert!(mappings.iter().any(|mapping| {
            mapping.src_area_id == "class-b"
                && mapping.dst_area_id == "cmem"
                && mapping.morphology_id == CLASSIFIER_CLASS_MORPHOLOGY
        }));
        assert!(mappings
            .iter()
            .any(|mapping| { mapping.morphology_id == CLASSIFIER_ASSOCIATIVE_MORPHOLOGY }));
    }

    #[test]
    fn removing_one_class_keeps_the_other_output() {
        let mut classifier = sample();
        classifier
            .attach_class("class-b".to_string(), "out-b".to_string())
            .expect("second class");
        assert_eq!(
            classifier.detach_class("class-a"),
            Some("out-a".to_string())
        );
        assert!(classifier.binding_for_class("class-a").is_none());
        assert_eq!(
            classifier
                .binding_for_class("class-b")
                .map(|binding| binding.class_output_area_id.as_str()),
            Some("out-b")
        );
        assert!(classifier.owns_area("out-b"));
        assert!(!classifier.owns_area("out-a"));
    }

    #[test]
    fn duplicate_class_mapping_is_rejected() {
        let mut classifier = sample();
        let result = classifier.attach_class("class-a".to_string(), "other".to_string());
        assert!(result.is_err());
        assert_eq!(classifier.class_bindings.len(), 1);
    }

    #[test]
    fn combo_class_with_one_neuron_per_area_is_not_an_error() {
        let pairs = vec![
            ClassPairFiring {
                input_coords: vec![(0, 0, 1)],
                output_coords: vec![(0, 0, 1)],
            },
            ClassPairFiring {
                input_coords: vec![(0, 0, 0)],
                output_coords: vec![(0, 0, 0)],
            },
            ClassPairFiring {
                input_coords: vec![(0, 0, 3)],
                output_coords: vec![(0, 0, 3)],
            },
            ClassPairFiring {
                input_coords: vec![(0, 0, 2)],
                output_coords: vec![(0, 0, 2)],
            },
        ];
        assert!(!classifier_reports_error(&pairs));
    }

    #[test]
    fn two_neurons_in_one_class_output_is_an_error() {
        let pairs = vec![ClassPairFiring {
            input_coords: vec![(0, 0, 1)],
            output_coords: vec![(0, 0, 1), (0, 0, 4)],
        }];
        assert!(classifier_reports_error(&pairs));
    }

    #[test]
    fn mismatch_is_per_class_area_and_silent_input_is_not_mismatch() {
        assert!(class_output_reports_error(&ClassPairFiring {
            input_coords: vec![(0, 0, 1)],
            output_coords: vec![(0, 0, 2)],
        }));
        assert!(!class_output_reports_error(&ClassPairFiring {
            input_coords: Vec::new(),
            output_coords: vec![(0, 0, 2)],
        }));
    }
}
