// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
First-class genome classifier assembly.

Classifiers are stored under the top-level genome key `classifiers`, parallel
to `brain_regions`. A classifier is not a brain region and is not exportable
as a circuit. It records the assembly's properties, owned internals, and
referenced input areas so neuroembryogenesis and area/mapping edits stay
aligned.

Each field binding is one Classifier mapping: an interconnect area scanning
the shared kernel memory, with its own class output.
*/

use crate::neuron_voxels::class_potential::validate_class_count;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Morphology used for kernel → kernel-memory encode.
pub const CLASSIFIER_KERNEL_MORPHOLOGY: &str = "episodic_memory";
/// Morphology used for class → class-memory encode.
pub const CLASSIFIER_CLASS_MORPHOLOGY: &str = "episodic_memory";
/// Morphology used for kernel-memory → class-memory bind.
pub const CLASSIFIER_ASSOCIATIVE_MORPHOLOGY: &str = "associative_memory";
/// Morphology used for field → kernel-memory scan.
pub const CLASSIFIER_SCAN_MORPHOLOGY: &str = "episodic_scan";

/// Directed mapping owned by a classifier assembly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifierMapping {
    pub src_area_id: String,
    pub dst_area_id: String,
    pub morphology_id: String,
}

/// One field area scanning this classifier, and the twin that shows its detections.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClassifierField {
    pub field_area_id: String,
    pub scan_twin_id: String,
}

/// How a classifier learns. Recall uses the same kernel geometry as training.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ClassifierTrainingMode {
    /// One kernel sample and one class sample per burst.
    #[default]
    Kernel,
    /// Slide `kernel_size` across each mapped field and label it from the mask.
    Scanner,
}

impl ClassifierTrainingMode {
    /// Genome and API spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Kernel => "kernel",
            Self::Scanner => "scanner",
        }
    }

    /// Parse a stored mode. Absent values are not accepted here.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "kernel" => Ok(Self::Kernel),
            "scanner" => Ok(Self::Scanner),
            other => Err(format!(
                "training_mode must be \"kernel\" or \"scanner\", got \"{other}\""
            )),
        }
    }
}

/// First-class classifier record persisted in the genome.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Classifier {
    /// UUID string, genome map key.
    pub classifier_id: String,
    pub name: String,
    /// Region that contains this assembly. Classifiers are not regions.
    pub parent_region_id: String,
    pub coordinates_3d: [i32; 3],
    /// Kernel mode ingests one sample per burst. Scanner mode learns from field windows.
    /// Genomes saved before modes existed load as kernel mode.
    #[serde(default)]
    pub training_mode: ClassifierTrainingMode,
    /// Referenced inputs. Cleared when that area is deleted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kernel_area_id: Option<String>,
    /// Kernel-mode class input. Must be `1×1×n`: each depth voxel is one class channel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class_area_id: Option<String>,
    /// Scanner-mode label plane, `W×H×1`. Width and height match each mapped field.
    /// Each pixel's potential is `(class_id + 1) / class_count`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mask_area_id: Option<String>,
    /// Scanner-mode class count. Decodes mask potentials and encodes detection twins.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class_count: Option<u32>,
    /// Scanner-mode kernel `[x, y, z]`. Z must equal each mapped field's depth.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kernel_size: Option<[u32; 3]>,
    /// Field scans. Empty until Classifier mappings are drawn.
    #[serde(default)]
    pub fields: Vec<ClassifierField>,
    /// Owned internals. Deleting kernel or class memory deletes the classifier.
    pub kernel_memory_id: String,
    pub class_memory_id: String,
    /// Per-scanning-instance pain and pleasure on the associative mapping.
    /// Genomes saved before reward training existed stay off.
    #[serde(default)]
    pub reward_training: bool,
    /// Correct-answer area. Scanner mode matches each detection twin.
    /// Kernel mode matches the class area. Empty until the user selects one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer_feedback_area_id: Option<String>,
    /// Hidden area whose firing is this classifier's pain signal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pain_area_id: Option<String>,
    /// Hidden area whose firing is this classifier's pleasure signal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pleasure_area_id: Option<String>,
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
    /// Kernel memory and class memory. Deleting either deletes the assembly.
    pub fn assembly_core_ids(&self) -> Vec<String> {
        vec![self.kernel_memory_id.clone(), self.class_memory_id.clone()]
    }

    /// Owned internals deleted with the classifier, including every field twin.
    pub fn owned_area_ids(&self) -> Vec<String> {
        let mut owned = self.assembly_core_ids();
        for field in &self.fields {
            if !field.scan_twin_id.is_empty() {
                owned.push(field.scan_twin_id.clone());
            }
        }
        if let Some(pain_area_id) = &self.pain_area_id {
            if !pain_area_id.is_empty() {
                owned.push(pain_area_id.clone());
            }
        }
        if let Some(pleasure_area_id) = &self.pleasure_area_id {
            if !pleasure_area_id.is_empty() {
                owned.push(pleasure_area_id.clone());
            }
        }
        owned
    }

    /// Referenced input areas that may be cleared independently.
    pub fn input_area_ids(&self) -> Vec<String> {
        let mut inputs = Vec::new();
        if let Some(kernel) = &self.kernel_area_id {
            inputs.push(kernel.clone());
        }
        if let Some(class) = &self.class_area_id {
            inputs.push(class.clone());
        }
        if let Some(mask) = &self.mask_area_id {
            inputs.push(mask.clone());
        }
        for field in &self.fields {
            inputs.push(field.field_area_id.clone());
        }
        inputs
    }

    pub fn owns_assembly_core(&self, area_id: &str) -> bool {
        self.kernel_memory_id == area_id || self.class_memory_id == area_id
    }

    /// True when `area_id` is kernel memory, class memory, or one of the field twins.
    pub fn owns_area(&self, area_id: &str) -> bool {
        self.owns_assembly_core(area_id) || self.field_for_twin(area_id).is_some()
    }

    pub fn field_for_twin(&self, twin_id: &str) -> Option<&ClassifierField> {
        self.fields
            .iter()
            .find(|field| field.scan_twin_id == twin_id)
    }

    pub fn binding_for_field(&self, field_area_id: &str) -> Option<&ClassifierField> {
        self.fields
            .iter()
            .find(|field| field.field_area_id == field_area_id)
    }

    /// True when `area_id` is a referenced input of this classifier.
    pub fn references_input(&self, area_id: &str) -> bool {
        self.kernel_area_id.as_deref() == Some(area_id)
            || self.class_area_id.as_deref() == Some(area_id)
            || self.mask_area_id.as_deref() == Some(area_id)
            || self.binding_for_field(area_id).is_some()
    }

    /// Replace the training mode and its inputs. The other mode's slots are cleared.
    ///
    /// Returns true when long-term memory learned under the previous geometry must be dropped.
    pub fn apply_training_inputs(
        &mut self,
        mode: ClassifierTrainingMode,
        kernel_area_id: Option<String>,
        class_area_id: Option<String>,
        mask_area_id: Option<String>,
        kernel_size: Option<[u32; 3]>,
        class_count: Option<u32>,
    ) -> Result<bool, String> {
        let previous_mode = self.training_mode;
        let previous_size = self.kernel_size;
        let previous_class_count = self.class_count;
        match mode {
            ClassifierTrainingMode::Kernel => {
                let kernel = kernel_area_id.ok_or_else(|| "kernel_area_id required".to_string())?;
                let class = class_area_id.ok_or_else(|| "class_area_id required".to_string())?;
                self.kernel_area_id = Some(required_area_id(kernel, "kernel_area_id")?);
                self.class_area_id = Some(required_area_id(class, "class_area_id")?);
                self.mask_area_id = None;
                self.kernel_size = None;
                self.class_count = None;
            }
            ClassifierTrainingMode::Scanner => {
                let mask = mask_area_id.ok_or_else(|| "mask_area_id required".to_string())?;
                let size = kernel_size.ok_or_else(|| "kernel_size required".to_string())?;
                let count = class_count.ok_or_else(|| "class_count required".to_string())?;
                validate_kernel_size(size)?;
                validate_class_count(count).map_err(|e| e.to_string())?;
                self.mask_area_id = Some(required_area_id(mask, "mask_area_id")?);
                self.kernel_size = Some(size);
                self.class_count = Some(count);
                self.kernel_area_id = None;
                self.class_area_id = None;
            }
        }
        self.training_mode = mode;
        Ok(previous_mode != mode
            || (mode == ClassifierTrainingMode::Scanner
                && (previous_size != self.kernel_size || previous_class_count != self.class_count)))
    }

    /// Apply classifier-level edit. Does not replace owned internals or field bindings.
    pub fn apply_assembly_update(
        &mut self,
        name: Option<String>,
        coordinates_3d: Option<[i32; 3]>,
        parent_region_id: Option<String>,
        kernel_area_id: Option<String>,
        class_area_id: Option<String>,
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
        if let Some(kernel_area_id) = kernel_area_id {
            self.kernel_area_id = Some(required_area_id(kernel_area_id, "kernel_area_id")?);
        }
        if let Some(class_area_id) = class_area_id {
            self.class_area_id = Some(required_area_id(class_area_id, "class_area_id")?);
        }
        Ok(())
    }

    /// Apply classifier-level metadata. Does not change owned internals or input slots.
    pub fn apply_metadata_update(
        &mut self,
        name: Option<String>,
        coordinates_3d: Option<[i32; 3]>,
    ) -> Result<(), String> {
        self.apply_assembly_update(name, coordinates_3d, None, None, None)
    }

    /// Mappings this classifier requires given its current inputs.
    pub fn required_mappings(&self) -> Vec<ClassifierMapping> {
        let mut mappings = Vec::new();
        if let Some(kernel) = &self.kernel_area_id {
            mappings.push(ClassifierMapping {
                src_area_id: kernel.clone(),
                dst_area_id: self.kernel_memory_id.clone(),
                morphology_id: CLASSIFIER_KERNEL_MORPHOLOGY.to_string(),
            });
        }
        if let Some(class) = &self.class_area_id {
            mappings.push(ClassifierMapping {
                src_area_id: class.clone(),
                dst_area_id: self.class_memory_id.clone(),
                morphology_id: CLASSIFIER_CLASS_MORPHOLOGY.to_string(),
            });
        }
        mappings.push(ClassifierMapping {
            src_area_id: self.kernel_memory_id.clone(),
            dst_area_id: self.class_memory_id.clone(),
            morphology_id: CLASSIFIER_ASSOCIATIVE_MORPHOLOGY.to_string(),
        });
        for field in &self.fields {
            mappings.push(ClassifierMapping {
                src_area_id: field.field_area_id.clone(),
                dst_area_id: self.kernel_memory_id.clone(),
                morphology_id: CLASSIFIER_SCAN_MORPHOLOGY.to_string(),
            });
        }
        mappings
    }

    /// Drop an input reference when that area is deleted.
    pub fn clear_input(&mut self, area_id: &str) {
        if self.kernel_area_id.as_deref() == Some(area_id) {
            self.kernel_area_id = None;
        }
        if self.class_area_id.as_deref() == Some(area_id) {
            self.class_area_id = None;
        }
        if self.mask_area_id.as_deref() == Some(area_id) {
            self.mask_area_id = None;
        }
        if self.answer_feedback_area_id.as_deref() == Some(area_id) {
            self.answer_feedback_area_id = None;
        }
        if self.learn_area_id.as_deref() == Some(area_id) {
            self.learn_area_id = None;
        }
        if self.confidence_area_id.as_deref() == Some(area_id) {
            self.confidence_area_id = None;
        }
        self.fields.retain(|field| field.field_area_id != area_id);
    }

    /// Remove one field binding. Returns the removed twin id.
    pub fn detach_field(&mut self, field_area_id: &str) -> Option<String> {
        let position = self
            .fields
            .iter()
            .position(|field| field.field_area_id == field_area_id)?;
        Some(self.fields.remove(position).scan_twin_id)
    }

    /// Remove the binding whose twin is `twin_id`. Returns the field area id.
    pub fn detach_twin(&mut self, twin_id: &str) -> Option<String> {
        let position = self
            .fields
            .iter()
            .position(|field| field.scan_twin_id == twin_id)?;
        Some(self.fields.remove(position).field_area_id)
    }

    pub fn attach_field(
        &mut self,
        field_area_id: String,
        scan_twin_id: String,
    ) -> Result<(), String> {
        let field_area_id = required_area_id(field_area_id, "field_area_id")?;
        let scan_twin_id = required_area_id(scan_twin_id, "scan_twin_id")?;
        if self.binding_for_field(&field_area_id).is_some() {
            return Err(format!(
                "field_area_id {field_area_id} is already mapped to this classifier"
            ));
        }
        self.fields.push(ClassifierField {
            field_area_id,
            scan_twin_id,
        });
        Ok(())
    }

    /// Bind or clear kernel/class inputs from a mapping change. Field scans are attached explicitly.
    pub fn apply_mapping_change(
        &mut self,
        src_area_id: &str,
        dst_area_id: &str,
        morphology_id: &str,
        removed: bool,
    ) -> bool {
        if dst_area_id == self.kernel_memory_id && morphology_id == CLASSIFIER_KERNEL_MORPHOLOGY {
            self.kernel_area_id = if removed {
                None
            } else {
                Some(src_area_id.to_string())
            };
            return true;
        }
        if dst_area_id == self.class_memory_id && morphology_id == CLASSIFIER_CLASS_MORPHOLOGY {
            self.class_area_id = if removed {
                None
            } else {
                Some(src_area_id.to_string())
            };
            return true;
        }
        false
    }
}

/// Kernel-mode class input. Width and height are 1 so depth index `z` is twin class `z`.
pub fn validate_kernel_class_area_shape(dimensions: [u32; 3]) -> Result<(), String> {
    if dimensions[0] != 1 || dimensions[1] != 1 || dimensions[2] == 0 {
        return Err(
            "class area must be 1x1xn so each depth voxel maps to one detection class".to_string(),
        );
    }
    Ok(())
}

/// Every kernel axis must be at least one voxel.
pub fn validate_kernel_size(size: [u32; 3]) -> Result<(), String> {
    if size[0] == 0 || size[1] == 0 || size[2] == 0 {
        return Err("kernel_size axes must be greater than zero".to_string());
    }
    Ok(())
}

/// Scanner class output: one layer over the field, class carried as potential.
pub fn detection_twin_shape(field: [u32; 3]) -> [u32; 3] {
    [field[0], field[1], 1]
}

/// Kernel mode compares the whole field to the kernel. The field must be that size.
pub fn validate_kernel_field(kernel: [u32; 3], field: [u32; 3]) -> Result<(), String> {
    if kernel[0] == 0 || kernel[1] == 0 || kernel[2] == 0 {
        return Err("kernel area dimensions must be greater than zero".to_string());
    }
    if kernel != field {
        return Err(format!(
            "kernel mode field must match the kernel area ({}x{}x{}); got {}x{}x{}",
            kernel[0], kernel[1], kernel[2], field[0], field[1], field[2]
        ));
    }
    Ok(())
}

/// Class output geometry for one training mode.
///
/// Kernel mode is `1×1×class_count`, the same shape as the class input.
/// Scanner mode is `field_w × field_h × 1`.
pub fn class_output_shape(
    mode: ClassifierTrainingMode,
    field: [u32; 3],
    class_count: u32,
) -> Result<[u32; 3], String> {
    match mode {
        ClassifierTrainingMode::Kernel => {
            validate_class_count(class_count).map_err(|error| error.to_string())?;
            Ok([1, 1, class_count])
        }
        ClassifierTrainingMode::Scanner => {
            if field[0] == 0 || field[1] == 0 {
                return Err("scanner class output needs a field width and height".to_string());
            }
            Ok(detection_twin_shape(field))
        }
    }
}

/// Scanner outputs forward the class potential. Kernel outputs fire depth `z`.
pub fn class_output_forwards_potential(mode: ClassifierTrainingMode) -> bool {
    matches!(mode, ClassifierTrainingMode::Scanner)
}

/// Visible name of one class output. A second field includes that field's name.
pub fn class_output_area_name(
    classifier_name: &str,
    field_name: &str,
    output_count: usize,
) -> String {
    if output_count <= 1 {
        format!("{classifier_name} class output")
    } else {
        format!("{classifier_name} {field_name} class output")
    }
}

/// Scanner kernel Z matches the image depth. The mask is one layer with the image's width and height.
pub fn validate_scanner_field(
    kernel_size: [u32; 3],
    field: [u32; 3],
    mask: [u32; 3],
) -> Result<(), String> {
    validate_kernel_size(kernel_size)?;
    if field[0] == 0 || field[1] == 0 || field[2] == 0 {
        return Err("field dimensions must be greater than zero".to_string());
    }
    if mask[2] != 1 {
        return Err("mask must be one layer deep; class ids are carried as potential".to_string());
    }
    if kernel_size[0] > field[0] || kernel_size[1] > field[1] {
        return Err("kernel_size does not fit the field".to_string());
    }
    if kernel_size[2] != field[2] {
        return Err("kernel depth must equal the field depth".to_string());
    }
    if mask[0] != field[0] || mask[1] != field[1] {
        return Err("mask width and height must equal the field".to_string());
    }
    Ok(())
}

/// One class for the whole image: two axes are 1 and the remaining axis is the class count.
pub fn is_whole_image_class_shape(feedback: [u32; 3], class_count: u32) -> bool {
    if class_count == 0 {
        return false;
    }
    let ones = feedback.iter().filter(|axis| **axis == 1).count();
    let volume = u64::from(feedback[0])
        .saturating_mul(u64::from(feedback[1]))
        .saturating_mul(u64::from(feedback[2]));
    ones >= 2 && volume == u64::from(class_count)
}

/// Answer feedback must match the classifier output.
///
/// Kernel mode compares the class area. Scanner mode compares each detection
/// twin (`W×H×1`, class as potential) or accepts a whole-image class of
/// `class_count` channels. With no twin yet, the mask is that output.
pub fn validate_answer_feedback_shape(
    mode: ClassifierTrainingMode,
    feedback: [u32; 3],
    reference: [u32; 3],
    output_shapes: &[[u32; 3]],
    class_count: u32,
) -> Result<(), String> {
    if feedback[0] == 0 || feedback[1] == 0 || feedback[2] == 0 {
        return Err("answer feedback dimensions must be greater than zero".to_string());
    }
    match mode {
        ClassifierTrainingMode::Kernel => {
            if feedback != reference {
                return Err("answer feedback dimensions must match the class area".to_string());
            }
        }
        ClassifierTrainingMode::Scanner => {
            if is_whole_image_class_shape(feedback, class_count) {
                return Ok(());
            }
            let shapes = if output_shapes.is_empty() {
                std::slice::from_ref(&reference)
            } else {
                output_shapes
            };
            if shapes.iter().any(|shape| *shape != feedback) {
                return Err(
                    "answer feedback dimensions must match the detection output or a whole-image class"
                        .to_string(),
                );
            }
        }
    }
    Ok(())
}

/// Mapping rule written for one classifier-owned edge.
///
/// Only `associative_memory` is plastic; its window is the kernel memory's temporal depth.
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
            training_mode: ClassifierTrainingMode::Kernel,
            kernel_area_id: Some("kernel".to_string()),
            class_area_id: Some("class".to_string()),
            mask_area_id: None,
            class_count: None,
            kernel_size: None,
            fields: Vec::new(),
            kernel_memory_id: "kmem".to_string(),
            class_memory_id: "cmem".to_string(),
            reward_training: false,
            answer_feedback_area_id: None,
            pain_area_id: None,
            pleasure_area_id: None,
            answer_latency_bursts: 0,
            learn_area_id: None,
            confidence_area_id: None,
            properties: HashMap::new(),
        };
        classifier
            .attach_field("field".to_string(), "twin".to_string())
            .expect("first field");
        classifier
    }

    #[test]
    fn required_mappings_cover_shared_edges_and_each_field() {
        let mut classifier = sample();
        classifier
            .attach_field("field-b".to_string(), "twin-b".to_string())
            .expect("second field");
        let mappings = classifier.required_mappings();
        assert_eq!(mappings.len(), 5);
        assert!(mappings.iter().any(|m| {
            m.src_area_id == "field"
                && m.dst_area_id == "kmem"
                && m.morphology_id == CLASSIFIER_SCAN_MORPHOLOGY
        }));
        assert!(mappings.iter().any(|m| m.src_area_id == "field-b"));
    }

    #[test]
    fn deleting_one_field_keeps_the_other_eye() {
        let mut classifier = sample();
        classifier
            .attach_field("field-b".to_string(), "twin-b".to_string())
            .expect("second field");
        assert_eq!(classifier.detach_field("field"), Some("twin".to_string()));
        assert!(classifier.binding_for_field("field").is_none());
        assert_eq!(
            classifier
                .binding_for_field("field-b")
                .map(|f| f.scan_twin_id.as_str()),
            Some("twin-b")
        );
        assert!(classifier.owns_assembly_core("kmem"));
        assert!(!classifier.owns_area("twin"));
        assert!(classifier.owns_area("twin-b"));
    }

    #[test]
    fn duplicate_field_mapping_is_rejected() {
        let mut classifier = sample();
        let result = classifier.attach_field("field".to_string(), "other-twin".to_string());
        assert!(result.is_err());
        assert_eq!(classifier.fields.len(), 1);
    }

    #[test]
    fn metadata_update_renames_without_touching_areas() {
        let mut classifier = sample();
        classifier
            .apply_metadata_update(Some("  renamed  ".to_string()), Some([9, 8, 7]))
            .expect("valid metadata");
        assert_eq!(classifier.name, "renamed");
        assert_eq!(classifier.coordinates_3d, [9, 8, 7]);
        assert_eq!(classifier.kernel_memory_id, "kmem");
        assert_eq!(classifier.fields[0].scan_twin_id, "twin");
        assert_eq!(classifier.kernel_area_id.as_deref(), Some("kernel"));
    }

    #[test]
    fn assembly_update_retargets_kernel_and_class_only() {
        let mut classifier = sample();
        classifier
            .apply_assembly_update(
                None,
                None,
                Some("other-region".to_string()),
                Some("kernel2".to_string()),
                Some("class2".to_string()),
            )
            .expect("valid assembly update");
        assert_eq!(classifier.parent_region_id, "other-region");
        assert_eq!(classifier.kernel_area_id.as_deref(), Some("kernel2"));
        assert_eq!(classifier.class_area_id.as_deref(), Some("class2"));
        assert_eq!(classifier.fields[0].field_area_id, "field");
    }

    #[test]
    fn scanner_mode_clears_kernel_inputs_and_reports_geometry_change() {
        let mut classifier = sample();
        let changed = classifier
            .apply_training_inputs(
                ClassifierTrainingMode::Scanner,
                None,
                None,
                Some("mask".to_string()),
                Some([8, 8, 3]),
                Some(19),
            )
            .expect("scanner inputs");
        assert!(changed);
        assert_eq!(classifier.training_mode, ClassifierTrainingMode::Scanner);
        assert!(classifier.kernel_area_id.is_none());
        assert!(classifier.class_area_id.is_none());
        assert_eq!(classifier.mask_area_id.as_deref(), Some("mask"));
        assert_eq!(classifier.kernel_size, Some([8, 8, 3]));
        assert_eq!(classifier.class_count, Some(19));
        assert!(classifier.references_input("mask"));
        let same = classifier
            .apply_training_inputs(
                ClassifierTrainingMode::Scanner,
                None,
                None,
                Some("mask".to_string()),
                Some([8, 8, 3]),
                Some(19),
            )
            .expect("same scanner geometry");
        assert!(!same);
        let recounted = classifier
            .apply_training_inputs(
                ClassifierTrainingMode::Scanner,
                None,
                None,
                Some("mask".to_string()),
                Some([8, 8, 3]),
                Some(6),
            )
            .expect("new class count");
        assert!(recounted, "a new class count changes what learned ids mean");
    }

    #[test]
    fn scanner_mode_requires_a_valid_class_count() {
        let mut classifier = sample();
        for bad in [None, Some(0)] {
            assert!(classifier
                .apply_training_inputs(
                    ClassifierTrainingMode::Scanner,
                    None,
                    None,
                    Some("mask".to_string()),
                    Some([8, 8, 3]),
                    bad,
                )
                .is_err());
        }
        assert_eq!(classifier.training_mode, ClassifierTrainingMode::Kernel);
    }

    #[test]
    fn kernel_mode_clears_scanner_inputs() {
        let mut classifier = sample();
        classifier
            .apply_training_inputs(
                ClassifierTrainingMode::Scanner,
                None,
                None,
                Some("mask".to_string()),
                Some([2, 2, 1]),
                Some(4),
            )
            .expect("scanner");
        classifier
            .apply_training_inputs(
                ClassifierTrainingMode::Kernel,
                Some("kernel".to_string()),
                Some("class".to_string()),
                None,
                None,
                None,
            )
            .expect("kernel");
        assert!(classifier.mask_area_id.is_none());
        assert!(classifier.kernel_size.is_none());
        assert!(classifier.class_count.is_none());
        assert_eq!(classifier.kernel_area_id.as_deref(), Some("kernel"));
    }

    #[test]
    fn detection_twin_is_one_layer_over_the_field() {
        assert_eq!(detection_twin_shape([256, 128, 3]), [256, 128, 1]);
    }

    #[test]
    fn class_output_shape_follows_training_mode() {
        assert_eq!(
            class_output_shape(ClassifierTrainingMode::Kernel, [13, 13, 3], 10).unwrap(),
            [1, 1, 10]
        );
        assert_eq!(
            class_output_shape(ClassifierTrainingMode::Scanner, [13, 13, 3], 10).unwrap(),
            [13, 13, 1]
        );
        assert!(validate_kernel_field([13, 13, 3], [13, 13, 3]).is_ok());
        assert!(validate_kernel_field([13, 13, 3], [28, 28, 3]).is_err());
        assert_eq!(
            class_output_area_name("MNIST classifier", "MNIST kernel", 1),
            "MNIST classifier class output"
        );
        assert_eq!(
            class_output_area_name("MNIST classifier", "MNIST kernel", 2),
            "MNIST classifier MNIST kernel class output"
        );
        assert!(!class_output_forwards_potential(
            ClassifierTrainingMode::Kernel
        ));
        assert!(class_output_forwards_potential(
            ClassifierTrainingMode::Scanner
        ));
    }

    #[test]
    fn answer_feedback_matches_class_area_or_detection_output() {
        assert!(validate_answer_feedback_shape(
            ClassifierTrainingMode::Kernel,
            [1, 1, 4],
            [1, 1, 4],
            &[],
            4,
        )
        .is_ok());
        assert!(validate_answer_feedback_shape(
            ClassifierTrainingMode::Kernel,
            [8, 8, 4],
            [1, 1, 4],
            &[],
            4,
        )
        .is_err());
        assert!(validate_answer_feedback_shape(
            ClassifierTrainingMode::Scanner,
            [16, 16, 1],
            [16, 16, 1],
            &[[16, 16, 1]],
            4,
        )
        .is_ok());
        assert!(validate_answer_feedback_shape(
            ClassifierTrainingMode::Scanner,
            [16, 16, 1],
            [16, 16, 1],
            &[[16, 16, 1], [8, 8, 1]],
            4,
        )
        .is_err());
        assert!(
            validate_answer_feedback_shape(
                ClassifierTrainingMode::Scanner,
                [16, 16, 4],
                [16, 16, 1],
                &[[16, 16, 1]],
                4,
            )
            .is_err(),
            "a one-hot class volume no longer matches a potential-coded twin"
        );
        assert!(validate_answer_feedback_shape(
            ClassifierTrainingMode::Scanner,
            [10, 1, 1],
            [16, 16, 1],
            &[[16, 16, 1]],
            10,
        )
        .is_ok());
    }

    #[test]
    fn kernel_class_area_must_be_one_by_one_by_n() {
        assert!(validate_kernel_class_area_shape([1, 1, 4]).is_ok());
        assert!(validate_kernel_class_area_shape([2, 7, 4]).is_err());
        assert!(validate_kernel_class_area_shape([4, 1, 1]).is_err());
        assert!(validate_kernel_class_area_shape([1, 1, 0]).is_err());
    }

    #[test]
    fn scanner_field_must_match_mask_and_kernel_depth() {
        assert!(validate_scanner_field([8, 8, 3], [256, 128, 3], [256, 128, 1]).is_ok());
        assert!(validate_scanner_field([8, 8, 1], [256, 128, 3], [256, 128, 1]).is_err());
        assert!(validate_scanner_field([8, 8, 3], [256, 128, 3], [200, 128, 1]).is_err());
        assert!(
            validate_scanner_field([8, 8, 3], [256, 128, 3], [256, 128, 10]).is_err(),
            "one-hot class masks are replaced by the single-layer potential mask"
        );
    }
}
