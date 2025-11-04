/*!
Convert ParsedGenome to RuntimeGenome.

This module transforms raw parsed JSON into strongly-typed runtime objects:
- Morphologies → MorphologyRegistry
- Physiology JSON → PhysiologyConfig
- Generate signatures
- Build complete RuntimeGenome

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use std::collections::HashMap;
use serde_json::Value;
use crate::{
    EvoResult, EvoError,
    ParsedGenome,
    RuntimeGenome, GenomeMetadata, MorphologyRegistry, Morphology,
    MorphologyType, MorphologyParameters, PatternElement,
    PhysiologyConfig, GenomeStats,
};
use crate::genome::signatures::generate_signatures;

/// Convert ParsedGenome to RuntimeGenome
pub fn to_runtime_genome(parsed: ParsedGenome, raw_json: &str) -> EvoResult<RuntimeGenome> {
    // Parse raw JSON to extract sections we need
    let raw_value: Value = serde_json::from_str(raw_json)?;
    
    // Extract metadata
    let metadata = GenomeMetadata {
        genome_id: parsed.genome_id.clone(),
        genome_title: parsed.genome_title.clone(),
        genome_description: raw_value["genome_description"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        version: parsed.version.clone(),
        timestamp: raw_value["timestamp"].as_f64().unwrap_or(0.0),
    };
    
    // Convert cortical areas to HashMap
    let mut cortical_areas = HashMap::new();
    for area in parsed.cortical_areas {
        cortical_areas.insert(area.cortical_id.clone(), area);
    }
    
    // Convert brain regions to HashMap
    let mut brain_regions = HashMap::new();
    for (region, _parent) in parsed.brain_regions {
        brain_regions.insert(region.region_id.clone(), region);
    }
    
    // Parse morphologies
    let morphologies = parse_morphologies(&parsed.neuron_morphologies)?;
    
    // Parse physiology
    let physiology = parse_physiology(&parsed.physiology)?;
    
    // Parse stats
    let stats = parse_stats(&raw_value)?;
    
    // Generate signatures
    // For signature generation, we need the raw blueprint and morphologies as HashMap<String, Value>
    let blueprint_map = extract_blueprint_map(&raw_value)?;
    let signatures = generate_signatures(
        &blueprint_map,
        &parsed.neuron_morphologies,
        &parsed.physiology,
    )?;
    
    Ok(RuntimeGenome {
        metadata,
        cortical_areas,
        brain_regions,
        morphologies,
        physiology,
        signatures,
        stats,
    })
}

/// Parse neuron morphologies into MorphologyRegistry
fn parse_morphologies(
    raw_morphologies: &HashMap<String, Value>
) -> EvoResult<MorphologyRegistry> {
    let mut registry = MorphologyRegistry::new();
    
    for (morphology_id, morphology_value) in raw_morphologies {
        let morphology = parse_single_morphology(morphology_value)?;
        registry.add_morphology(morphology_id.clone(), morphology);
    }
    
    Ok(registry)
}

/// Parse a single morphology
fn parse_single_morphology(value: &Value) -> EvoResult<Morphology> {
    let morphology_type_str = value["type"].as_str()
        .ok_or_else(|| EvoError::InvalidGenome("Morphology missing 'type' field".to_string()))?;
    
    let morphology_type = match morphology_type_str {
        "vectors" => MorphologyType::Vectors,
        "patterns" => MorphologyType::Patterns,
        "functions" => MorphologyType::Functions,
        "composite" => MorphologyType::Composite,
        _ => return Err(EvoError::InvalidGenome(
            format!("Unknown morphology type: {}", morphology_type_str)
        )),
    };
    
    let parameters = parse_morphology_parameters(&morphology_type, &value["parameters"])?;
    
    let class = value["class"].as_str().unwrap_or("custom").to_string();
    
    Ok(Morphology {
        morphology_type,
        parameters,
        class,
    })
}

/// Parse morphology parameters based on type
fn parse_morphology_parameters(
    morphology_type: &MorphologyType,
    params_value: &Value,
) -> EvoResult<MorphologyParameters> {
    match morphology_type {
        MorphologyType::Vectors => {
            let vectors_array = params_value["vectors"].as_array()
                .ok_or_else(|| EvoError::InvalidGenome("Vectors morphology missing 'vectors' array".to_string()))?;
            
            let mut vectors = Vec::new();
            for vec in vectors_array {
                let vec_array = vec.as_array()
                    .ok_or_else(|| EvoError::InvalidGenome("Vector must be an array".to_string()))?;
                
                if vec_array.len() != 3 {
                    return Err(EvoError::InvalidGenome(
                        format!("Vector must have 3 elements, got {}", vec_array.len())
                    ));
                }
                
                let x = vec_array[0].as_i64().ok_or_else(|| 
                    EvoError::InvalidGenome("Vector element must be an integer".to_string()))? as i32;
                let y = vec_array[1].as_i64().ok_or_else(|| 
                    EvoError::InvalidGenome("Vector element must be an integer".to_string()))? as i32;
                let z = vec_array[2].as_i64().ok_or_else(|| 
                    EvoError::InvalidGenome("Vector element must be an integer".to_string()))? as i32;
                
                vectors.push([x, y, z]);
            }
            
            Ok(MorphologyParameters::Vectors { vectors })
        }
        
        MorphologyType::Patterns => {
            let patterns_array = params_value["patterns"].as_array()
                .ok_or_else(|| EvoError::InvalidGenome("Patterns morphology missing 'patterns' array".to_string()))?;
            
            let mut patterns = Vec::new();
            for pattern in patterns_array {
                let pattern_pair = pattern.as_array()
                    .ok_or_else(|| EvoError::InvalidGenome("Pattern must be an array of [source, dest]".to_string()))?;
                
                if pattern_pair.len() != 2 {
                    return Err(EvoError::InvalidGenome(
                        "Pattern must have 2 elements [source, dest]".to_string()
                    ));
                }
                
                let source = parse_pattern_elements(&pattern_pair[0])?;
                let dest = parse_pattern_elements(&pattern_pair[1])?;
                
                patterns.push([source, dest]);
            }
            
            Ok(MorphologyParameters::Patterns { patterns })
        }
        
        MorphologyType::Functions => {
            Ok(MorphologyParameters::Functions {})
        }
        
        MorphologyType::Composite => {
            let src_seed = parse_u32_array(params_value["src_seed"].as_array()
                .ok_or_else(|| EvoError::InvalidGenome("Composite missing 'src_seed'".to_string()))?,
                3)?;
            
            let src_pattern_array = params_value["src_pattern"].as_array()
                .ok_or_else(|| EvoError::InvalidGenome("Composite missing 'src_pattern'".to_string()))?;
            
            let mut src_pattern = Vec::new();
            for item in src_pattern_array {
                let pair = item.as_array()
                    .ok_or_else(|| EvoError::InvalidGenome("src_pattern item must be [i32, i32]".to_string()))?;
                
                if pair.len() != 2 {
                    return Err(EvoError::InvalidGenome("src_pattern item must have 2 elements".to_string()));
                }
                
                let a = pair[0].as_i64().ok_or_else(|| EvoError::InvalidGenome("src_pattern value must be integer".to_string()))? as i32;
                let b = pair[1].as_i64().ok_or_else(|| EvoError::InvalidGenome("src_pattern value must be integer".to_string()))? as i32;
                
                src_pattern.push([a, b]);
            }
            
            let mapper_morphology = params_value["mapper_morphology"].as_str()
                .ok_or_else(|| EvoError::InvalidGenome("Composite missing 'mapper_morphology'".to_string()))?
                .to_string();
            
            Ok(MorphologyParameters::Composite {
                src_seed,
                src_pattern,
                mapper_morphology,
            })
        }
    }
}

/// Parse pattern elements (handles *, ?, !, and integer values)
fn parse_pattern_elements(value: &Value) -> EvoResult<Vec<PatternElement>> {
    let array = value.as_array()
        .ok_or_else(|| EvoError::InvalidGenome("Pattern element must be an array".to_string()))?;
    
    let mut elements = Vec::new();
    for elem in array {
        let pattern_elem = if let Some(s) = elem.as_str() {
            match s {
                "*" => PatternElement::Wildcard,
                "?" => PatternElement::Skip,
                "!" => PatternElement::Exclude,
                _ => return Err(EvoError::InvalidGenome(
                    format!("Unknown pattern element: {}", s)
                )),
            }
        } else if let Some(i) = elem.as_i64() {
            PatternElement::Value(i as i32)
        } else {
            return Err(EvoError::InvalidGenome(
                "Pattern element must be string or integer".to_string()
            ));
        };
        
        elements.push(pattern_elem);
    }
    
    Ok(elements)
}

/// Parse u32 array from JSON array
fn parse_u32_array(array: &[Value], expected_len: usize) -> EvoResult<[u32; 3]> {
    if array.len() != expected_len {
        return Err(EvoError::InvalidGenome(
            format!("Expected array of length {}, got {}", expected_len, array.len())
        ));
    }
    
    let mut result = [0u32; 3];
    for (i, val) in array.iter().enumerate() {
        result[i] = val.as_u64()
            .ok_or_else(|| EvoError::InvalidGenome("Array element must be unsigned integer".to_string()))? as u32;
    }
    
    Ok(result)
}

/// Parse physiology configuration
fn parse_physiology(physiology_value: &Option<Value>) -> EvoResult<PhysiologyConfig> {
    match physiology_value {
        Some(value) => {
            // Handle migration: burst_delay → simulation_timestep
            let simulation_timestep = value["simulation_timestep"].as_f64()
                .or_else(|| value["burst_delay"].as_f64())
                .unwrap_or(0.025);
            
            // Parse quantization precision (new field)
            let quantization_precision = value["quantization_precision"]
                .as_str()
                .unwrap_or("fp32")
                .to_string();
            
            Ok(PhysiologyConfig {
                simulation_timestep,
                max_age: value["max_age"].as_u64().unwrap_or(10_000_000),
                evolution_burst_count: value["evolution_burst_count"].as_u64().unwrap_or(50),
                ipu_idle_threshold: value["ipu_idle_threshold"].as_u64().unwrap_or(1000),
                plasticity_queue_depth: value["plasticity_queue_depth"].as_u64().unwrap_or(3) as usize,
                lifespan_mgmt_interval: value["lifespan_mgmt_interval"].as_u64().unwrap_or(10),
                quantization_precision,
            })
        }
        None => Ok(PhysiologyConfig::default()),
    }
}

/// Parse genome statistics
fn parse_stats(genome_value: &Value) -> EvoResult<GenomeStats> {
    if let Some(stats_value) = genome_value.get("stats") {
        Ok(GenomeStats {
            innate_cortical_area_count: stats_value["innate_cortical_area_count"].as_u64().unwrap_or(0) as usize,
            innate_neuron_count: stats_value["innate_neuron_count"].as_u64().unwrap_or(0) as usize,
            innate_synapse_count: stats_value["innate_synapse_count"].as_u64().unwrap_or(0) as usize,
        })
    } else {
        Ok(GenomeStats::default())
    }
}

/// Extract blueprint as HashMap<String, Value> for signature generation
fn extract_blueprint_map(genome_value: &Value) -> EvoResult<HashMap<String, Value>> {
    let blueprint = genome_value.get("blueprint")
        .ok_or_else(|| EvoError::InvalidGenome("Missing blueprint section".to_string()))?;
    
    if let Some(obj) = blueprint.as_object() {
        let mut map = HashMap::new();
        for (k, v) in obj {
            map.insert(k.clone(), v.clone());
        }
        Ok(map)
    } else {
        Err(EvoError::InvalidGenome("Blueprint must be an object".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_vector_morphology() {
        let json = serde_json::json!({
            "type": "vectors",
            "parameters": {
                "vectors": [[1, 0, 0], [0, 1, 0]]
            },
            "class": "test"
        });
        
        let morphology = parse_single_morphology(&json).unwrap();
        assert_eq!(morphology.morphology_type, MorphologyType::Vectors);
        assert_eq!(morphology.class, "test");
        
        if let MorphologyParameters::Vectors { vectors } = morphology.parameters {
            assert_eq!(vectors.len(), 2);
            assert_eq!(vectors[0], [1, 0, 0]);
            assert_eq!(vectors[1], [0, 1, 0]);
        } else {
            panic!("Expected Vectors parameters");
        }
    }
    
    #[test]
    fn test_parse_pattern_elements() {
        let json = serde_json::json!([1, "*", "?", "!", 5]);
        let elements = parse_pattern_elements(&json).unwrap();
        
        assert_eq!(elements.len(), 5);
        assert_eq!(elements[0], PatternElement::Value(1));
        assert_eq!(elements[1], PatternElement::Wildcard);
        assert_eq!(elements[2], PatternElement::Skip);
        assert_eq!(elements[3], PatternElement::Exclude);
        assert_eq!(elements[4], PatternElement::Value(5));
    }
    
    #[test]
    fn test_parse_physiology_with_migration() {
        // Test burst_delay → simulation_timestep migration
        let json = serde_json::json!({
            "burst_delay": 0.030,
            "max_age": 5000000
        });
        
        let physiology = parse_physiology(&Some(json)).unwrap();
        assert_eq!(physiology.simulation_timestep, 0.030);
        assert_eq!(physiology.max_age, 5000000);
    }
}

