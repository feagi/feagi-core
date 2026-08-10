// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Integration tests for genome loading with real genome files.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use feagi_evolutionary::{convert_hierarchical_to_flat, ensure_core_components, load_genome_from_file};

/// The runtime name of the per-voxel neuron count. Genome documents spell it
/// `per_voxel_neuron_cnt` (`_n_cnt-i` when flat) and the parser translates it to this on the way
/// in, so everything reading a `RuntimeGenome` must agree on this spelling.
const NEURONS_PER_VOXEL: &str = "neurons_per_voxel";

fn barebones_genome_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("genomes")
        .join("barebones_genome.json")
}

#[test]
fn test_load_barebones_genome() {
    // Path to real genome file (repository-local; independent of external feagi-py checkout)
    let genome_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("genomes")
        .join("barebones_genome.json");

    // Load genome
    let mut genome = load_genome_from_file(&genome_path).expect("Failed to load barebones genome");
    ensure_core_components(&mut genome);

    // Verify metadata
    assert_eq!(genome.metadata.version, "2.0");
    assert!(!genome.metadata.genome_id.is_empty());

    // Verify cortical_area areas (should have _death, _power, and _fatigue)
    assert!(
        genome.cortical_areas.len() >= 3,
        "Expected at least 3 cortical areas, got {}",
        genome.cortical_areas.len()
    );

    let death_id = feagi_evolutionary::genome::parser::string_to_cortical_id("_death").expect("Valid ID");
    let power_id = feagi_evolutionary::genome::parser::string_to_cortical_id("_power").expect("Valid ID");
    let fatigue_id = feagi_evolutionary::genome::parser::string_to_cortical_id("_fatigue").expect("Valid ID");
    assert!(genome.cortical_areas.contains_key(&death_id), "Missing _death cortical area");
    assert!(genome.cortical_areas.contains_key(&power_id), "Missing _power cortical area");
    assert!(genome.cortical_areas.contains_key(&fatigue_id), "Missing _fatigue cortical area");

    // Verify morphologies
    assert!(genome.morphologies.count() != 0, "Should have morphologies");
    assert!(genome.morphologies.contains("block_to_block"), "Missing block_to_block morphology");
    assert!(genome.morphologies.contains("projector"), "Missing projector morphology");

    // Verify physiology
    assert!(genome.physiology.simulation_timestep > 0.0);
    assert!(genome.physiology.max_age > 0);

    // Verify signatures
    assert_eq!(genome.signatures.genome.len(), 16);
    assert_eq!(genome.signatures.blueprint.len(), 16);
    assert_eq!(genome.signatures.physiology.len(), 16);

    // Verify stats
    assert!(genome.stats.innate_cortical_area_count > 0);

    println!("✅ Successfully loaded barebones genome:");
    println!("   - Genome ID: {}", genome.metadata.genome_id);
    println!("   - Cortical areas: {}", genome.cortical_areas.len());
    println!("   - Morphologies: {}", genome.morphologies.count());
    println!("   - Physiology timestep: {}", genome.physiology.simulation_timestep);
}

/// Areas injected by `ensure_core_components` come from templates rather than the genome
/// document, so they bypass the parser that supplies the runtime property names. They must still
/// arrive spelled the way every reader expects, or corticogenesis rejects them.
#[test]
fn injected_core_areas_carry_the_runtime_neuron_count_property() {
    let mut genome = load_genome_from_file(barebones_genome_path()).expect("barebones genome should load");
    let (areas_added, _) = ensure_core_components(&mut genome);

    assert!(areas_added > 0, "the barebones genome should be missing at least one core area");

    for area in genome.cortical_areas.values() {
        let neurons_per_voxel = area
            .properties
            .get(NEURONS_PER_VOXEL)
            .unwrap_or_else(|| panic!("cortical area '{}' is missing the '{NEURONS_PER_VOXEL}' property", area.name));

        assert!(
            neurons_per_voxel.as_u64().is_some_and(|count| count > 0),
            "cortical area '{}' has a non-positive '{NEURONS_PER_VOXEL}': {neurons_per_voxel}",
            area.name
        );
    }
}

/// Export reads the runtime property map, so a mismatch between the name written on load and the
/// name read on save silently replaces every area's neuron count with the default.
#[test]
fn exporting_a_genome_preserves_the_per_voxel_neuron_count() {
    let mut genome = load_genome_from_file(barebones_genome_path()).expect("barebones genome should load");

    // A value no default could produce, so the assertion cannot pass by coincidence.
    let distinctive_count = 7u64;
    let (cortical_id, area) = genome.cortical_areas.iter_mut().next().expect("genome should have a cortical area");
    area.properties
        .insert(NEURONS_PER_VOXEL.to_string(), serde_json::json!(distinctive_count));
    let exported_key = format!("_____10c-{}-cx-_n_cnt-i", cortical_id.as_base_64());

    let flat = convert_hierarchical_to_flat(&genome).expect("genome should export to flat format");

    let exported = flat
        .get("blueprint")
        .and_then(|blueprint| blueprint.get(&exported_key))
        .unwrap_or_else(|| panic!("exported genome should carry '{exported_key}'"));

    assert_eq!(
        exported.as_u64(),
        Some(distinctive_count),
        "export should carry the area's own neuron count rather than a default"
    );
}

#[test]
fn test_load_all_sample_genomes() {
    let genome_files = ["barebones_genome.json", "essential_genome.json", "test_genome.json", "vision_genome.json"];

    for genome_file in &genome_files {
        let genome_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("genomes").join(genome_file);

        match load_genome_from_file(&genome_path) {
            Ok(mut genome) => {
                ensure_core_components(&mut genome);
                println!("✅ Loaded {} successfully:", genome_file);
                println!("   - Genome ID: {}", genome.metadata.genome_id);
                println!("   - Cortical areas: {}", genome.cortical_areas.len());
                println!("   - Morphologies: {}", genome.morphologies.count());

                // Basic validation
                assert!(!genome.metadata.genome_id.is_empty());
                assert!(
                    genome.metadata.version.starts_with("2."),
                    "Expected genome version to start with '2.' but got '{}'",
                    genome.metadata.version
                );
                assert!(genome.cortical_areas.len() >= 3); // At least _death, _power, and _fatigue
            }
            Err(e) => {
                println!("⚠️  Could not load {}: {} (this is OK if file doesn't exist)", genome_file, e);
            }
        }
    }
}
