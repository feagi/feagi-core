/*!
Genome validation for FEAGI.

Validates genome structure, morphologies, parameters, and constraints.
Provides clear error messages for debugging.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use std::collections::HashSet;
use serde_json::Value;
use crate::{RuntimeGenome, MorphologyParameters};

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the genome is valid
    pub valid: bool,
    /// List of errors (blocking issues)
    pub errors: Vec<String>,
    /// List of warnings (non-blocking issues)
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a new valid result
    pub fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    /// Add an error
    pub fn add_error(&mut self, error: String) {
        self.valid = false;
        self.errors.push(error);
    }
    
    /// Add a warning
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
    
    /// Merge another validation result into this one
    pub fn merge(&mut self, other: ValidationResult) {
        if !other.valid {
            self.valid = false;
        }
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate a RuntimeGenome
pub fn validate_genome(genome: &RuntimeGenome) -> ValidationResult {
    let mut result = ValidationResult::new();
    
    // Validate metadata
    validate_metadata(genome, &mut result);
    
    // Validate cortical areas
    validate_cortical_areas(genome, &mut result);
    
    // Validate morphologies
    validate_morphologies(genome, &mut result);
    
    // Validate physiology
    validate_physiology(genome, &mut result);
    
    // Cross-validate (e.g., check references between sections)
    cross_validate(genome, &mut result);
    
    result
}

/// Auto-fix common genome issues (zero dimensions, zero per_voxel_neuron_cnt, missing physiology)
///
/// This function modifies the genome in-place to fix issues that can be automatically corrected.
/// Should be called before validation to prevent common user errors.
///
/// # Arguments
/// * `genome` - Mutable reference to genome to fix
///
/// # Returns
/// * Number of fixes applied
pub fn auto_fix_genome(genome: &mut RuntimeGenome) -> usize {
    use tracing::info;
    
    let mut fixes_applied = 0;
    
    // Fix missing or invalid physiology values
    if genome.physiology.simulation_timestep <= 0.0 {
        let default_timestep = crate::runtime::PhysiologyConfig::default().simulation_timestep;
        info!("🔧 AUTO-FIX: Invalid simulation_timestep {} → {} (default)", 
              genome.physiology.simulation_timestep, default_timestep);
        genome.physiology.simulation_timestep = default_timestep;
        fixes_applied += 1;
    }
    
    if genome.physiology.max_age == 0 {
        let default_age = crate::runtime::PhysiologyConfig::default().max_age;
        info!("🔧 AUTO-FIX: max_age 0 → {} (default)", default_age);
        genome.physiology.max_age = default_age;
        fixes_applied += 1;
    }
    
    // Fix missing or invalid quantization_precision
    if genome.physiology.quantization_precision.is_empty() {
        let default_precision = crate::runtime::default_quantization_precision();
        info!("🔧 AUTO-FIX: Missing quantization_precision → '{}' (default)", default_precision);
        genome.physiology.quantization_precision = default_precision;
        fixes_applied += 1;
    } else {
        // Normalize to canonical format
        use feagi_types::Precision;
        match Precision::from_str(&genome.physiology.quantization_precision) {
            Ok(precision) => {
                let canonical = precision.as_str().to_string();
                if genome.physiology.quantization_precision != canonical {
                    info!("🔧 AUTO-FIX: Quantization precision '{}' → '{}' (normalized)", 
                          genome.physiology.quantization_precision, canonical);
                    genome.physiology.quantization_precision = canonical;
                    fixes_applied += 1;
                }
            }
            Err(_) => {
                // Invalid precision - will be caught by validator
                let default_precision = crate::runtime::default_quantization_precision();
                info!("🔧 AUTO-FIX: Invalid quantization_precision '{}' → '{}' (default)", 
                      genome.physiology.quantization_precision, default_precision);
                genome.physiology.quantization_precision = default_precision;
                fixes_applied += 1;
            }
        }
    }
    
    for (cortical_id, area) in &mut genome.cortical_areas {
        // Fix zero dimensions
        if area.dimensions.width == 0 {
            info!("🔧 AUTO-FIX: Cortical area '{}' width 0 → 1", cortical_id);
            area.dimensions.width = 1;
            fixes_applied += 1;
        }
        if area.dimensions.height == 0 {
            info!("🔧 AUTO-FIX: Cortical area '{}' height 0 → 1", cortical_id);
            area.dimensions.height = 1;
            fixes_applied += 1;
        }
        if area.dimensions.depth == 0 {
            info!("🔧 AUTO-FIX: Cortical area '{}' depth 0 → 1", cortical_id);
            area.dimensions.depth = 1;
            fixes_applied += 1;
        }
        
        // Fix zero per_voxel_neuron_cnt
        if let Some(per_voxel_value) = area.properties.get_mut("per_voxel_neuron_cnt") {
            if let Some(0) = per_voxel_value.as_i64() {
                info!("🔧 AUTO-FIX: Cortical area '{}' per_voxel_neuron_cnt 0 → 1", cortical_id);
                *per_voxel_value = serde_json::Value::from(1);
                fixes_applied += 1;
            }
        }
    }
    
    if fixes_applied > 0 {
        info!("🔧 AUTO-FIX: Applied {} automatic corrections to genome", fixes_applied);
    }
    
    fixes_applied
}

/// Validate genome metadata
fn validate_metadata(genome: &RuntimeGenome, result: &mut ValidationResult) {
    if genome.metadata.genome_id.is_empty() {
        result.add_error("Genome ID is empty".to_string());
    }
    
    if genome.metadata.version.is_empty() {
        result.add_error("Genome version is empty".to_string());
    }
    
    if genome.metadata.version != "2.0" {
        result.add_warning(format!(
            "Genome version '{}' may not be fully supported (expected '2.0')",
            genome.metadata.version
        ));
    }
}

/// Validate cortical areas
fn validate_cortical_areas(genome: &RuntimeGenome, result: &mut ValidationResult) {
    if genome.cortical_areas.is_empty() {
        result.add_warning("Genome has no cortical areas defined".to_string());
        return;
    }
    
    for (cortical_id, area) in &genome.cortical_areas {
        // Validate cortical ID format (should be 6 characters)
        if cortical_id.len() != 6 {
            result.add_error(format!(
                "Cortical ID '{}' has invalid length {} (expected 6 characters)",
                cortical_id, cortical_id.len()
            ));
        }
        
        // Validate dimensions - AUTO-FIX zeros to 1
        if area.dimensions.width == 0 || area.dimensions.height == 0 || area.dimensions.depth == 0 {
            result.add_warning(format!(
                "AUTO-FIX: Cortical area '{}' has zero dimension(s): {}x{}x{} - will be corrected to minimum (1,1,1)",
                cortical_id, area.dimensions.width, area.dimensions.height, area.dimensions.depth
            ));
            // Note: Auto-fix happens in auto_fix_genome() - this just detects the issue
        }
        
        // Validate per_voxel_neuron_cnt
        if let Some(per_voxel) = area.properties.get("per_voxel_neuron_cnt").and_then(|v| v.as_i64()) {
            if per_voxel == 0 {
                result.add_warning(format!(
                    "AUTO-FIX: Cortical area '{}' has per_voxel_neuron_cnt=0 - will be corrected to 1",
                    cortical_id
                ));
            }
        }
        
        // Warn about very large dimensions
        let total_voxels = area.dimensions.width * area.dimensions.height * area.dimensions.depth;
        if total_voxels > 1_000_000 {
            result.add_warning(format!(
                "Cortical area '{}' has very large dimensions: {} total voxels",
                cortical_id, total_voxels
            ));
        }
        
        // Validate name
        if area.name.is_empty() {
            result.add_warning(format!(
                "Cortical area '{}' has empty name",
                cortical_id
            ));
        }
    }
}

/// Validate morphologies
fn validate_morphologies(genome: &RuntimeGenome, result: &mut ValidationResult) {
    if genome.morphologies.count() == 0 {
        result.add_warning("Genome has no morphologies defined".to_string());
        return;
    }
    
    // Check for required core morphologies
    let required_core = vec!["block_to_block", "projector"];
    for morph_id in required_core {
        if !genome.morphologies.contains(morph_id) {
            result.add_warning(format!(
                "Missing recommended core morphology: '{}'",
                morph_id
            ));
        }
    }
    
    for (morphology_id, morphology) in genome.morphologies.iter() {
        validate_single_morphology(morphology_id, morphology, result);
    }
}

/// Validate a single morphology
fn validate_single_morphology(
    morphology_id: &str,
    morphology: &crate::Morphology,
    result: &mut ValidationResult,
) {
    match &morphology.parameters {
        MorphologyParameters::Vectors { vectors } => {
            if vectors.is_empty() {
                result.add_error(format!(
                    "Morphology '{}' (vectors) has no vectors defined",
                    morphology_id
                ));
            }
            
            // Check for all-zero vectors (useless)
            for (i, vec) in vectors.iter().enumerate() {
                if vec[0] == 0 && vec[1] == 0 && vec[2] == 0 {
                    result.add_warning(format!(
                        "Morphology '{}' has zero vector at index {}: [{}, {}, {}]",
                        morphology_id, i, vec[0], vec[1], vec[2]
                    ));
                }
            }
        }
        
        MorphologyParameters::Patterns { patterns } => {
            if patterns.is_empty() {
                result.add_error(format!(
                    "Morphology '{}' (patterns) has no patterns defined",
                    morphology_id
                ));
            }
            
            for (i, pattern) in patterns.iter().enumerate() {
                if pattern[0].len() != 3 || pattern[1].len() != 3 {
                    result.add_error(format!(
                        "Morphology '{}' pattern {} has invalid structure (expected [src[3], dst[3]])",
                        morphology_id, i
                    ));
                }
            }
        }
        
        MorphologyParameters::Functions {} => {
            // Functions are built-in, no parameters to validate
        }
        
        MorphologyParameters::Composite { src_seed, src_pattern, mapper_morphology } => {
            // Validate src_seed
            if src_seed[0] == 0 || src_seed[1] == 0 || src_seed[2] == 0 {
                result.add_warning(format!(
                    "Morphology '{}' has zero dimension in src_seed: [{}, {}, {}]",
                    morphology_id, src_seed[0], src_seed[1], src_seed[2]
                ));
            }
            
            // Validate src_pattern
            if src_pattern.is_empty() {
                result.add_error(format!(
                    "Morphology '{}' (composite) has empty src_pattern",
                    morphology_id
                ));
            }
            
            // Validate mapper_morphology reference
            if mapper_morphology.is_empty() {
                result.add_error(format!(
                    "Morphology '{}' (composite) has empty mapper_morphology reference",
                    morphology_id
                ));
            }
        }
    }
}

/// Validate physiology parameters
fn validate_physiology(genome: &RuntimeGenome, result: &mut ValidationResult) {
    let phys = &genome.physiology;
    
    if phys.simulation_timestep <= 0.0 {
        result.add_error(format!(
            "Invalid simulation_timestep: {} (must be > 0.0)",
            phys.simulation_timestep
        ));
    }
    
    if phys.simulation_timestep > 1.0 {
        result.add_warning(format!(
            "Very large simulation_timestep: {} seconds (typical: 0.01-0.1)",
            phys.simulation_timestep
        ));
    }
    
    if phys.max_age == 0 {
        result.add_warning("max_age is 0 (neurons will never age)".to_string());
    }
    
    if phys.plasticity_queue_depth == 0 {
        result.add_warning("plasticity_queue_depth is 0 (no plasticity history)".to_string());
    }
    
    // Validate quantization_precision
    validate_quantization_precision(&phys.quantization_precision, result);
}

/// Validate quantization precision value
fn validate_quantization_precision(precision: &str, result: &mut ValidationResult) {
    use feagi_types::Precision;
    
    // Try to parse the precision string
    match Precision::from_str(precision) {
        Ok(parsed_precision) => {
            // Valid - log what was selected
            if precision != parsed_precision.as_str() {
                result.add_warning(format!(
                    "Quantization precision '{}' normalized to '{}'",
                    precision, parsed_precision.as_str()
                ));
            }
        }
        Err(_) => {
            result.add_error(format!(
                "Invalid quantization_precision: '{}' (must be 'fp32', 'fp16', or 'int8')",
                precision
            ));
        }
    }
}

/// Cross-validate references between genome sections
fn cross_validate(genome: &RuntimeGenome, result: &mut ValidationResult) {
    // Build morphology ID set for quick lookup
    let morphology_ids: HashSet<String> = genome.morphologies.morphology_ids().into_iter().collect();
    
    // Check if cortical areas reference morphologies in their properties
    for (cortical_id, area) in &genome.cortical_areas {
        if let Some(Value::Object(dstmap)) = area.properties.get("dstmap") {
            for (dest_area, rules) in dstmap {
                // Check if destination area exists
                if !genome.cortical_areas.contains_key(dest_area) {
                    result.add_error(format!(
                        "Cortical area '{}' references non-existent destination area '{}' in dstmap",
                        cortical_id, dest_area
                    ));
                }
                
                // Check morphology references in rules
                if let Value::Array(rules_array) = rules {
                    for rule in rules_array {
                        if let Value::Array(rule_array) = rule {
                            if let Some(Value::String(morph_id)) = rule_array.first() {
                                if !morphology_ids.contains(morph_id) {
                                    result.add_error(format!(
                                        "Cortical area '{}' references undefined morphology '{}' in dstmap rule",
                                        cortical_id, morph_id
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Validate brain region references
    for (region_id, region) in &genome.brain_regions {
        // Check if cortical areas in region exist
        for cortical_id in &region.cortical_areas {
            if !genome.cortical_areas.contains_key(cortical_id) {
                result.add_error(format!(
                    "Brain region '{}' references non-existent cortical area '{}'",
                    region_id, cortical_id
                ));
            }
        }
    }
    
    // Validate composite morphology references
    for (morphology_id, morphology) in genome.morphologies.iter() {
        if let MorphologyParameters::Composite { mapper_morphology, .. } = &morphology.parameters {
            if !morphology_ids.contains(mapper_morphology) {
                result.add_error(format!(
                    "Composite morphology '{}' references undefined mapper morphology '{}'",
                    morphology_id, mapper_morphology
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GenomeMetadata, PhysiologyConfig, GenomeSignatures, GenomeStats, MorphologyRegistry};
    use std::collections::HashMap;
    
    #[test]
    fn test_validate_empty_genome() {
        let genome = RuntimeGenome {
            metadata: GenomeMetadata {
                genome_id: "test".to_string(),
                genome_title: "Test".to_string(),
                genome_description: "".to_string(),
                version: "2.0".to_string(),
                timestamp: 0.0,
            },
            cortical_areas: HashMap::new(),
            brain_regions: HashMap::new(),
            morphologies: MorphologyRegistry::new(),
            physiology: PhysiologyConfig::default(),
            signatures: GenomeSignatures {
                genome: "0".to_string(),
                blueprint: "0".to_string(),
                physiology: "0".to_string(),
                morphologies: None,
            },
            stats: GenomeStats::default(),
        };
        
        let result = validate_genome(&genome);
        
        // Should have warnings about empty cortical areas and morphologies
        assert!(!result.warnings.is_empty());
        println!("Warnings: {:?}", result.warnings);
    }
    
    #[test]
    fn test_validate_valid_genome() {
        let mut genome = RuntimeGenome {
            metadata: GenomeMetadata {
                genome_id: "test_genome".to_string(),
                genome_title: "Test Genome".to_string(),
                genome_description: "Valid test genome".to_string(),
                version: "2.0".to_string(),
                timestamp: 1234567890.0,
            },
            cortical_areas: HashMap::new(),
            brain_regions: HashMap::new(),
            morphologies: MorphologyRegistry::new(),
            physiology: PhysiologyConfig::default(),
            signatures: GenomeSignatures {
                genome: "abc123".to_string(),
                blueprint: "def456".to_string(),
                physiology: "ghi789".to_string(),
                morphologies: None,
            },
            stats: GenomeStats::default(),
        };
        
        // Add a valid cortical area
        let area = feagi_types::CorticalArea::new(
            "test01".to_string(),
            0,
            "Test Area".to_string(),
            feagi_types::Dimensions::new(10, 10, 10),
            (0, 0, 0),
            feagi_types::AreaType::Custom,
        ).expect("Failed to create cortical area");
        genome.cortical_areas.insert("test01".to_string(), area);
        
        let result = validate_genome(&genome);
        
        // Should pass with only warnings (empty morphologies)
        println!("Errors: {:?}", result.errors);
        println!("Warnings: {:?}", result.warnings);
        
        // Genome is valid but has warnings
        assert!(result.errors.is_empty());
        assert!(!result.warnings.is_empty()); // Warning about no morphologies
    }
    
    #[test]
    fn test_validate_quantization_precision() {
        let mut genome = create_minimal_genome();
        
        // Test 1: Valid precision (fp32)
        genome.physiology.quantization_precision = "fp32".to_string();
        let result = validate_genome(&genome);
        assert!(result.errors.is_empty(), "fp32 should be valid");
        
        // Test 2: Valid precision (int8)
        genome.physiology.quantization_precision = "int8".to_string();
        let result = validate_genome(&genome);
        assert!(result.errors.is_empty(), "int8 should be valid");
        
        // Test 3: Valid but non-canonical (i8 → int8)
        genome.physiology.quantization_precision = "i8".to_string();
        let result = validate_genome(&genome);
        assert!(result.errors.is_empty(), "i8 should be valid");
        assert!(result.warnings.iter().any(|w| w.contains("normalized")), 
                "Should warn about normalization");
        
        // Test 4: Invalid precision
        genome.physiology.quantization_precision = "invalid".to_string();
        let result = validate_genome(&genome);
        assert!(!result.errors.is_empty(), "invalid should produce error");
        assert!(result.errors.iter().any(|e| e.contains("Invalid quantization_precision")),
                "Should have quantization error");
    }
    
    #[test]
    fn test_auto_fix_quantization_precision() {
        // Test 1: Missing precision (empty string)
        let mut genome = create_minimal_genome();
        genome.physiology.quantization_precision = "".to_string();
        
        let fixes = auto_fix_genome(&mut genome);
        assert!(fixes > 0, "Should apply at least one fix");
        assert_eq!(genome.physiology.quantization_precision, "fp32", 
                   "Should default to fp32");
        
        // Test 2: Non-canonical (i8 → int8)
        genome.physiology.quantization_precision = "i8".to_string();
        let _fixes = auto_fix_genome(&mut genome);
        assert_eq!(genome.physiology.quantization_precision, "int8",
                   "Should normalize i8 to int8");
        
        // Test 3: Invalid → default
        genome.physiology.quantization_precision = "invalid".to_string();
        let _fixes = auto_fix_genome(&mut genome);
        assert_eq!(genome.physiology.quantization_precision, "fp32",
                   "Invalid should default to fp32");
    }
    
    fn create_minimal_genome() -> RuntimeGenome {
        RuntimeGenome {
            metadata: GenomeMetadata {
                genome_id: "test".to_string(),
                genome_title: "Test".to_string(),
                genome_description: "".to_string(),
                version: "2.0".to_string(),
                timestamp: 0.0,
            },
            cortical_areas: HashMap::new(),
            brain_regions: HashMap::new(),
            morphologies: MorphologyRegistry::new(),
            physiology: PhysiologyConfig::default(),
            signatures: GenomeSignatures {
                genome: "0".to_string(),
                blueprint: "0".to_string(),
                physiology: "0".to_string(),
                morphologies: None,
            },
            stats: GenomeStats::default(),
        }
    }
}

