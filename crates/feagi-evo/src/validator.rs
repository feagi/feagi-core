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
use feagi_data_structures::genomic::cortical_area::CorticalID;

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
        let cortical_id_display = cortical_id.to_string();
        // Fix zero dimensions
        if area.dimensions.width == 0 {
            info!("🔧 AUTO-FIX: Cortical area '{}' width 0 → 1", cortical_id_display);
            area.dimensions.width = 1;
            fixes_applied += 1;
        }
        if area.dimensions.height == 0 {
            info!("🔧 AUTO-FIX: Cortical area '{}' height 0 → 1", cortical_id_display);
            area.dimensions.height = 1;
            fixes_applied += 1;
        }
        if area.dimensions.depth == 0 {
            info!("🔧 AUTO-FIX: Cortical area '{}' depth 0 → 1", cortical_id_display);
            area.dimensions.depth = 1;
            fixes_applied += 1;
        }
        
        // Fix zero per_voxel_neuron_cnt
        if let Some(per_voxel_value) = area.properties.get_mut("per_voxel_neuron_cnt") {
            if let Some(0) = per_voxel_value.as_i64() {
                info!("🔧 AUTO-FIX: Cortical area '{}' per_voxel_neuron_cnt 0 → 1", cortical_id_display);
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
        let cortical_id_display = cortical_id.to_string();
        
        // CRITICAL: Validate cortical ID format and compliance with feagi-data-processing templates
        validate_cortical_id_format(cortical_id, &cortical_id_display, result);
        
        // Validate dimensions - AUTO-FIX zeros to 1
        if area.dimensions.width == 0 || area.dimensions.height == 0 || area.dimensions.depth == 0 {
            result.add_warning(format!(
                "AUTO-FIX: Cortical area '{}' has zero dimension(s): {}x{}x{} - will be corrected to minimum (1,1,1)",
                cortical_id_display, area.dimensions.width, area.dimensions.height, area.dimensions.depth
            ));
            // Note: Auto-fix happens in auto_fix_genome() - this just detects the issue
        }
        
        // Validate per_voxel_neuron_cnt
        if let Some(per_voxel) = area.properties.get("per_voxel_neuron_cnt").and_then(|v| v.as_i64()) {
            if per_voxel == 0 {
                result.add_warning(format!(
                    "AUTO-FIX: Cortical area '{}' has per_voxel_neuron_cnt=0 - will be corrected to 1",
                    cortical_id_display
                ));
            }
        }
        
        // Warn about very large dimensions
        let total_voxels = area.dimensions.width * area.dimensions.height * area.dimensions.depth;
        if total_voxels > 1_000_000 {
            result.add_warning(format!(
                "Cortical area '{}' has very large dimensions: {} total voxels",
                cortical_id_display, total_voxels
            ));
        }
        
        // Validate name
        if area.name.is_empty() {
            result.add_warning(format!(
                "Cortical area '{}' has empty name",
                cortical_id_display
            ));
        }
    }
}

/// Validate cortical ID format and compliance with feagi-data-processing templates
fn validate_cortical_id_format(_cortical_id: &CorticalID, display: &str, result: &mut ValidationResult) {
    // Check if display format is 8 characters (required)
    if display.len() != 8 {
        result.add_error(format!(
            "Invalid cortical ID length: '{}' is {} characters (must be exactly 8)",
            display, display.len()
        ));
        return;
    }
    
    // Check if it's a CORE area (starts with underscore)
    if display.starts_with('_') {
        validate_core_area_id(display, result);
        return;
    }
    
    // Check if it's a CUSTOM/MEMORY area (starts with 'c')
    if display.starts_with('c') {
        // Custom areas: No strict validation yet, but should follow naming conventions
        // Just check that it's properly padded
        if !display.chars().all(|c| c.is_alphanumeric() || c == '_') {
            result.add_warning(format!(
                "Custom cortical ID '{}' contains non-alphanumeric characters",
                display
            ));
        }
        return;
    }
    
    // Check if it's an IPU/OPU area (3-char prefix + 5 chars)
    validate_io_area_id(display, result);
}

/// Validate CORE area IDs (power, death, health, energy, etc.)
fn validate_core_area_id(display: &str, result: &mut ValidationResult) {
    // Known valid CORE area IDs
    const VALID_CORE_IDS: &[&str] = &[
        "_power__",  // Power injector
        "_death__",  // Death detector
        "_health_",  // Health monitor (7 chars + 1 padding)
        "_energy_",  // Energy monitor (7 chars + 1 padding)
    ];
    
    if !VALID_CORE_IDS.contains(&display) {
        result.add_error(format!(
            "Invalid CORE cortical ID: '{}' - must be one of: {:?}",
            display, VALID_CORE_IDS
        ));
    }
}

/// Validate IPU/OPU area IDs (should follow template system)
fn validate_io_area_id(display: &str, result: &mut ValidationResult) {
    // Extract 3-character prefix
    let prefix = &display[0..3];
    
    // Known valid IPU prefixes from feagi-data-processing templates
    const VALID_IPU_PREFIXES: &[&str] = &[
        "svi",  // SegmentedVision (9 areas: svi0____ to svi8____)
        "aud",  // Audio
        "tac",  // Tactile
        "olf",  // Olfactory
        "vis",  // Vision (generic)
    ];
    
    // Known valid OPU prefixes from feagi-data-processing templates
    const VALID_OPU_PREFIXES: &[&str] = &[
        "mot",  // Motor
        "voc",  // Vocal
        "gaz",  // Gaze control
    ];
    
    let is_valid_ipu = VALID_IPU_PREFIXES.contains(&prefix);
    let is_valid_opu = VALID_OPU_PREFIXES.contains(&prefix);
    
    if !is_valid_ipu && !is_valid_opu {
        // Check for OLD invalid formats
        if prefix.starts_with("iic") || prefix.starts_with("opu") || prefix.starts_with("omo") || prefix.starts_with("oga") {
            result.add_error(format!(
                "INVALID OLD-FORMAT cortical ID: '{}' - prefix '{}' is not compliant with feagi-data-processing templates. \
                Valid IPU prefixes: {:?}, Valid OPU prefixes: {:?}. \
                This genome needs migration to the new format.",
                display, prefix, VALID_IPU_PREFIXES, VALID_OPU_PREFIXES
            ));
        } else {
            result.add_warning(format!(
                "Unknown cortical ID prefix: '{}' in '{}' - may not follow feagi-data-processing template system. \
                Valid IPU prefixes: {:?}, Valid OPU prefixes: {:?}",
                prefix, display, VALID_IPU_PREFIXES, VALID_OPU_PREFIXES
            ));
        }
        return;
    }
    
    // Validate the index/suffix part (characters 3-7)
    let suffix = &display[3..];
    
    // For SegmentedVision (svi), indices should be 0-8
    if prefix == "svi" {
        if let Some(first_char) = suffix.chars().next() {
            if first_char.is_ascii_digit() {
                let digit = first_char as u8 - b'0';
                if digit > 8 {
                    result.add_error(format!(
                        "Invalid SegmentedVision index: '{}' in '{}' - SegmentedVision has 9 areas (indices 0-8)",
                        digit, display
                    ));
                }
            }
        }
    }
    
    // Check that suffix is properly padded with underscores
    if !suffix.chars().all(|c| c.is_alphanumeric() || c == '_') {
        result.add_warning(format!(
            "Cortical ID '{}' has invalid characters in suffix (should be alphanumeric or underscore)",
            display
        ));
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
        let cortical_id_display = cortical_id.to_string();
        if let Some(Value::Object(dstmap)) = area.properties.get("dstmap") {
            for (dest_area, rules) in dstmap {
                // Check if destination area exists (convert string to CorticalID)
                if let Ok(dest_cortical_id) = crate::genome::parser::string_to_cortical_id(&dest_area) {
                    if !genome.cortical_areas.contains_key(&dest_cortical_id) {
                        result.add_error(format!(
                            "Cortical area '{}' references non-existent destination area '{}' in dstmap",
                            cortical_id_display, dest_area
                        ));
                    }
                } else {
                    result.add_error(format!(
                        "Cortical area '{}' has invalid destination area ID '{}' in dstmap",
                        cortical_id_display, dest_area
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
                                        cortical_id_display, morph_id
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
        for cortical_id_str in &region.cortical_areas {
            // Convert string to CorticalID for lookup
            if let Ok(cortical_id) = crate::genome::parser::string_to_cortical_id(cortical_id_str) {
                if !genome.cortical_areas.contains_key(&cortical_id) {
                    result.add_error(format!(
                        "Brain region '{}' references non-existent cortical area '{}'",
                        region_id, cortical_id_str
                    ));
                }
            } else {
                result.add_error(format!(
                    "Brain region '{}' has invalid cortical area ID '{}'",
                    region_id, cortical_id_str
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
        
        // Add a valid cortical area (use _power which is a valid core ID)
        let area = feagi_types::CorticalArea::new(
            "_power".to_string(),
            0,
            "Test Area".to_string(),
            feagi_types::Dimensions::new(10, 10, 10),
            (0, 0, 0),
            feagi_types::AreaType::Custom,
        ).expect("Failed to create cortical area");
        
        let test_id = crate::genome::parser::string_to_cortical_id("_power").expect("Valid ID");
        genome.cortical_areas.insert(test_id, area);
        
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
        assert_eq!(genome.physiology.quantization_precision, "int8", 
                   "Should default to int8");
        
        // Test 2: Non-canonical (i8 → int8)
        genome.physiology.quantization_precision = "i8".to_string();
        let _fixes = auto_fix_genome(&mut genome);
        assert_eq!(genome.physiology.quantization_precision, "int8",
                   "Should normalize i8 to int8");
        
        // Test 3: Invalid → default
        genome.physiology.quantization_precision = "invalid".to_string();
        let _fixes = auto_fix_genome(&mut genome);
        assert_eq!(genome.physiology.quantization_precision, "int8",
                   "Invalid should default to int8");
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

