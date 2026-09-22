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
the shared kernel memory, with its own detection twin.
*/

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class_area_id: Option<String>,
    /// Scanner-mode label volume. Width and height match each mapped field. Depth is the class count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mask_area_id: Option<String>,
    /// Scanner-mode kernel `[x, y, z]`. Z must equal each mapped field's depth.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kernel_size: Option<[u32; 3]>,
    /// Field scans. Empty until Classifier mappings are drawn.
    #[serde(default)]
    pub fields: Vec<ClassifierField>,
    /// Owned internals. Deleting kernel or class memory deletes the classifier.
    pub kernel_memory_id: String,
    pub class_memory_id: String,
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
    ) -> Result<bool, String> {
        let previous_mode = self.training_mode;
        let previous_size = self.kernel_size;
        match mode {
            ClassifierTrainingMode::Kernel => {
                let kernel = kernel_area_id.ok_or_else(|| "kernel_area_id required".to_string())?;
                let class = class_area_id.ok_or_else(|| "class_area_id required".to_string())?;
                self.kernel_area_id = Some(required_area_id(kernel, "kernel_area_id")?);
                self.class_area_id = Some(required_area_id(class, "class_area_id")?);
                self.mask_area_id = None;
                self.kernel_size = None;
            }
            ClassifierTrainingMode::Scanner => {
                let mask = mask_area_id.ok_or_else(|| "mask_area_id required".to_string())?;
                let size = kernel_size.ok_or_else(|| "kernel_size required".to_string())?;
                validate_kernel_size(size)?;
                self.mask_area_id = Some(required_area_id(mask, "mask_area_id")?);
                self.kernel_size = Some(size);
                self.kernel_area_id = None;
                self.class_area_id = None;
            }
        }
        self.training_mode = mode;
        Ok(previous_mode != mode
            || (mode == ClassifierTrainingMode::Scanner && previous_size != self.kernel_size))
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

/// Every kernel axis must be at least one voxel.
pub fn validate_kernel_size(size: [u32; 3]) -> Result<(), String> {
    if size[0] == 0 || size[1] == 0 || size[2] == 0 {
        return Err("kernel_size axes must be greater than zero".to_string());
    }
    Ok(())
}

/// Scanner kernel Z matches the image depth, and the mask shares the image's width and height.
pub fn validate_scanner_field(
    kernel_size: [u32; 3],
    field: [u32; 3],
    mask: [u32; 3],
) -> Result<(), String> {
    validate_kernel_size(kernel_size)?;
    if field[0] == 0 || field[1] == 0 || field[2] == 0 {
        return Err("field dimensions must be greater than zero".to_string());
    }
    if mask[2] == 0 {
        return Err("mask depth must be greater than zero".to_string());
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
            kernel_size: None,
            fields: Vec::new(),
            kernel_memory_id: "kmem".to_string(),
            class_memory_id: "cmem".to_string(),
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
            )
            .expect("scanner inputs");
        assert!(changed);
        assert_eq!(classifier.training_mode, ClassifierTrainingMode::Scanner);
        assert!(classifier.kernel_area_id.is_none());
        assert!(classifier.class_area_id.is_none());
        assert_eq!(classifier.mask_area_id.as_deref(), Some("mask"));
        assert_eq!(classifier.kernel_size, Some([8, 8, 3]));
        assert!(classifier.references_input("mask"));
        let same = classifier
            .apply_training_inputs(
                ClassifierTrainingMode::Scanner,
                None,
                None,
                Some("mask".to_string()),
                Some([8, 8, 3]),
            )
            .expect("same scanner geometry");
        assert!(!same);
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
            )
            .expect("scanner");
        classifier
            .apply_training_inputs(
                ClassifierTrainingMode::Kernel,
                Some("kernel".to_string()),
                Some("class".to_string()),
                None,
                None,
            )
            .expect("kernel");
        assert!(classifier.mask_area_id.is_none());
        assert!(classifier.kernel_size.is_none());
        assert_eq!(classifier.kernel_area_id.as_deref(), Some("kernel"));
    }

    #[test]
    fn scanner_field_must_match_mask_and_kernel_depth() {
        assert!(validate_scanner_field([8, 8, 3], [256, 128, 3], [256, 128, 10]).is_ok());
        assert!(validate_scanner_field([8, 8, 1], [256, 128, 3], [256, 128, 10]).is_err());
        assert!(validate_scanner_field([8, 8, 3], [256, 128, 3], [200, 128, 10]).is_err());
    }
}
