// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Runtime genome representation for FEAGI.

This module defines the in-memory Rust objects that represent a loaded genome.
These objects are created by the genome parser and consumed by neuroembryogenesis.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use feagi_genome_definitions::::CorticalArea;
use feagi_genome_definitions::::CorticalID;
use feagi_genome_definitions::::brain_region::BrainRegion;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete runtime genome representation
#[derive(Debug, Clone)]
pub struct RuntimeGenome {
    /// Genome metadata
    pub metadata: GenomeMetadata,

    /// Cortical areas (by cortical_id as CorticalID)
    pub cortical_areas: HashMap<CorticalID, CorticalArea>,

    /// Brain regions (by region_id)
    pub brain_regions: HashMap<String, BrainRegion>,

    /// Morphology registry
    pub morphologies: MorphologyRegistry,

    /// Physiology configuration
    pub physiology: PhysiologyConfig,

    /// Genome signatures
    pub signatures: GenomeSignatures,

    /// Statistics
    pub stats: GenomeStats,
}

/// Genome metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeMetadata {
    pub genome_id: String,
    pub genome_title: String,
    pub genome_description: String,
    pub version: String,
    pub timestamp: f64, // Unix timestamp

    /// Root brain region ID (UUID string) - explicit identification for O(1) lookup
    /// This eliminates the need to search through all regions to find which has no parent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brain_regions_root: Option<String>,
}

/// Neuron morphology registry
#[derive(Debug, Clone, Default)]
pub struct MorphologyRegistry {
    /// All morphologies by morphology_id
    morphologies: HashMap<String, Morphology>,
}

impl MorphologyRegistry {
    /// Create empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a morphology
    pub fn add_morphology(&mut self, id: String, morphology: Morphology) {
        self.morphologies.insert(id, morphology);
    }

    /// Get a morphology by ID
    pub fn get(&self, id: &str) -> Option<&Morphology> {
        self.morphologies.get(id)
    }

    /// Check if morphology exists
    pub fn contains(&self, id: &str) -> bool {
        self.morphologies.contains_key(id)
    }

    /// Get all morphology IDs
    pub fn morphology_ids(&self) -> Vec<String> {
        self.morphologies.keys().cloned().collect()
    }

    /// Remove a morphology by ID.
    ///
    /// Returns true if the morphology existed and was removed.
    pub fn remove_morphology(&mut self, id: &str) -> bool {
        self.morphologies.remove(id).is_some()
    }

    /// Get count of morphologies
    pub fn count(&self) -> usize {
        self.morphologies.len()
    }

    /// Iterate over all morphologies
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Morphology)> {
        self.morphologies.iter()
    }
}

/// Neuron morphology definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Morphology {
    /// Morphology type: "vectors", "patterns", "functions", or "composite"
    pub morphology_type: MorphologyType,

    /// Morphology parameters
    pub parameters: MorphologyParameters,

    /// Morphology class: "core", "custom", etc.
    pub class: String,
}

/// Morphology type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MorphologyType {
    /// Vector-based morphology (3D offset vectors)
    Vectors,

    /// Pattern-based morphology (source → destination patterns)
    Patterns,

    /// Function-based morphology (built-in algorithms)
    Functions,

    /// Composite morphology (combines multiple morphologies)
    Composite,
}

/// Morphology parameters (type-specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MorphologyParameters {
    /// Vector parameters: list of [x, y, z] offsets
    Vectors { vectors: Vec<[i32; 3]> },

    /// Pattern parameters: list of [source_pattern, dest_pattern] pairs
    Patterns {
        patterns: Vec<[Vec<PatternElement>; 2]>,
    },

    /// Function parameters: empty for built-in functions
    Functions {},

    /// Composite parameters: combines seed + pattern + mapper
    Composite {
        src_seed: [u32; 3],
        src_pattern: Vec<[i32; 2]>,
        mapper_morphology: String,
    },
}

/// Pattern element: exact value, wildcard (*), skip (?), exclude (!), or relative directional
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternElement {
    /// Exact coordinate value
    Value(i32),
    /// Wildcard - matches any value
    Wildcard, // "*"
    /// Skip - don't check this coordinate
    Skip, // "?"
    /// Exclude - exclude this coordinate
    Exclude, // "!"
    /// All coordinates strictly above source on this axis
    DirectionPositive, // "?+"
    /// All coordinates strictly below source on this axis
    DirectionNegative, // "?-"
    /// All coordinates at or above source on this axis
    DirectionPositiveInclusive, // "?+="
    /// All coordinates at or below source on this axis
    DirectionNegativeInclusive, // "?-="
    /// Single coordinate at offset from source
    Offset(i32), // "?+N" or "?-N"
    /// Inclusive range relative to source [src+lo, src+hi]
    Range(i32, i32), // "?-A:?+B"
}

// Custom serialization to convert PatternElement back to JSON properly
impl Serialize for PatternElement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            PatternElement::Value(v) => serializer.serialize_i32(*v),
            PatternElement::Wildcard => serializer.serialize_str("*"),
            PatternElement::Skip => serializer.serialize_str("?"),
            PatternElement::Exclude => serializer.serialize_str("!"),
            PatternElement::DirectionPositive => serializer.serialize_str("?+"),
            PatternElement::DirectionNegative => serializer.serialize_str("?-"),
            PatternElement::DirectionPositiveInclusive => serializer.serialize_str("?+="),
            PatternElement::DirectionNegativeInclusive => serializer.serialize_str("?-="),
            PatternElement::Offset(off) => {
                if *off >= 0 {
                    serializer.serialize_str(&format!("?+{}", off))
                } else {
                    serializer.serialize_str(&format!("?{}", off))
                }
            }
            PatternElement::Range(lo, hi) => {
                let lo_str = if *lo >= 0 {
                    format!("?+{}", lo)
                } else {
                    format!("?{}", lo)
                };
                let hi_str = if *hi >= 0 {
                    format!("?+{}", hi)
                } else {
                    format!("?{}", hi)
                };
                serializer.serialize_str(&format!("{}:{}", lo_str, hi_str))
            }
        }
    }
}

// Custom deserialization to parse JSON into PatternElement
impl<'de> Deserialize<'de> for PatternElement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(PatternElement::Value(i as i32))
                } else {
                    Err(serde::de::Error::custom(
                        "Pattern element must be an integer",
                    ))
                }
            }
            serde_json::Value::String(s) => Self::parse_string(&s)
                .ok_or_else(|| serde::de::Error::custom(format!("Unknown pattern element: {}", s))),
            _ => Err(serde::de::Error::custom(
                "Pattern element must be number or string",
            )),
        }
    }
}

impl PatternElement {
    /// Parse a pattern element from its string representation.
    pub fn parse_string(s: &str) -> Option<Self> {
        match s {
            "*" => Some(PatternElement::Wildcard),
            "?" => Some(PatternElement::Skip),
            "!" => Some(PatternElement::Exclude),
            "?+" => Some(PatternElement::DirectionPositive),
            "?-" => Some(PatternElement::DirectionNegative),
            "?+=" => Some(PatternElement::DirectionPositiveInclusive),
            "?-=" => Some(PatternElement::DirectionNegativeInclusive),
            _ => {
                if let Some(range) = Self::try_parse_range(s) {
                    return Some(range);
                }
                if let Some(offset) = Self::try_parse_offset(s) {
                    return Some(offset);
                }
                None
            }
        }
    }

    fn try_parse_range(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
        let lo = Self::extract_relative_offset(parts[0])?;
        let hi = Self::extract_relative_offset(parts[1])?;
        Some(PatternElement::Range(lo, hi))
    }

    fn try_parse_offset(s: &str) -> Option<Self> {
        let offset = Self::extract_relative_offset(s)?;
        Some(PatternElement::Offset(offset))
    }

    fn extract_relative_offset(s: &str) -> Option<i32> {
        if !s.starts_with('?') {
            return None;
        }
        let rest = &s[1..];
        if rest.is_empty() || rest == "+" || rest == "-" || rest == "+=" || rest == "-=" {
            return None;
        }
        rest.parse::<i32>().ok()
    }
}

/// Physiology configuration (runtime parameters)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysiologyConfig {
    /// Simulation timestep in seconds (formerly burst_delay)
    pub simulation_timestep: f64,

    /// Maximum neuron age
    pub max_age: u64,

    /// Evolution burst count
    pub evolution_burst_count: u64,

    /// IPU idle threshold
    pub ipu_idle_threshold: u64,

    /// Plasticity queue depth
    pub plasticity_queue_depth: usize,

    /// Lifespan management interval
    pub lifespan_mgmt_interval: u64,

    /// Quantization precision for numeric values
    /// Options: "fp32" (default), "fp16", "int8"
    #[serde(default = "default_quantization_precision")]
    pub quantization_precision: String,
}

pub fn default_quantization_precision() -> String {
    "int8".to_string() // Default to INT8 for memory efficiency
}

impl Default for PhysiologyConfig {
    fn default() -> Self {
        Self {
            simulation_timestep: 0.025,
            max_age: 10_000_000,
            evolution_burst_count: 50,
            ipu_idle_threshold: 1000,
            plasticity_queue_depth: 3,
            lifespan_mgmt_interval: 10,
            quantization_precision: default_quantization_precision(),
        }
    }
}

/// Genome signatures for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeSignatures {
    /// Full genome signature
    pub genome: String,

    /// Blueprint signature
    pub blueprint: String,

    /// Physiology signature
    pub physiology: String,

    /// Morphologies signature (optional, for future extension)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub morphologies: Option<String>,
}

/// Genome statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenomeStats {
    /// Innate cortical area count
    pub innate_cortical_area_count: usize,

    /// Innate neuron count
    pub innate_neuron_count: usize,

    /// Innate synapse count
    pub innate_synapse_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_morphology_registry_creation() {
        let registry = MorphologyRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_morphology_registry_add_and_get() {
        let mut registry = MorphologyRegistry::new();

        let morphology = Morphology {
            morphology_type: MorphologyType::Vectors,
            parameters: MorphologyParameters::Vectors {
                vectors: vec![[1, 0, 0], [0, 1, 0]],
            },
            class: "test".to_string(),
        };

        registry.add_morphology("test_morph".to_string(), morphology);

        assert_eq!(registry.count(), 1);
        assert!(registry.contains("test_morph"));
        assert!(registry.get("test_morph").is_some());
    }

    #[test]
    fn test_physiology_config_default() {
        let config = PhysiologyConfig::default();
        assert_eq!(config.simulation_timestep, 0.025);
        assert_eq!(config.max_age, 10_000_000);
    }
}
