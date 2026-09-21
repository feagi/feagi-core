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

/// First-class classifier record persisted in the genome.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Classifier {
    /// UUID string, genome map key.
    pub classifier_id: String,
    pub name: String,
    /// Region that contains this assembly. Classifiers are not regions.
    pub parent_region_id: String,
    pub coordinates_3d: [i32; 3],
    /// Referenced inputs. Cleared when that area is deleted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kernel_area_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class_area_id: Option<String>,
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
            || self.binding_for_field(area_id).is_some()
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
            kernel_area_id: Some("kernel".to_string()),
            class_area_id: Some("class".to_string()),
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
}
