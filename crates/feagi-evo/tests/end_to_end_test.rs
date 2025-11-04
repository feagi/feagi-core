/*!
End-to-end integration test for feagi-evo.

Tests the complete genome workflow:
1. Create genome from template
2. Validate genome
3. Save to JSON
4. Load from JSON
5. Validate again

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use feagi_evo::{
    create_genome_with_core_morphologies,
    save_genome_to_json,
    load_genome_from_json,
    validate_genome,
};

#[test]
fn test_complete_genome_workflow() {
    // 1. Create genome from template
    let mut genome = create_genome_with_core_morphologies(
        "test_workflow_genome".to_string(),
        "End-to-End Test Genome".to_string(),
    );
    
    // Add a cortical area
    let area = feagi_types::CorticalArea::new(
        "test01".to_string(),
        0,
        "Test Area".to_string(),
        feagi_types::Dimensions::new(10, 10, 10),
        (0, 0, 0),
        feagi_types::AreaType::Custom,
    ).expect("Failed to create cortical area");
    genome.cortical_areas.insert("test01".to_string(), area);
    
    // 2. Validate genome
    let validation = validate_genome(&genome);
    assert!(validation.errors.is_empty(), "Genome should have no errors: {:?}", validation.errors);
    
    // 3. Save to JSON
    let json_str = save_genome_to_json(&genome).expect("Failed to save genome");
    assert!(json_str.contains("test_workflow_genome"));
    assert!(json_str.contains("test01"));
    assert!(json_str.contains("block_to_block"));
    
    // 4. Load from JSON
    let loaded_genome = load_genome_from_json(&json_str).expect("Failed to load genome");
    
    // 5. Validate loaded genome
    let validation2 = validate_genome(&loaded_genome);
    assert!(validation2.errors.is_empty(), "Loaded genome should have no errors: {:?}", validation2.errors);
    
    // 6. Verify data integrity
    assert_eq!(loaded_genome.metadata.genome_id, "test_workflow_genome");
    assert_eq!(loaded_genome.cortical_areas.len(), 1);
    assert!(loaded_genome.cortical_areas.contains_key("test01"));
    assert!(loaded_genome.morphologies.contains("block_to_block"));
    assert!(loaded_genome.morphologies.contains("projector"));
    assert_eq!(loaded_genome.metadata.version, "2.0");
    
    println!("✅ Complete genome workflow test passed!");
    println!("   - Created genome with {} morphologies", loaded_genome.morphologies.count());
    println!("   - Cortical areas: {}", loaded_genome.cortical_areas.len());
    println!("   - Validation: {} warnings", validation2.warnings.len());
}

#[test]
fn test_minimal_genome_creation() {
    use feagi_evo::create_minimal_genome;
    
    let genome = create_minimal_genome(
        "minimal_test".to_string(),
        "Minimal Test".to_string(),
    );
    
    assert_eq!(genome.metadata.genome_id, "minimal_test");
    assert_eq!(genome.cortical_areas.len(), 0);
    assert_eq!(genome.morphologies.count(), 0);
    
    let validation = validate_genome(&genome);
    // Should have warnings about empty areas and morphologies, but no errors
    assert!(validation.errors.is_empty());
    assert!(!validation.warnings.is_empty());
    
    println!("✅ Minimal genome creation test passed!");
}

#[test]
fn test_flat_to_hierarchical_conversion() {
    use feagi_evo::convert_flat_to_hierarchical;
    use serde_json::json;
    
    let flat = json!({
        "genome_id": "flat_test",
        "genome_title": "Flat Test Genome",
        "version": "2.0",
        "blueprint": {
            "_____10c-test01-cx-__name-t": "Test Area",
            "_____10c-test01-cx-___bbx-i": 10,
            "_____10c-test01-cx-___bby-i": 10,
            "_____10c-test01-cx-___bbz-i": 10,
            "_____10c-test01-cx-rcordx-i": 0,
            "_____10c-test01-cx-rcordy-i": 0,
            "_____10c-test01-cx-rcordz-i": 0,
            "_____10c-test01-cx-_group-t": "CUSTOM"
        },
        "neuron_morphologies": {
            "block_to_block": {
                "type": "vectors",
                "parameters": {"vectors": [[0, 0, 0]]},
                "class": "core"
            }
        },
        "physiology": {
            "simulation_timestep": 0.025,
            "max_age": 10000000
        },
    });
    
    // Convert flat to hierarchical
    let hierarchical = convert_flat_to_hierarchical(&flat).expect("Conversion failed");
    
    // Verify conversion
    assert!(hierarchical.get("blueprint").is_some());
    let blueprint = hierarchical.get("blueprint").unwrap().as_object().unwrap();
    assert!(blueprint.contains_key("test01"), "Blueprint should contain test01 area");
    
    let area = blueprint.get("test01").unwrap().as_object().unwrap();
    assert_eq!(area.get("cortical_name").unwrap(), "Test Area");
    assert_eq!(area.get("cortical_type").unwrap(), "CUSTOM");
    
    // Load as RuntimeGenome
    let json_str = serde_json::to_string_pretty(&hierarchical).unwrap();
    let genome = load_genome_from_json(&json_str).expect("Failed to load converted genome");
    
    assert_eq!(genome.metadata.genome_id, "flat_test");
    assert_eq!(genome.cortical_areas.len(), 1);
    
    println!("✅ Flat-to-hierarchical conversion test passed!");
}




