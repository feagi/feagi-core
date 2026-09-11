// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Integration tests for genome loading with real genome files.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use feagi_evolutionary::{convert_hierarchical_to_flat, ensure_core_components, load_barebones_genome, load_essential_genome, load_genome_from_file};

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

/// Every distinct cortical area named by a flat blueprint key, in declaration order.
///
/// Flat keys are spelled `_____10c-<cortical_id>-<section>-<property>-<type>`.
fn cortical_ids_declared_in_flat_genome(genome_json: &str) -> Vec<String> {
    let genome: serde_json::Value = serde_json::from_str(genome_json).expect("embedded genome is valid JSON");
    let blueprint = genome.get("blueprint").and_then(|b| b.as_object()).expect("genome carries a blueprint");

    let mut declared: Vec<String> = Vec::new();
    for flat_key in blueprint.keys() {
        let Some(cortical_id) = flat_key.split('-').nth(1) else {
            continue;
        };
        if !declared.iter().any(|seen| seen == cortical_id) {
            declared.push(cortical_id.to_string());
        }
    }
    declared
}

/// A genome load must publish every cortical area the document declares.
///
/// The loader skips a blueprint entry whose ID it cannot resolve, and it does so with only a log
/// line, so a gap in ID resolution shows up as areas quietly missing from the connectome rather
/// than as a failed load. The essential genome is the case that matters most here: twelve of its
/// twenty-four areas are custom areas addressed by legacy ASCII name, and when those stopped
/// resolving, loading it left the brain holding nothing but the injected core areas.
#[test]
fn loading_the_essential_genome_publishes_every_declared_area() {
    let declared = cortical_ids_declared_in_flat_genome(feagi_evolutionary::ESSENTIAL_GENOME_JSON);
    let genome = feagi_evolutionary::load_genome_from_json(feagi_evolutionary::ESSENTIAL_GENOME_JSON).expect("essential genome loads");

    // The document is loaded verbatim here, without the core-area injection, so the counts have to
    // match exactly: a dropped area lowers the count and a collision between two migrated IDs
    // lowers it too.
    assert_eq!(
        genome.cortical_areas.len(),
        declared.len(),
        "genome declares {} areas but {} were loaded",
        declared.len(),
        genome.cortical_areas.len()
    );

    // Custom areas keep their legacy ASCII name through the migration, so they can be named
    // directly. These are the twelve that went missing.
    for legacy_id in declared.iter().filter(|id| id.starts_with('c')) {
        let cortical_id = feagi_evolutionary::genome::parser::string_to_cortical_id(legacy_id)
            .unwrap_or_else(|e| panic!("custom area '{legacy_id}' has no resolvable cortical ID: {e}"));
        assert!(
            genome.cortical_areas.contains_key(&cortical_id),
            "custom area '{legacy_id}' is missing from the loaded genome"
        );
    }

    // Loading through the template entry point adds the core areas the document omits.
    let with_core_areas = load_essential_genome().expect("essential genome loads with core areas");
    assert!(
        with_core_areas.cortical_areas.len() > declared.len(),
        "core-area injection should add to the declared set, not replace it"
    );
}

/// A genome that declares no regions still has to arrive with a hierarchy the visualizer can draw.
///
/// The essential genome carries no `brain_regions` section, so the whole hierarchy is synthesized
/// during the load. Putting every area on the root leaves the visualizer with a single plate and
/// no nesting, which is what a region-less genome looked like after the areas were placed by hand.
#[test]
fn loading_the_essential_genome_nests_its_custom_areas_under_the_root() {
    let mut genome = load_essential_genome().expect("essential genome loads");
    let (root_region_id, root_was_created) = feagi_evolutionary::ensure_root_brain_region(&mut genome);

    assert!(root_was_created, "the essential genome declares no root of its own");
    assert_eq!(genome.brain_regions.len(), 2, "expected a root plus the synthesized subregion");

    let root = &genome.brain_regions[&root_region_id];
    let subregion = genome
        .brain_regions
        .values()
        .find(|region| region.properties.get("parent_region_id").and_then(|id| id.as_str()) == Some(root_region_id.as_str()))
        .expect("the subregion names the root as its parent");

    // The genome's twelve custom areas belong to the subregion, the sensory, motor and core areas
    // to the root, and no area belongs to both.
    assert_eq!(subregion.cortical_areas.len(), 12);
    for area_id in &subregion.cortical_areas {
        assert!(!root.contains_area(area_id), "area {area_id} is on both plates");
    }
    assert_eq!(root.cortical_areas.len() + subregion.cortical_areas.len(), genome.cortical_areas.len());

    // Both plates need a position and ports, or the visualizer draws them stacked and unconnected.
    assert!(subregion.properties.contains_key("coordinate_3d"));
    assert!(subregion.properties.contains_key("inputs"), "subregion should declare its inbound ports");
    assert!(root.properties.contains_key("outputs"), "root should declare its outbound ports");
}

/// The barebones genome declares only core areas, and the same no-silent-drop rule applies.
#[test]
fn loading_the_barebones_genome_publishes_every_declared_area() {
    let declared = cortical_ids_declared_in_flat_genome(feagi_evolutionary::BAREBONES_GENOME_JSON);
    let genome = feagi_evolutionary::load_genome_from_json(feagi_evolutionary::BAREBONES_GENOME_JSON).expect("barebones genome loads");

    assert_eq!(
        genome.cortical_areas.len(),
        declared.len(),
        "genome declares {} areas but {} were loaded",
        declared.len(),
        genome.cortical_areas.len()
    );

    let with_core_areas = load_barebones_genome().expect("barebones genome loads with core areas");
    assert!(with_core_areas.cortical_areas.len() >= declared.len());
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
