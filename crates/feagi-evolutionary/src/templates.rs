// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Genome cortical_units for FEAGI.

Provides cortical_units for creating genomes from scratch, including:
- Minimal genome template
- Cortical area cortical_units (IPU, OPU, CORE)
- Default neural parameters
- Embedded default genomes

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::{GenomeMetadata, GenomeSignatures, GenomeStats, MorphologyRegistry, PhysiologyConfig, RuntimeGenome};
use feagi_data::neurons::voxel_potentials::wrapped_values::NeuronVoxelDimensionsGenomic;
use feagi_genomic_context::brain_region::{BrainRegion, RegionID, RegionType};
use feagi_genomic_context::cortical_area::{CoreCorticalType, CorticalID};
use feagi_genomic_context::genome_positioning::GenomeCoordinate3D;
use feagi_genomic_data::cortical_area_prev::CorticalArea;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// Embedded essential genome (loaded at compile time)
pub const ESSENTIAL_GENOME_JSON: &str = include_str!("../genomes/essential_genome.json");

/// Embedded barebones genome (loaded at compile time)
pub const BAREBONES_GENOME_JSON: &str = include_str!("../genomes/barebones_genome.json");

/// Embedded test genome (loaded at compile time)
pub const TEST_GENOME_JSON: &str = include_str!("../genomes/test_genome.json");

/// Embedded vision genome (loaded at compile time)
pub const VISION_GENOME_JSON: &str = include_str!("../genomes/vision_genome.json");

/// Default neural properties for all cortical_area areas
///
/// These populate a runtime [`CorticalArea::properties`] map, so they use the runtime property
/// names the rest of the system reads. Where a genome file spells a property differently, the
/// parser translates it on the way in; `per_voxel_neuron_cnt` in a genome document becomes
/// `neurons_per_voxel` here.
pub fn get_default_neural_properties() -> HashMap<String, Value> {
    let mut props = HashMap::new();
    props.insert("neurons_per_voxel".to_string(), Value::from(1));
    props.insert("synapse_attractivity".to_string(), Value::from(100.0));
    props.insert("degeneration".to_string(), Value::from(0.0));
    props.insert("psp_uniform_distribution".to_string(), Value::from(true));
    props.insert("postsynaptic_current_max".to_string(), Value::from(10000.0));
    props.insert("postsynaptic_current".to_string(), Value::from(500.0));
    props.insert("firing_threshold".to_string(), Value::from(0.1));
    props.insert("refractory_period".to_string(), Value::from(0));
    props.insert("leak_coefficient".to_string(), Value::from(0.0));
    props.insert("leak_variability".to_string(), Value::from(0.0));
    props.insert("consecutive_fire_cnt_max".to_string(), Value::from(0));
    props.insert("snooze_length".to_string(), Value::from(0));
    props.insert("mp_charge_accumulation".to_string(), Value::from(false));
    props.insert("mp_driven_psp".to_string(), Value::from(false));
    props.insert("neuron_excitability".to_string(), Value::from(1.0));
    props.insert("visualization".to_string(), Value::from(true));
    props.insert("memory_twin_of".to_string(), Value::Null);
    props.insert("cortical_mapping_dst".to_string(), Value::Object(serde_json::Map::new()));
    props
}

/// Create _death cortical_area area (cortical_idx = 0) from template
pub fn create_death_area() -> CorticalArea {
    let cortical_id = CoreCorticalType::Death.to_cortical_id();
    let cortical_type = cortical_id.as_cortical_type().expect("Death cortical_area ID should map to Core type");

    let mut area = CorticalArea::new(
        cortical_id,
        0, // cortical_idx = 0 (reserved)
        "Death".to_string(),
        NeuronVoxelDimensionsGenomic::new_from_usizes_unchecked(1, 1, 1),
        GenomeCoordinate3D::new(0, 0, 20),
        cortical_type,
    )
    .expect("Failed to create _death area");

    let mut props = get_default_neural_properties();
    props.insert("cortical_group".to_string(), Value::from("CORE"));
    props.insert("2d_coordinate".to_string(), Value::from(vec![-10, -20]));
    area.properties = props;
    area
}

/// Create _power cortical_area area (cortical_idx = 1) from template
pub fn create_power_area() -> CorticalArea {
    let cortical_id = CoreCorticalType::Power.to_cortical_id();
    let cortical_type = cortical_id.as_cortical_type().expect("Power cortical_area ID should map to Core type");

    let mut area = CorticalArea::new(
        cortical_id,
        1, // cortical_idx = 1 (reserved)
        "Brain_Power".to_string(),
        NeuronVoxelDimensionsGenomic::new_from_usizes_unchecked(1, 1, 1),
        GenomeCoordinate3D::new(0, 0, 20),
        cortical_type,
    )
    .expect("Failed to create _power area");

    let mut props = get_default_neural_properties();
    props.insert("cortical_group".to_string(), Value::from("CORE"));
    props.insert("2d_coordinate".to_string(), Value::from(vec![-10, -10]));
    props.insert("firing_threshold".to_string(), Value::from(0.1));
    props.insert("postsynaptic_current".to_string(), Value::from(500.0));
    props.insert("neuron_excitability".to_string(), Value::from(100.0));
    area.properties = props;
    area
}

/// Create _fatigue cortical_area area (cortical_idx = 2) from template
pub fn create_fatigue_area() -> CorticalArea {
    let cortical_id = CoreCorticalType::Fatigue.to_cortical_id();
    let cortical_type = cortical_id.as_cortical_type().expect("Fatigue cortical_area ID should map to Core type");

    let mut area = CorticalArea::new(
        cortical_id,
        2, // cortical_idx = 2 (reserved)
        "Fatigue".to_string(),
        NeuronVoxelDimensionsGenomic::new_from_usizes_unchecked(1, 1, 1),
        GenomeCoordinate3D::new(0, 0, 20),
        cortical_type,
    )
    .expect("Failed to create _fatigue area");

    let mut props = get_default_neural_properties();
    props.insert("cortical_group".to_string(), Value::from("CORE"));
    props.insert("2d_coordinate".to_string(), Value::from(vec![-10, 0]));
    area.properties = props;
    area
}

/// Create _pain cortical_area area (cortical_idx = 3) from template
pub fn create_pain_area() -> CorticalArea {
    let cortical_id = CoreCorticalType::Pain.to_cortical_id();
    let cortical_type = cortical_id.as_cortical_type().expect("Pain cortical_area ID should map to Core type");

    let mut area = CorticalArea::new(
        cortical_id,
        3, // cortical_idx = 3 (reserved)
        "Pain".to_string(),
        NeuronVoxelDimensionsGenomic::new_from_usizes_unchecked(1, 1, 1),
        GenomeCoordinate3D::new(0, 0, 20),
        cortical_type,
    )
    .expect("Failed to create _pain area");

    let mut props = get_default_neural_properties();
    props.insert("cortical_group".to_string(), Value::from("CORE"));
    props.insert("2d_coordinate".to_string(), Value::from(vec![-10, 10]));
    area.properties = props;
    area
}

/// Create _pleasure cortical_area area (cortical_idx = 4) from template
pub fn create_pleasure_area() -> CorticalArea {
    let cortical_id = CoreCorticalType::Pleasure.to_cortical_id();
    let cortical_type = cortical_id.as_cortical_type().expect("Pleasure cortical_area ID should map to Core type");

    let mut area = CorticalArea::new(
        cortical_id,
        4, // cortical_idx = 4 (reserved)
        "Pleasure".to_string(),
        NeuronVoxelDimensionsGenomic::new_from_usizes_unchecked(1, 1, 1),
        GenomeCoordinate3D::new(0, 0, 20),
        cortical_type,
    )
    .expect("Failed to create _pleasure area");

    let mut props = get_default_neural_properties();
    props.insert("cortical_group".to_string(), Value::from("CORE"));
    props.insert("2d_coordinate".to_string(), Value::from(vec![-10, 20]));
    area.properties = props;
    area
}

/// Create _fear cortical_area area (cortical_idx = 5) from template
pub fn create_fear_area() -> CorticalArea {
    let cortical_id = CoreCorticalType::Fear.to_cortical_id();
    let cortical_type = cortical_id.as_cortical_type().expect("Fear cortical_area ID should map to Core type");

    let mut area = CorticalArea::new(
        cortical_id,
        5, // cortical_idx = 5 (reserved)
        "Fear".to_string(),
        NeuronVoxelDimensionsGenomic::new_from_usizes_unchecked(1, 1, 1),
        GenomeCoordinate3D::new(0, 0, 20),
        cortical_type,
    )
    .expect("Failed to create _fear area");

    let mut props = get_default_neural_properties();
    props.insert("cortical_group".to_string(), Value::from("CORE"));
    props.insert("2d_coordinate".to_string(), Value::from(vec![-10, 30]));
    area.properties = props;
    area
}

/// Create _hope cortical_area area (cortical_idx = 6) from template
pub fn create_hope_area() -> CorticalArea {
    let cortical_id = CoreCorticalType::Hope.to_cortical_id();
    let cortical_type = cortical_id.as_cortical_type().expect("Hope cortical_area ID should map to Core type");

    let mut area = CorticalArea::new(
        cortical_id,
        6, // cortical_idx = 6 (reserved)
        "Hope".to_string(),
        NeuronVoxelDimensionsGenomic::new_from_usizes_unchecked(1, 1, 1),
        GenomeCoordinate3D::new(0, 0, 20),
        cortical_type,
    )
    .expect("Failed to create _hope area");

    let mut props = get_default_neural_properties();
    props.insert("cortical_group".to_string(), Value::from("CORE"));
    props.insert("2d_coordinate".to_string(), Value::from(vec![-10, 40]));
    area.properties = props;
    area
}

/// Create a minimal empty genome
pub fn create_minimal_genome(genome_id: String, genome_title: String) -> RuntimeGenome {
    RuntimeGenome {
        metadata: GenomeMetadata {
            genome_id,
            genome_title,
            genome_description: "Minimal genome template".to_string(),
            version: "2.0".to_string(),
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64(),
            brain_regions_root: None, // Will be set after neuroembryogenesis
        },
        cortical_areas: HashMap::new(),
        brain_regions: HashMap::new(),
        morphologies: MorphologyRegistry::new(),
        physiology: PhysiologyConfig::default(),
        signatures: GenomeSignatures {
            genome: String::new(),
            blueprint: String::new(),
            physiology: String::new(),
            morphologies: None,
        },
        stats: GenomeStats::default(),
    }
}

/// Create a genome with core areas (_death, _power, _fatigue, _pain, _pleasure, _fear, _hope)
pub fn create_genome_with_core_areas(genome_id: String, genome_title: String) -> RuntimeGenome {
    let mut genome = create_minimal_genome(genome_id, genome_title);

    // Add core areas (convert 6-char strings to CorticalID)
    let death_id = crate::genome::parser::string_to_cortical_id("_death").expect("Valid cortical_area ID");
    let power_id = crate::genome::parser::string_to_cortical_id("_power").expect("Valid cortical_area ID");
    let fatigue_id = crate::genome::parser::string_to_cortical_id("_fatigue").expect("Valid cortical_area ID");
    let pain_id = crate::genome::parser::string_to_cortical_id("_pain").expect("Valid cortical_area ID");
    let pleasure_id = crate::genome::parser::string_to_cortical_id("_pleasure").expect("Valid cortical_area ID");
    let fear_id = crate::genome::parser::string_to_cortical_id("_fear").expect("Valid cortical_area ID");
    let hope_id = crate::genome::parser::string_to_cortical_id("_hope").expect("Valid cortical_area ID");

    genome.cortical_areas.insert(death_id, create_death_area());
    genome.cortical_areas.insert(power_id, create_power_area());
    genome.cortical_areas.insert(fatigue_id, create_fatigue_area());
    genome.cortical_areas.insert(pain_id, create_pain_area());
    genome.cortical_areas.insert(pleasure_id, create_pleasure_area());
    genome.cortical_areas.insert(fear_id, create_fear_area());
    genome.cortical_areas.insert(hope_id, create_hope_area());

    genome
}

/// Create a genome with core morphologies
pub fn create_genome_with_core_morphologies(genome_id: String, genome_title: String) -> RuntimeGenome {
    let mut genome = create_minimal_genome(genome_id, genome_title);

    // Add core morphologies
    add_core_morphologies(&mut genome.morphologies);

    genome
}

/// CRITICAL: Ensure a genome has all required core components
///
/// This function checks if a genome has:
/// 1. Core cortical_area areas (_death, _power)
/// 2. Core morphologies (block_to_block, projector, etc.)
///
/// If any are missing, they are automatically added. This ensures every genome
/// can function properly regardless of its source.
///
/// # Arguments
/// * `genome` - The genome to validate and fix
///
/// # Returns
/// A tuple of (areas_added, morphologies_added) indicating what was added
pub fn ensure_core_components(genome: &mut RuntimeGenome) -> (usize, usize) {
    let mut areas_added = 0;
    let mut morphologies_added = 0;

    // Convert core area IDs
    let death_id = crate::genome::parser::string_to_cortical_id("_death").expect("Valid cortical_area ID");
    let power_id = crate::genome::parser::string_to_cortical_id("_power").expect("Valid cortical_area ID");
    let fatigue_id = crate::genome::parser::string_to_cortical_id("_fatigue").expect("Valid cortical_area ID");
    let pain_id = crate::genome::parser::string_to_cortical_id("_pain").expect("Valid cortical_area ID");
    let pleasure_id = crate::genome::parser::string_to_cortical_id("_pleasure").expect("Valid cortical_area ID");
    let fear_id = crate::genome::parser::string_to_cortical_id("_fear").expect("Valid cortical_area ID");
    let hope_id = crate::genome::parser::string_to_cortical_id("_hope").expect("Valid cortical_area ID");

    // 1. Ensure core cortical_area areas exist
    if let std::collections::hash_map::Entry::Vacant(e) = genome.cortical_areas.entry(death_id) {
        let death_area = create_death_area();
        e.insert(death_area);
        areas_added += 1;
        tracing::info!("Added missing core area: _death (cortical_idx=0)");
    }

    if let std::collections::hash_map::Entry::Vacant(e) = genome.cortical_areas.entry(power_id) {
        let power_area = create_power_area();
        e.insert(power_area);
        areas_added += 1;
        tracing::info!("Added missing core area: _power (cortical_idx=1)");
    }

    if let std::collections::hash_map::Entry::Vacant(e) = genome.cortical_areas.entry(fatigue_id) {
        let fatigue_area = create_fatigue_area();
        e.insert(fatigue_area);
        areas_added += 1;
        tracing::info!("Added missing core area: _fatigue (cortical_idx=2)");
    }

    if let std::collections::hash_map::Entry::Vacant(e) = genome.cortical_areas.entry(pain_id) {
        let pain_area = create_pain_area();
        e.insert(pain_area);
        areas_added += 1;
        tracing::info!("Added missing core area: _pain (cortical_idx=3)");
    }

    if let std::collections::hash_map::Entry::Vacant(e) = genome.cortical_areas.entry(pleasure_id) {
        let pleasure_area = create_pleasure_area();
        e.insert(pleasure_area);
        areas_added += 1;
        tracing::info!("Added missing core area: _pleasure (cortical_idx=4)");
    }

    if let std::collections::hash_map::Entry::Vacant(e) = genome.cortical_areas.entry(fear_id) {
        let fear_area = create_fear_area();
        e.insert(fear_area);
        areas_added += 1;
        tracing::info!("Added missing core area: _fear (cortical_idx=5)");
    }

    if let std::collections::hash_map::Entry::Vacant(e) = genome.cortical_areas.entry(hope_id) {
        let hope_area = create_hope_area();
        e.insert(hope_area);
        areas_added += 1;
        tracing::info!("Added missing core area: _hope (cortical_idx=6)");
    }

    // 2. Ensure core morphologies exist
    let required_morphologies = vec![
        "block_to_block",
        "projector",
        "centered_projector",
        "transpose_xy",
        "transpose_yz",
        "transpose_xz",
        "sweeper",
        "last_to_first",
        "first_to_last",
        "bitmask_encoder_x",
        "bitmask_encoder_y",
        "bitmask_encoder_z",
        "bitmask_decoder_x",
        "bitmask_decoder_y",
        "bitmask_decoder_z",
        "episodic_memory",
        "memory_replay",
        "associative_memory",
        "rotator_z",
        "all_to_0-0-0",
        "0-0-0_to_all",
        "tile",
        "lateral_+x",
        "lateral_-x",
        "lateral_+y",
        "lateral_-y",
        "lateral_+z",
        "lateral_-z",
    ];

    for morph_name in required_morphologies {
        if !genome.morphologies.contains(morph_name) {
            morphologies_added += 1;
        }
    }

    // Add all missing core morphologies in one call
    if morphologies_added > 0 {
        add_core_morphologies(&mut genome.morphologies);
        tracing::info!("Added {} missing core morphologies", morphologies_added);
    }

    (areas_added, morphologies_added)
}

/// Name given to a root region created here because the genome document declared none.
///
/// Matches the name the pre-refactor development path used, so a genome that was saved with an
/// auto-created root before this change still resolves to the same region rather than gaining a
/// second one.
pub const SYNTHESIZED_ROOT_REGION_NAME: &str = "Root Brain Region";

/// Property under which a region records its parent, spelled as the genome document spells it.
const PARENT_REGION_ID_KEY: &str = "parent_region_id";

/// Name for the synthesized subregion when the genome carries no title worth showing.
const AUTOGEN_SUBREGION_FALLBACK_NAME: &str = "Autogen Circuit";

/// Region properties the brain visualizer reads to place a region and draw its ports.
const REGION_INPUTS_KEY: &str = "inputs";
const REGION_OUTPUTS_KEY: &str = "outputs";
const REGION_COORDINATE_2D_KEY: &str = "coordinate_2d";
const REGION_COORDINATE_3D_KEY: &str = "coordinate_3d";

/// Area property holding the area's outbound mappings, keyed by destination cortical ID.
const CORTICAL_MAPPING_DST_KEY: &str = "cortical_mapping_dst";

/// CRITICAL: Ensure a genome describes a brain region hierarchy with one reachable root
///
/// Every consumer of the hierarchy - the brain visualizer, the region endpoints, the health
/// check's `brain_regions_root` - locates the root as the region that names no parent and walks
/// downward from there. A genome document is not required to carry a `brain_regions` section at
/// all (no v2 genome does, including the barebones and essential templates), so without this the
/// hierarchy is empty, no cortical area can be placed in a region, and the visualizer cannot
/// build its cache.
///
/// When no root is declared, one is created holding the sensory, motor and core areas that no
/// other region claims. Custom and memory areas are gathered into a subregion beneath it rather
/// than sharing the root plate, which is how the pre-refactor development path arranged a
/// region-less genome and what the visualizer draws its region plates from: a root holding
/// everything gives it one plate and nothing to nest. Both regions declare the areas that carry
/// traffic across their boundary as ports.
///
/// Regions whose declared parent is absent from the genome are attached to the root so the whole
/// hierarchy stays reachable from it.
///
/// # Arguments
/// * `genome` - The genome to normalize; `metadata.brain_regions_root` is stamped with the result
///
/// # Returns
/// A tuple of (root region ID, whether the root was created here rather than declared)
pub fn ensure_root_brain_region(genome: &mut RuntimeGenome) -> (String, bool) {
    if let Some(root_region_id) = declared_root_region_id(genome) {
        genome.metadata.brain_regions_root = Some(root_region_id.clone());
        return (root_region_id, false);
    }

    let declared_region_ids: HashSet<String> = genome.brain_regions.keys().cloned().collect();
    let claimed_areas: HashSet<CorticalID> = genome
        .brain_regions
        .values()
        .flat_map(|region| region.cortical_areas.iter().copied())
        .collect();

    // Sorted so that two loads of the same genome describe their regions identically; the genome
    // holds its areas in a hash map, whose order is not stable across runs.
    let mut unclaimed_areas: Vec<CorticalID> = genome
        .cortical_areas
        .keys()
        .copied()
        .filter(|area_id| !claimed_areas.contains(area_id))
        .collect();
    unclaimed_areas.sort_by_key(|area_id| area_id.as_base_64());
    let (subregion_areas, root_areas): (Vec<CorticalID>, Vec<CorticalID>) =
        unclaimed_areas.into_iter().partition(|area_id| belongs_in_autogen_subregion(area_id));

    let root_region_id = RegionID::new();
    let root_region_id_str = root_region_id.to_string();
    let mut root_region = BrainRegion::new(root_region_id, SYNTHESIZED_ROOT_REGION_NAME.to_string(), RegionType::Undefined)
        .expect("the root region name is a non-empty constant")
        .with_areas(root_areas.iter().copied());
    declare_region_ports(&mut root_region, &root_areas, genome);

    let subregion_area_count = subregion_areas.len();
    let subregion = (!subregion_areas.is_empty()).then(|| {
        let subregion_id = RegionID::new();
        let mut subregion = BrainRegion::new(
            subregion_id,
            autogen_subregion_display_name(&genome.metadata.genome_title),
            RegionType::Undefined,
        )
        .expect("the subregion name is never empty")
        .with_areas(subregion_areas.iter().copied());

        subregion.add_property(PARENT_REGION_ID_KEY.to_string(), Value::String(root_region_id_str.clone()));
        // Placed clear of the root's areas so the two plates do not overlap in the visualizer.
        subregion.add_property(
            REGION_COORDINATE_3D_KEY.to_string(),
            Value::from(autogen_subregion_position(&root_areas, genome).to_vec()),
        );
        subregion.add_property(REGION_COORDINATE_2D_KEY.to_string(), Value::from(vec![0, 0]));
        declare_region_ports(&mut subregion, &subregion_areas, genome);

        (subregion_id.to_string(), subregion)
    });

    // No region is parentless here, or the branch above would have returned, so every region left
    // names a parent. One naming a parent this genome does not contain is unreachable from the
    // root, and the hierarchy is only ever traversed downward from the root.
    let mut reparented_regions = 0;
    for region in genome.brain_regions.values_mut() {
        let parent_is_present = region
            .properties
            .get(PARENT_REGION_ID_KEY)
            .and_then(|parent_id| parent_id.as_str())
            .is_some_and(|parent_id| declared_region_ids.contains(parent_id));
        if parent_is_present {
            continue;
        }
        region.add_property(PARENT_REGION_ID_KEY.to_string(), Value::String(root_region_id_str.clone()));
        reparented_regions += 1;
    }

    let root_area_count = root_region.cortical_areas.len();
    genome.brain_regions.insert(root_region_id_str.clone(), root_region);
    if let Some((subregion_id, subregion)) = subregion {
        genome.brain_regions.insert(subregion_id, subregion);
    }
    genome.metadata.brain_regions_root = Some(root_region_id_str.clone());

    tracing::info!(
        target: "feagi-evo",
        "Genome declared no root brain region; created '{}' ({}) holding {} cortical area(s), a subregion holding {} custom/memory area(s), {} region(s) reparented under the root",
        SYNTHESIZED_ROOT_REGION_NAME,
        root_region_id_str,
        root_area_count,
        subregion_area_count,
        reparented_regions
    );

    (root_region_id_str, true)
}

/// Whether an area belongs in the synthesized subregion rather than on the root plate.
///
/// Sensory, motor and core areas are the genome's fixed scaffolding and stay on the root. Custom
/// and memory areas are what a circuit actually contributes, so they are the ones worth grouping.
fn belongs_in_autogen_subregion(area_id: &CorticalID) -> bool {
    use feagi_genomic_context::cortical_area::CorticalAreaType;

    matches!(
        area_id.as_cortical_type(),
        Ok(CorticalAreaType::Custom(_)) | Ok(CorticalAreaType::Memory(_))
    )
}

/// The subregion's display name: the genome's own title, since a circuit is usually loaded under
/// the name it was saved as, and a generic label only when the genome supplies nothing.
fn autogen_subregion_display_name(genome_title: &str) -> String {
    let title = genome_title.trim();
    if title.is_empty() || title.eq_ignore_ascii_case("untitled") {
        AUTOGEN_SUBREGION_FALLBACK_NAME.to_string()
    } else {
        title.to_string()
    }
}

/// Record which of a region's areas carry traffic across its boundary.
///
/// An area is an output when it maps to somewhere outside the region and an input when something
/// outside maps into it. The visualizer draws a region's ports from these two lists, so a region
/// without them reads as isolated no matter how its areas are wired.
fn declare_region_ports(region: &mut BrainRegion, region_areas: &[CorticalID], genome: &RuntimeGenome) {
    let (inputs, outputs) = analyze_region_io(region_areas, genome);
    if !inputs.is_empty() {
        region.add_property(REGION_INPUTS_KEY.to_string(), Value::from(inputs));
    }
    if !outputs.is_empty() {
        region.add_property(REGION_OUTPUTS_KEY.to_string(), Value::from(outputs));
    }
}

/// The areas of `region_areas` that receive traffic from outside the region, and those that send
/// traffic outside it, each named by base64 cortical ID.
fn analyze_region_io(region_areas: &[CorticalID], genome: &RuntimeGenome) -> (Vec<String>, Vec<String>) {
    use crate::genome::parser::string_to_cortical_id;

    /// The destinations an area maps to. Keys are cortical IDs; the parser has already rewritten
    /// them to base64, but a genome may still be read before that pass, so each is resolved rather
    /// than compared as text.
    fn destinations_of(area: &CorticalArea) -> Vec<CorticalID> {
        area.properties
            .get(CORTICAL_MAPPING_DST_KEY)
            .and_then(|mappings| mappings.as_object())
            .map(|mappings| mappings.keys().filter_map(|dst| string_to_cortical_id(dst).ok()).collect())
            .unwrap_or_default()
    }

    let members: HashSet<CorticalID> = region_areas.iter().copied().collect();

    let mut outputs: Vec<String> = Vec::new();
    for area_id in region_areas {
        let Some(area) = genome.cortical_areas.get(area_id) else {
            continue;
        };
        if destinations_of(area).iter().any(|dst| !members.contains(dst)) {
            outputs.push(area_id.as_base_64());
        }
    }

    let mut inputs: Vec<String> = Vec::new();
    let mut seen_inputs: HashSet<CorticalID> = HashSet::new();
    for (source_id, source) in genome.cortical_areas.iter() {
        if members.contains(source_id) {
            continue;
        }
        for destination in destinations_of(source) {
            if members.contains(&destination) && seen_inputs.insert(destination) {
                inputs.push(destination.as_base_64());
            }
        }
    }
    inputs.sort();

    (inputs, outputs)
}

/// Where to place the synthesized subregion: clear of the root areas' bounding box along x, and
/// centred on it in the other two axes, so the two plates read as neighbours rather than overlap.
fn autogen_subregion_position(root_areas: &[CorticalID], genome: &RuntimeGenome) -> [i32; 3] {
    /// Distance kept between the two plates when the root's own extent is too small to scale from.
    const MINIMUM_PADDING: i32 = 50;

    let mut bounds: Option<([i32; 3], [i32; 3])> = None;
    for area_id in root_areas {
        let Some(area) = genome.cortical_areas.get(area_id) else {
            continue;
        };
        let low = [area.position.x(), area.position.y(), area.position.z()];
        let high = [
            low[0].saturating_add(*area.dimensions.get_x().as_ref() as i32),
            low[1].saturating_add(*area.dimensions.get_y().as_ref() as i32),
            low[2].saturating_add(*area.dimensions.get_z().as_ref() as i32),
        ];
        bounds = Some(match bounds {
            None => (low, high),
            Some((mut min, mut max)) => {
                for axis in 0..3 {
                    min[axis] = min[axis].min(low[axis]);
                    max[axis] = max[axis].max(high[axis]);
                }
                (min, max)
            }
        });
    }

    let Some((min, max)) = bounds else {
        // Nothing on the root plate to sit beside, so any placement off the origin will do.
        return [MINIMUM_PADDING * 2, 0, 0];
    };

    let padding = ((max[0] - min[0]).max(1) / 5).max(MINIMUM_PADDING);
    [max[0].saturating_add(padding), (min[1] + max[1]) / 2, (min[2] + max[2]) / 2]
}

/// The ID of the region the genome declares as its root: the one that names no parent.
///
/// A genome with more than one parentless region is malformed, but reading it must still be
/// repeatable, so the lowest ID wins rather than whichever the map happens to yield first.
fn declared_root_region_id(genome: &RuntimeGenome) -> Option<String> {
    genome
        .brain_regions
        .iter()
        .filter(|(_, region)| !matches!(region.properties.get(PARENT_REGION_ID_KEY), Some(parent_id) if !parent_id.is_null()))
        .map(|(region_id, _)| region_id.clone())
        .min()
}

/// Add core morphologies to a registry
pub fn add_core_morphologies(registry: &mut MorphologyRegistry) {
    use crate::{Morphology, MorphologyParameters, MorphologyType};

    // block_to_block - Connect neurons in same position
    registry.add_morphology(
        "block_to_block".to_string(),
        Morphology {
            morphology_type: MorphologyType::Vectors,
            parameters: MorphologyParameters::Vectors { vectors: vec![[0, 0, 0]] },
            class: "core".to_string(),
        },
    );

    // projector - Function-based morphology
    registry.add_morphology(
        "projector".to_string(),
        Morphology {
            morphology_type: MorphologyType::Functions,
            parameters: MorphologyParameters::Functions {},
            class: "core".to_string(),
        },
    );

    // centered_projector - Function-based center-aligned 1:1 mapping morphology
    registry.add_morphology(
        "centered_projector".to_string(),
        Morphology {
            morphology_type: MorphologyType::Functions,
            parameters: MorphologyParameters::Functions {},
            class: "core".to_string(),
        },
    );

    // transpose_xy - Projector with x/y axis transposition
    registry.add_morphology(
        "transpose_xy".to_string(),
        Morphology {
            morphology_type: MorphologyType::Functions,
            parameters: MorphologyParameters::Functions {},
            class: "core".to_string(),
        },
    );

    // transpose_yz - Projector with y/z axis transposition
    registry.add_morphology(
        "transpose_yz".to_string(),
        Morphology {
            morphology_type: MorphologyType::Functions,
            parameters: MorphologyParameters::Functions {},
            class: "core".to_string(),
        },
    );

    // transpose_xz - Projector with x/z axis transposition
    registry.add_morphology(
        "transpose_xz".to_string(),
        Morphology {
            morphology_type: MorphologyType::Functions,
            parameters: MorphologyParameters::Functions {},
            class: "core".to_string(),
        },
    );

    // sweeper - Function-based sequential sweep mapping morphology
    registry.add_morphology(
        "sweeper".to_string(),
        Morphology {
            morphology_type: MorphologyType::Functions,
            parameters: MorphologyParameters::Functions {},
            class: "core".to_string(),
        },
    );

    // last_to_first - connect highest source voxel to destination origin
    registry.add_morphology(
        "last_to_first".to_string(),
        Morphology {
            morphology_type: MorphologyType::Functions,
            parameters: MorphologyParameters::Functions {},
            class: "core".to_string(),
        },
    );

    // first_to_last - connect source origin to highest destination voxel
    registry.add_morphology(
        "first_to_last".to_string(),
        Morphology {
            morphology_type: MorphologyType::Functions,
            parameters: MorphologyParameters::Functions {},
            class: "core".to_string(),
        },
    );

    // episodic_memory - Function-based morphology
    registry.add_morphology(
        "episodic_memory".to_string(),
        Morphology {
            morphology_type: MorphologyType::Functions,
            parameters: MorphologyParameters::Functions {},
            class: "core".to_string(),
        },
    );

    // memory_replay - Function-based morphology
    registry.add_morphology(
        "memory_replay".to_string(),
        Morphology {
            morphology_type: MorphologyType::Functions,
            parameters: MorphologyParameters::Functions {},
            class: "core".to_string(),
        },
    );

    // associative_memory (bi-directional STDP) - Function-based morphology
    registry.add_morphology(
        "associative_memory".to_string(),
        Morphology {
            morphology_type: MorphologyType::Functions,
            parameters: MorphologyParameters::Functions {},
            class: "core".to_string(),
        },
    );

    // rotator_z - Function-based morphology for z-layered XY rotations [-90,+90]
    registry.add_morphology(
        "rotator_z".to_string(),
        Morphology {
            morphology_type: MorphologyType::Functions,
            parameters: MorphologyParameters::Functions {},
            class: "core".to_string(),
        },
    );

    // bitmask_encoder_x - Bitmask encode along X axis
    registry.add_morphology(
        "bitmask_encoder_x".to_string(),
        Morphology {
            morphology_type: MorphologyType::Functions,
            parameters: MorphologyParameters::Functions {},
            class: "core".to_string(),
        },
    );

    // bitmask_encoder_y - Bitmask encode along Y axis
    registry.add_morphology(
        "bitmask_encoder_y".to_string(),
        Morphology {
            morphology_type: MorphologyType::Functions,
            parameters: MorphologyParameters::Functions {},
            class: "core".to_string(),
        },
    );

    // bitmask_encoder_z - Bitmask encode along Z axis
    registry.add_morphology(
        "bitmask_encoder_z".to_string(),
        Morphology {
            morphology_type: MorphologyType::Functions,
            parameters: MorphologyParameters::Functions {},
            class: "core".to_string(),
        },
    );

    // bitmask_decoder_x - Bitmask decode along X axis
    registry.add_morphology(
        "bitmask_decoder_x".to_string(),
        Morphology {
            morphology_type: MorphologyType::Functions,
            parameters: MorphologyParameters::Functions {},
            class: "core".to_string(),
        },
    );

    // bitmask_decoder_y - Bitmask decode along Y axis
    registry.add_morphology(
        "bitmask_decoder_y".to_string(),
        Morphology {
            morphology_type: MorphologyType::Functions,
            parameters: MorphologyParameters::Functions {},
            class: "core".to_string(),
        },
    );

    // bitmask_decoder_z - Bitmask decode along Z axis
    registry.add_morphology(
        "bitmask_decoder_z".to_string(),
        Morphology {
            morphology_type: MorphologyType::Functions,
            parameters: MorphologyParameters::Functions {},
            class: "core".to_string(),
        },
    );

    // all_to_0-0-0 - Connect all neurons to origin
    registry.add_morphology(
        "all_to_0-0-0".to_string(),
        Morphology {
            morphology_type: MorphologyType::Patterns,
            parameters: MorphologyParameters::Patterns {
                patterns: vec![[
                    vec![
                        crate::PatternElement::Wildcard,
                        crate::PatternElement::Wildcard,
                        crate::PatternElement::Wildcard,
                    ],
                    vec![
                        crate::PatternElement::Value(0),
                        crate::PatternElement::Value(0),
                        crate::PatternElement::Value(0),
                    ],
                ]],
            },
            class: "core".to_string(),
        },
    );

    // 0-0-0_to_all - Connect origin to all neurons
    registry.add_morphology(
        "0-0-0_to_all".to_string(),
        Morphology {
            morphology_type: MorphologyType::Patterns,
            parameters: MorphologyParameters::Patterns {
                patterns: vec![[
                    vec![
                        crate::PatternElement::Value(0),
                        crate::PatternElement::Value(0),
                        crate::PatternElement::Value(0),
                    ],
                    vec![
                        crate::PatternElement::Wildcard,
                        crate::PatternElement::Wildcard,
                        crate::PatternElement::Wildcard,
                    ],
                ]],
            },
            class: "core".to_string(),
        },
    );

    // tile - Composite tiling morphology (mapper + subregion parameters)
    registry.add_morphology(
        "tile".to_string(),
        Morphology {
            morphology_type: MorphologyType::Composite,
            parameters: MorphologyParameters::Composite {
                src_seed: [16, 16, 1],
                src_pattern: vec![[1, 0], [1, 0], [1, 0]],
                mapper_morphology: "projector".to_string(),
            },
            class: "core".to_string(),
        },
    );

    // lateral_+x - Connect along +X axis
    registry.add_morphology(
        "lateral_+x".to_string(),
        Morphology {
            morphology_type: MorphologyType::Vectors,
            parameters: MorphologyParameters::Vectors { vectors: vec![[1, 0, 0]] },
            class: "core".to_string(),
        },
    );

    // lateral_-x - Connect along -X axis
    registry.add_morphology(
        "lateral_-x".to_string(),
        Morphology {
            morphology_type: MorphologyType::Vectors,
            parameters: MorphologyParameters::Vectors { vectors: vec![[-1, 0, 0]] },
            class: "core".to_string(),
        },
    );

    // lateral_+y - Connect along +Y axis
    registry.add_morphology(
        "lateral_+y".to_string(),
        Morphology {
            morphology_type: MorphologyType::Vectors,
            parameters: MorphologyParameters::Vectors { vectors: vec![[0, 1, 0]] },
            class: "core".to_string(),
        },
    );

    // lateral_-y - Connect along -Y axis
    registry.add_morphology(
        "lateral_-y".to_string(),
        Morphology {
            morphology_type: MorphologyType::Vectors,
            parameters: MorphologyParameters::Vectors { vectors: vec![[0, -1, 0]] },
            class: "core".to_string(),
        },
    );

    // lateral_+z - Connect along +Z axis
    registry.add_morphology(
        "lateral_+z".to_string(),
        Morphology {
            morphology_type: MorphologyType::Vectors,
            parameters: MorphologyParameters::Vectors { vectors: vec![[0, 0, 1]] },
            class: "core".to_string(),
        },
    );

    // lateral_-z - Connect along -Z axis
    registry.add_morphology(
        "lateral_-z".to_string(),
        Morphology {
            morphology_type: MorphologyType::Vectors,
            parameters: MorphologyParameters::Vectors { vectors: vec![[0, 0, -1]] },
            class: "core".to_string(),
        },
    );
}

/// Load essential genome from embedded JSON
///
/// Automatically ensures core components (_death, _power, core morphologies) are present
pub fn load_essential_genome() -> Result<RuntimeGenome, crate::types::EvoError> {
    use crate::genome::loader::load_genome_from_json;
    let mut genome = load_genome_from_json(ESSENTIAL_GENOME_JSON)?;
    let (areas_added, morphs_added) = ensure_core_components(&mut genome);
    if areas_added > 0 || morphs_added > 0 {
        tracing::info!("Essential genome: added {} core areas, {} core morphologies", areas_added, morphs_added);
    }
    Ok(genome)
}

/// Load barebones genome from embedded JSON
///
/// Automatically ensures core components (_death, _power, core morphologies) are present
pub fn load_barebones_genome() -> Result<RuntimeGenome, crate::types::EvoError> {
    use crate::genome::loader::load_genome_from_json;
    let mut genome = load_genome_from_json(BAREBONES_GENOME_JSON)?;
    let (areas_added, morphs_added) = ensure_core_components(&mut genome);
    if areas_added > 0 || morphs_added > 0 {
        tracing::info!("Barebones genome: added {} core areas, {} core morphologies", areas_added, morphs_added);
    }
    Ok(genome)
}

/// Load test genome from embedded JSON
///
/// Automatically ensures core components (_death, _power, core morphologies) are present
pub fn load_test_genome() -> Result<RuntimeGenome, crate::types::EvoError> {
    use crate::genome::loader::load_genome_from_json;
    let mut genome = load_genome_from_json(TEST_GENOME_JSON)?;
    let (areas_added, morphs_added) = ensure_core_components(&mut genome);
    if areas_added > 0 || morphs_added > 0 {
        tracing::info!("Test genome: added {} core areas, {} core morphologies", areas_added, morphs_added);
    }
    Ok(genome)
}

/// Load vision genome from embedded JSON
///
/// Automatically ensures core components (_death, _power, core morphologies) are present
pub fn load_vision_genome() -> Result<RuntimeGenome, crate::types::EvoError> {
    use crate::genome::loader::load_genome_from_json;
    let mut genome = load_genome_from_json(VISION_GENOME_JSON)?;
    let (areas_added, morphs_added) = ensure_core_components(&mut genome);
    if areas_added > 0 || morphs_added > 0 {
        tracing::info!("Vision genome: added {} core areas, {} core morphologies", areas_added, morphs_added);
    }
    Ok(genome)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_minimal_genome() {
        let genome = create_minimal_genome("test_genome".to_string(), "Test Genome".to_string());

        assert_eq!(genome.metadata.genome_id, "test_genome");
        assert_eq!(genome.metadata.version, "2.0");
        assert_eq!(genome.cortical_areas.len(), 0);
        assert_eq!(genome.morphologies.count(), 0);
    }

    #[test]
    fn test_create_genome_with_core_morphologies() {
        let genome = create_genome_with_core_morphologies("test_genome".to_string(), "Test Genome".to_string());

        assert_eq!(genome.metadata.genome_id, "test_genome");
        assert!(genome.morphologies.count() > 0);
        assert!(genome.morphologies.contains("block_to_block"));
        assert!(genome.morphologies.contains("projector"));
        assert!(genome.morphologies.contains("centered_projector"));
        assert!(genome.morphologies.contains("transpose_xy"));
        assert!(genome.morphologies.contains("transpose_yz"));
        assert!(genome.morphologies.contains("transpose_xz"));
        assert!(genome.morphologies.contains("first_to_last"));
        assert!(genome.morphologies.contains("lateral_+x"));
    }

    #[test]
    fn test_add_core_morphologies() {
        let mut registry = MorphologyRegistry::new();
        add_core_morphologies(&mut registry);

        // Should have at least 11 core morphologies
        assert!(registry.count() >= 11);
        assert!(registry.contains("block_to_block"));
        assert!(registry.contains("projector"));
        assert!(registry.contains("centered_projector"));
        assert!(registry.contains("transpose_xy"));
        assert!(registry.contains("transpose_yz"));
        assert!(registry.contains("transpose_xz"));
        assert!(registry.contains("first_to_last"));
        assert!(registry.contains("all_to_0-0-0"));
        assert!(registry.contains("lateral_+x"));
        assert!(registry.contains("lateral_-z"));
    }

    #[test]
    fn test_embedded_genomes_exist() {
        // Test that embedded genome strings are not empty
        // These are compile-time constants, so they're always non-empty
        // The assertions verify the constants are defined correctly
        #[allow(clippy::const_is_empty)]
        {
            assert!(!ESSENTIAL_GENOME_JSON.is_empty());
            assert!(!BAREBONES_GENOME_JSON.is_empty());
            assert!(!TEST_GENOME_JSON.is_empty());
            assert!(!VISION_GENOME_JSON.is_empty());
        }
    }

    #[test]
    fn test_load_essential_genome() {
        let genome = load_essential_genome().expect("Failed to load essential genome");
        assert!(!genome.cortical_areas.is_empty());
        // Essential genome should have _power
        let power_id = crate::genome::parser::string_to_cortical_id("_power").expect("Valid ID");
        assert!(genome.cortical_areas.contains_key(&power_id));
    }

    #[test]
    fn test_ensure_core_components_adds_missing_areas() {
        // Create a minimal genome without core areas
        let mut genome = create_minimal_genome("test".to_string(), "Test".to_string());

        assert_eq!(genome.cortical_areas.len(), 0);

        // Ensure core components
        let (areas_added, _) = ensure_core_components(&mut genome);

        // Should have added _death, _power, _fatigue, _pain, _pleasure, _fear, _hope
        assert_eq!(areas_added, 7);

        let death_id = crate::genome::parser::string_to_cortical_id("_death").expect("Valid ID");
        let power_id = crate::genome::parser::string_to_cortical_id("_power").expect("Valid ID");
        let fatigue_id = crate::genome::parser::string_to_cortical_id("_fatigue").expect("Valid ID");
        let pain_id = crate::genome::parser::string_to_cortical_id("_pain").expect("Valid ID");
        let pleasure_id = crate::genome::parser::string_to_cortical_id("_pleasure").expect("Valid ID");
        let fear_id = crate::genome::parser::string_to_cortical_id("_fear").expect("Valid ID");
        let hope_id = crate::genome::parser::string_to_cortical_id("_hope").expect("Valid ID");
        assert!(genome.cortical_areas.contains_key(&death_id));
        assert!(genome.cortical_areas.contains_key(&power_id));
        assert!(genome.cortical_areas.contains_key(&fatigue_id));
        assert!(genome.cortical_areas.contains_key(&pain_id));
        assert!(genome.cortical_areas.contains_key(&pleasure_id));
        assert!(genome.cortical_areas.contains_key(&fear_id));
        assert!(genome.cortical_areas.contains_key(&hope_id));

        // Verify cortical_idx assignments
        assert_eq!(genome.cortical_areas.get(&death_id).unwrap().cortical_idx, 0);
        assert_eq!(genome.cortical_areas.get(&power_id).unwrap().cortical_idx, 1);
        assert_eq!(genome.cortical_areas.get(&fatigue_id).unwrap().cortical_idx, 2);
        assert_eq!(genome.cortical_areas.get(&pain_id).unwrap().cortical_idx, 3);
        assert_eq!(genome.cortical_areas.get(&pleasure_id).unwrap().cortical_idx, 4);
        assert_eq!(genome.cortical_areas.get(&fear_id).unwrap().cortical_idx, 5);
        assert_eq!(genome.cortical_areas.get(&hope_id).unwrap().cortical_idx, 6);
    }

    #[test]
    fn test_ensure_core_components_adds_missing_morphologies() {
        // Create a genome with core areas but no morphologies
        let mut genome = create_genome_with_core_areas("test".to_string(), "Test".to_string());

        assert_eq!(genome.morphologies.count(), 0);

        // Ensure core components
        let (_, morphs_added) = ensure_core_components(&mut genome);

        // Should have added core morphologies
        assert!(morphs_added > 0);
        assert!(genome.morphologies.contains("block_to_block"));
        assert!(genome.morphologies.contains("projector"));
        assert!(genome.morphologies.contains("centered_projector"));
        assert!(genome.morphologies.contains("transpose_xy"));
        assert!(genome.morphologies.contains("transpose_yz"));
        assert!(genome.morphologies.contains("transpose_xz"));
        assert!(genome.morphologies.contains("first_to_last"));
        assert!(genome.morphologies.contains("episodic_memory"));
        assert!(genome.morphologies.contains("lateral_+x"));
    }

    /// Adds a custom cortical area of the given size at the given position, and returns its ID.
    fn add_custom_area(genome: &mut RuntimeGenome, legacy_name: &str, position: (i32, i32, i32), size: usize) -> CorticalID {
        let cortical_id = crate::genome::parser::string_to_cortical_id(legacy_name).expect("custom area name resolves");
        let area = CorticalArea::new(
            cortical_id,
            genome.cortical_areas.len() as u32,
            legacy_name.to_string(),
            NeuronVoxelDimensionsGenomic::new_from_usizes_unchecked(size, size, size),
            GenomeCoordinate3D::new(position.0, position.1, position.2),
            cortical_id.as_cortical_type().expect("custom ID maps to a cortical type"),
        )
        .expect("custom area is well formed");
        genome.cortical_areas.insert(cortical_id, area);
        cortical_id
    }

    /// Records that `source` maps to `destination`, the way the parser stores an area's mappings.
    fn map_area_to(genome: &mut RuntimeGenome, source: &CorticalID, destination: &CorticalID) {
        let area = genome.cortical_areas.get_mut(source).expect("source area is in the genome");
        let mut mappings = area
            .properties
            .get(CORTICAL_MAPPING_DST_KEY)
            .and_then(|existing| existing.as_object().cloned())
            .unwrap_or_default();
        mappings.insert(destination.as_base_64(), Value::Array(Vec::new()));
        area.properties.insert(CORTICAL_MAPPING_DST_KEY.to_string(), Value::Object(mappings));
    }

    /// The one region under the root, which is where the custom and memory areas are gathered.
    fn subregion_of<'a>(genome: &'a RuntimeGenome, root_region_id: &str) -> &'a BrainRegion {
        let children: Vec<&BrainRegion> = genome
            .brain_regions
            .values()
            .filter(|region| region.properties.get(PARENT_REGION_ID_KEY).and_then(|id| id.as_str()) == Some(root_region_id))
            .collect();
        assert_eq!(children.len(), 1, "expected exactly one region under the root");
        children[0]
    }

    #[test]
    fn test_ensure_root_brain_region_gathers_custom_areas_into_a_subregion() {
        // A root plate holding everything gives the visualizer nothing to nest, so the areas a
        // circuit contributes are grouped beneath it instead.
        let mut genome = create_genome_with_core_areas("test".to_string(), "Test Circuit".to_string());
        let core_area_count = genome.cortical_areas.len();
        let left = add_custom_area(&mut genome, "c__lef", (0, 0, 0), 4);
        let right = add_custom_area(&mut genome, "c__rig", (10, 0, 0), 4);

        let (root_region_id, _) = ensure_root_brain_region(&mut genome);

        let root = &genome.brain_regions[&root_region_id];
        assert_eq!(root.cortical_areas.len(), core_area_count, "core areas stay on the root plate");
        assert!(!root.contains_area(&left));
        assert!(!root.contains_area(&right));

        let subregion = subregion_of(&genome, &root_region_id);
        assert_eq!(subregion.name, "Test Circuit", "the subregion is named after the genome");
        assert!(subregion.contains_area(&left));
        assert!(subregion.contains_area(&right));
        assert_eq!(subregion.cortical_areas.len(), 2);
    }

    #[test]
    fn test_ensure_root_brain_region_creates_no_subregion_without_custom_areas() {
        let mut genome = create_genome_with_core_areas("test".to_string(), "Test".to_string());

        let (root_region_id, _) = ensure_root_brain_region(&mut genome);

        assert_eq!(genome.brain_regions.len(), 1, "nothing to gather means no second region");
        assert_eq!(genome.brain_regions[&root_region_id].cortical_areas.len(), genome.cortical_areas.len());
    }

    #[test]
    fn test_ensure_root_brain_region_names_untitled_subregion_generically() {
        let mut genome = create_genome_with_core_areas("test".to_string(), "Untitled".to_string());
        add_custom_area(&mut genome, "c__lef", (0, 0, 0), 4);

        let (root_region_id, _) = ensure_root_brain_region(&mut genome);

        assert_eq!(subregion_of(&genome, &root_region_id).name, AUTOGEN_SUBREGION_FALLBACK_NAME);
    }

    #[test]
    fn test_ensure_root_brain_region_declares_ports_for_cross_boundary_mappings() {
        // The visualizer draws a region's ports from these lists; a mapping that stays inside a
        // region is not a port, and one that leaves it is.
        let mut genome = create_genome_with_core_areas("test".to_string(), "Test".to_string());
        let power_id = crate::genome::parser::string_to_cortical_id("_power").expect("core area name resolves");
        let left = add_custom_area(&mut genome, "c__lef", (0, 0, 0), 4);
        let right = add_custom_area(&mut genome, "c__rig", (10, 0, 0), 4);

        map_area_to(&mut genome, &power_id, &left); // crosses into the subregion
        map_area_to(&mut genome, &left, &right); // stays inside the subregion
        map_area_to(&mut genome, &right, &power_id); // crosses back out to the root

        let (root_region_id, _) = ensure_root_brain_region(&mut genome);

        let root = &genome.brain_regions[&root_region_id];
        assert_eq!(root.properties[REGION_OUTPUTS_KEY], Value::from(vec![power_id.as_base_64()]));
        assert_eq!(root.properties[REGION_INPUTS_KEY], Value::from(vec![power_id.as_base_64()]));

        let subregion = subregion_of(&genome, &root_region_id);
        assert_eq!(subregion.properties[REGION_INPUTS_KEY], Value::from(vec![left.as_base_64()]));
        assert_eq!(
            subregion.properties[REGION_OUTPUTS_KEY],
            Value::from(vec![right.as_base_64()]),
            "the area mapping only within the region is not a port"
        );
    }

    #[test]
    fn test_ensure_root_brain_region_places_subregion_clear_of_the_root_areas() {
        let mut genome = create_minimal_genome("test".to_string(), "Test".to_string());
        let anchor = crate::genome::parser::string_to_cortical_id("_power").expect("core area name resolves");
        let mut power = create_power_area();
        power.position = GenomeCoordinate3D::new(0, 0, 0);
        power.dimensions = NeuronVoxelDimensionsGenomic::new_from_usizes_unchecked(200, 10, 10);
        genome.cortical_areas.insert(anchor, power);
        add_custom_area(&mut genome, "c__lef", (0, 0, 0), 4);

        let (root_region_id, _) = ensure_root_brain_region(&mut genome);

        let position = subregion_of(&genome, &root_region_id).properties[REGION_COORDINATE_3D_KEY]
            .as_array()
            .expect("the subregion carries a 3D position")
            .iter()
            .map(|axis| axis.as_i64().expect("each axis is a number"))
            .collect::<Vec<i64>>();

        // The root's only area spans x 0..200, so the subregion sits beyond its far edge.
        assert!(position[0] > 200, "subregion at x={} overlaps the root areas", position[0]);
    }

    /// Builds a region that names `parent` as its parent, or none when `parent` is `None`.
    fn region_with_parent(name: &str, parent: Option<&str>) -> (String, BrainRegion) {
        let region_id = RegionID::new();
        let mut region = BrainRegion::new(region_id, name.to_string(), RegionType::Undefined).expect("region name is non-empty");
        if let Some(parent) = parent {
            region.add_property(PARENT_REGION_ID_KEY.to_string(), Value::String(parent.to_string()));
        }
        (region_id.to_string(), region)
    }

    #[test]
    fn test_ensure_root_brain_region_creates_root_holding_every_area() {
        // A genome document is allowed to carry no regions at all; the areas still have to be
        // placed somewhere or no reader can resolve them.
        let mut genome = create_genome_with_core_areas("test".to_string(), "Test".to_string());
        assert!(genome.brain_regions.is_empty());

        let (root_region_id, root_was_created) = ensure_root_brain_region(&mut genome);

        assert!(root_was_created);
        assert_eq!(genome.brain_regions.len(), 1);
        assert_eq!(genome.metadata.brain_regions_root.as_deref(), Some(root_region_id.as_str()));

        let root = genome.brain_regions.get(&root_region_id).expect("root region is in the genome");
        assert_eq!(root.name, SYNTHESIZED_ROOT_REGION_NAME);
        assert!(root.properties.get(PARENT_REGION_ID_KEY).is_none());
        assert_eq!(root.cortical_areas.len(), genome.cortical_areas.len());
        for area_id in genome.cortical_areas.keys() {
            assert!(root.contains_area(area_id), "core area {area_id} should be placed in the root region");
        }
    }

    #[test]
    fn test_ensure_root_brain_region_keeps_declared_root() {
        // A genome that declares its hierarchy must be published as written.
        let mut genome = create_genome_with_core_areas("test".to_string(), "Test".to_string());
        let (declared_root_id, declared_root) = region_with_parent("Declared Root", None);
        let (child_id, child) = region_with_parent("Child", Some(&declared_root_id));
        genome.brain_regions.insert(declared_root_id.clone(), declared_root);
        genome.brain_regions.insert(child_id.clone(), child);

        let (root_region_id, root_was_created) = ensure_root_brain_region(&mut genome);

        assert!(!root_was_created);
        assert_eq!(root_region_id, declared_root_id);
        assert_eq!(genome.brain_regions.len(), 2);
        assert_eq!(genome.metadata.brain_regions_root.as_deref(), Some(declared_root_id.as_str()));
        assert_eq!(
            genome.brain_regions[&child_id].properties[PARENT_REGION_ID_KEY],
            Value::String(declared_root_id)
        );
    }

    #[test]
    fn test_ensure_root_brain_region_claims_only_unclaimed_areas() {
        let mut genome = create_genome_with_core_areas("test".to_string(), "Test".to_string());
        let claimed_area = *genome.cortical_areas.keys().next().expect("genome has core areas");
        let (orphan_id, orphan) = region_with_parent("Orphan", Some("a-region-this-genome-does-not-have"));
        genome.brain_regions.insert(orphan_id.clone(), orphan.with_areas([claimed_area]));

        let (root_region_id, root_was_created) = ensure_root_brain_region(&mut genome);

        assert!(root_was_created);
        let root = &genome.brain_regions[&root_region_id];
        assert!(!root.contains_area(&claimed_area), "an area another region claims must not be duplicated");
        assert_eq!(root.cortical_areas.len(), genome.cortical_areas.len() - 1);

        // A region pointing at a parent outside the genome would be unreachable from the root.
        assert_eq!(
            genome.brain_regions[&orphan_id].properties[PARENT_REGION_ID_KEY],
            Value::String(root_region_id)
        );
    }

    #[test]
    fn test_ensure_root_brain_region_idempotent() {
        let mut genome = create_genome_with_core_areas("test".to_string(), "Test".to_string());

        let (first_root_id, first_was_created) = ensure_root_brain_region(&mut genome);
        let (second_root_id, second_was_created) = ensure_root_brain_region(&mut genome);

        assert!(first_was_created);
        assert!(!second_was_created, "a second pass must not add another root");
        assert_eq!(first_root_id, second_root_id);
        assert_eq!(genome.brain_regions.len(), 1);
    }

    #[test]
    fn test_ensure_core_components_idempotent() {
        // Create a genome with all core components
        let mut genome = create_genome_with_core_areas("test".to_string(), "Test".to_string());
        add_core_morphologies(&mut genome.morphologies);

        // Run ensure_core_components
        let (areas_added, morphs_added) = ensure_core_components(&mut genome);

        // Should not add anything (already present)
        assert_eq!(areas_added, 0);
        assert_eq!(morphs_added, 0);
    }
}
