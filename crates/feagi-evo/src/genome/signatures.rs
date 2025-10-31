/*!
Genome signature generation for comparison and versioning.

Signatures are SHA-256 hashes of genome sections, enabling:
- Quick genome comparison without full deep-equal
- Detecting changes in specific genome sections
- Genome versioning and tracking

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use std::collections::HashMap;
use serde_json::Value;
use sha2::{Sha256, Digest};
use crate::{EvoResult, GenomeSignatures};

/// Generate all genome signatures
pub fn generate_signatures(
    blueprint: &HashMap<String, Value>,
    morphologies: &HashMap<String, Value>,
    physiology: &Option<Value>,
) -> EvoResult<GenomeSignatures> {
    Ok(GenomeSignatures {
        genome: generate_genome_signature(blueprint, morphologies, physiology)?,
        blueprint: generate_blueprint_signature(blueprint)?,
        physiology: generate_physiology_signature(physiology)?,
        morphologies: Some(generate_morphologies_signature(morphologies)?),
    })
}

/// Generate full genome signature
fn generate_genome_signature(
    blueprint: &HashMap<String, Value>,
    morphologies: &HashMap<String, Value>,
    physiology: &Option<Value>,
) -> EvoResult<String> {
    // Combine all sections into a single JSON value
    let mut combined = serde_json::json!({
        "blueprint": blueprint,
        "neuron_morphologies": morphologies,
    });
    
    if let Some(phys) = physiology {
        combined["physiology"] = phys.clone();
    }
    
    // Serialize with deterministic ordering
    let json_str = serde_json::to_string(&combined)?;
    
    // Generate SHA-256 hash
    Ok(hash_string(&json_str))
}

/// Generate blueprint signature
fn generate_blueprint_signature(blueprint: &HashMap<String, Value>) -> EvoResult<String> {
    let json_str = serde_json::to_string(blueprint)?;
    Ok(hash_string(&json_str))
}

/// Generate physiology signature
fn generate_physiology_signature(physiology: &Option<Value>) -> EvoResult<String> {
    match physiology {
        Some(phys) => {
            let json_str = serde_json::to_string(phys)?;
            Ok(hash_string(&json_str))
        }
        None => Ok("0000000000000000".to_string()), // Empty signature for missing physiology
    }
}

/// Generate morphologies signature
fn generate_morphologies_signature(morphologies: &HashMap<String, Value>) -> EvoResult<String> {
    let json_str = serde_json::to_string(morphologies)?;
    Ok(hash_string(&json_str))
}

/// Generate SHA-256 hash of a string, return first 16 hex chars
fn hash_string(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    let result = hasher.finalize();
    
    // Convert to hex string and take first 16 characters (matches Python behavior)
    format!("{:x}", result)[..16].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_string() {
        let hash1 = hash_string("test");
        let hash2 = hash_string("test");
        let hash3 = hash_string("different");
        
        assert_eq!(hash1, hash2, "Same input should produce same hash");
        assert_ne!(hash1, hash3, "Different inputs should produce different hashes");
        assert_eq!(hash1.len(), 16, "Hash should be 16 characters");
    }
    
    #[test]
    fn test_generate_blueprint_signature() {
        let mut blueprint = HashMap::new();
        blueprint.insert("test_area".to_string(), serde_json::json!({
            "dimensions": [10, 10, 10],
            "position": [0, 0, 0],
        }));
        
        let signature = generate_blueprint_signature(&blueprint).unwrap();
        assert_eq!(signature.len(), 16);
    }
    
    #[test]
    fn test_generate_physiology_signature() {
        let physiology = Some(serde_json::json!({
            "simulation_timestep": 0.025,
            "max_age": 10000000,
        }));
        
        let signature = generate_physiology_signature(&physiology).unwrap();
        assert_eq!(signature.len(), 16);
    }
    
    #[test]
    fn test_generate_full_signatures() {
        let mut blueprint = HashMap::new();
        blueprint.insert("test".to_string(), serde_json::json!({}));
        
        let mut morphologies = HashMap::new();
        morphologies.insert("test_morph".to_string(), serde_json::json!({
            "type": "vectors",
            "parameters": { "vectors": [[1, 0, 0]] }
        }));
        
        let physiology = Some(serde_json::json!({ "simulation_timestep": 0.025 }));
        
        let signatures = generate_signatures(&blueprint, &morphologies, &physiology).unwrap();
        
        assert_eq!(signatures.genome.len(), 16);
        assert_eq!(signatures.blueprint.len(), 16);
        assert_eq!(signatures.physiology.len(), 16);
        assert!(signatures.morphologies.is_some());
        assert_eq!(signatures.morphologies.unwrap().len(), 16);
    }
}


