// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Registration of `capture_max_bursts`, the `sequential_memory` mapping and the `memory.capture`
//! loop warning: store, validate, and survive a genome save and load. Nothing records or replays.

use std::collections::HashMap;

use feagi_evolutionary::validate_genome;
use feagi_evolutionary::{
    extract_memory_properties, load_genome_from_json, save_genome_to_json,
    validate_memory_mp_properties, ModulatorInstance, CAPTURE_MAX_BURSTS_KEY,
};
use feagi_structures::genomic::brain_regions::RegionID;
use feagi_structures::genomic::cortical_area::{
    CorticalArea, CorticalAreaDimensions, CorticalAreaType, CorticalID, MemoryCorticalType,
};
use feagi_structures::genomic::ModulatorKind;
use feagi_structures::genomic::{BrainRegion, RegionType};
use serde_json::{json, Value};

fn memory_area(id: CorticalID, idx: u32, name: &str, extra: &[(&str, Value)]) -> CorticalArea {
    let mut area = CorticalArea::new(
        id,
        idx,
        name.to_string(),
        CorticalAreaDimensions::new(2, 2, 1).unwrap(),
        (0, 0, 0).into(),
        CorticalAreaType::Memory(MemoryCorticalType::Memory),
    )
    .unwrap();
    area.properties
        .insert("is_mem_type".to_string(), json!(true));
    for (key, value) in extra {
        area.properties.insert(key.to_string(), value.clone());
    }
    area
}

fn genome_with(areas: Vec<CorticalArea>) -> feagi_evolutionary::RuntimeGenome {
    let ids: Vec<CorticalID> = areas.iter().map(|a| a.cortical_id).collect();
    let region = BrainRegion::new(RegionID::new(), "r".to_string(), RegionType::Undefined)
        .unwrap()
        .with_areas(ids);
    let region_key = region.region_id.to_string();
    let mut brain_regions = HashMap::new();
    brain_regions.insert(region_key.clone(), region);
    feagi_evolutionary::RuntimeGenome {
        metadata: feagi_evolutionary::GenomeMetadata {
            genome_id: "eviction_keys".to_string(),
            genome_title: "Eviction keys".to_string(),
            genome_description: String::new(),
            version: "2.0".to_string(),
            timestamp: 1.0,
            brain_regions_root: Some(region_key),
        },
        cortical_areas: areas.into_iter().map(|a| (a.cortical_id, a)).collect(),
        brain_regions,
        classifiers: HashMap::new(),
        morphologies: feagi_evolutionary::MorphologyRegistry::new(),
        modulators: feagi_evolutionary::ModulatorRegistry::new(),
        physiology: feagi_evolutionary::PhysiologyConfig::default(),
        signatures: feagi_evolutionary::GenomeSignatures {
            genome: "0".to_string(),
            blueprint: "0".to_string(),
            physiology: "0".to_string(),
            morphologies: None,
        },
        stats: feagi_evolutionary::GenomeStats::default(),
        change_history: Vec::new(),
    }
}

const D10_WARNING: &str = "memory.capture on this area records episodes but cannot replay them: add a sequential_memory mapping from the area to itself.";

fn capture_modulator() -> ModulatorInstance {
    ModulatorInstance {
        kind: ModulatorKind::MemoryCapture,
        magnitude_percent: 50.0,
        effect_duration_bursts: 1,
        rest_bursts: 0,
        graded: false,
        full_scale_potential: None,
        driver_cortical_id: CorticalID::modulator_driver(1),
    }
}

fn loop_rule() -> Value {
    json!({
        "morphology_id": "sequential_memory",
        "morphology_scalar": [1],
        "postSynapticCurrent_multiplier": 1.0,
        "plasticity_flag": false,
        "synaptic_delay_bursts": 2
    })
}

#[test]
fn capture_max_bursts_is_optional_validated_and_survives_a_save_and_load() {
    let props = HashMap::from([("is_mem_type".to_string(), json!(true))]);
    assert_eq!(
        extract_memory_properties(&props)
            .unwrap()
            .capture_max_bursts,
        None
    );
    let check = |v: Value| {
        validate_memory_mp_properties(&HashMap::from([(CAPTURE_MAX_BURSTS_KEY.to_string(), v)]))
    };
    assert!(check(json!(1)).is_ok());
    assert!(check(json!(300)).is_ok());
    assert!(check(Value::Null).is_ok());
    assert!(check(json!(0)).is_err());
    assert!(check(json!(-3)).is_err());
    assert!(check(json!(1.5)).is_err());
    assert!(check(json!("5")).is_err());

    let capped_id = CorticalID::try_from_bytes(b"mmem0001").unwrap();
    let plain_id = CorticalID::try_from_bytes(b"mmem0002").unwrap();
    let genome = genome_with(vec![
        memory_area(
            capped_id,
            8,
            "capped",
            &[(CAPTURE_MAX_BURSTS_KEY, json!(120))],
        ),
        memory_area(plain_id, 9, "plain", &[]),
    ]);
    let saved = save_genome_to_json(&genome).expect("save");
    let flat: Value = serde_json::from_str(&saved).unwrap();
    let capped = capped_id.as_base_64();
    assert_eq!(
        flat["blueprint"][&format!("_____10c-{capped}-cx-capmxb-i")],
        json!(120)
    );
    let loaded = load_genome_from_json(&saved).expect("load");
    let capped_props = extract_memory_properties(&loaded.cortical_areas[&capped_id].properties)
        .expect("memory area");
    assert_eq!(capped_props.capture_max_bursts, Some(120));
    let plain_props = extract_memory_properties(&loaded.cortical_areas[&plain_id].properties)
        .expect("memory area");
    assert_eq!(plain_props.capture_max_bursts, None);
    let resaved: Value = serde_json::from_str(&save_genome_to_json(&loaded).unwrap()).unwrap();
    assert_eq!(
        resaved["blueprint"][&format!("_____10c-{capped}-cx-capmxb-i")],
        json!(120)
    );
}

#[test]
fn sequential_memory_mapping_survives_hierarchical_flat_hierarchical() {
    let id = CorticalID::try_from_bytes(b"mmem0001").unwrap();
    let b64 = id.as_base_64();
    let mut area = memory_area(id, 8, "looped", &[(CAPTURE_MAX_BURSTS_KEY, json!(77))]);
    area.properties.insert(
        "cortical_mapping_dst".to_string(),
        json!({ b64.clone(): [loop_rule()] }),
    );
    let mut genome = genome_with(vec![area]);
    genome.morphologies = feagi_evolutionary::MorphologyRegistry::new();

    let saved = save_genome_to_json(&genome).expect("save");
    let loaded = load_genome_from_json(&saved).expect("load");
    let area = &loaded.cortical_areas[&id];
    let rules = area.properties["cortical_mapping_dst"][&b64]
        .as_array()
        .expect("loop mapping kept");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0]["morphology_id"], json!("sequential_memory"));
    assert_eq!(rules[0]["postSynapticCurrent_multiplier"], json!(1.0));
    assert_eq!(rules[0]["synaptic_delay_bursts"], json!(2));
    assert_eq!(
        extract_memory_properties(&area.properties)
            .unwrap()
            .capture_max_bursts,
        Some(77)
    );

    // Unchanged on a second round trip.
    let again = load_genome_from_json(&save_genome_to_json(&loaded).unwrap()).unwrap();
    assert_eq!(
        again.cortical_areas[&id].properties["cortical_mapping_dst"],
        loaded.cortical_areas[&id].properties["cortical_mapping_dst"]
    );
}

#[test]
fn capture_without_a_loop_warns_and_with_a_loop_does_not() {
    let id = CorticalID::try_from_bytes(b"mmem0001").unwrap();
    let b64 = id.as_base_64();

    let mut bare = memory_area(id, 8, "bare", &[("modulators", json!(["rec"]))]);
    bare.properties
        .insert("is_mem_type".to_string(), json!(true));
    let mut genome = genome_with(vec![bare]);
    genome
        .modulators
        .insert("rec".to_string(), capture_modulator());
    let result = validate_genome(&genome);
    assert!(
        result.warnings.iter().any(|w| w == D10_WARNING),
        "{:?}",
        result.warnings
    );

    let mut looped = memory_area(id, 8, "looped", &[("modulators", json!(["rec"]))]);
    looped.properties.insert(
        "cortical_mapping_dst".to_string(),
        json!({ b64: [loop_rule()] }),
    );
    let mut genome = genome_with(vec![looped]);
    genome
        .modulators
        .insert("rec".to_string(), capture_modulator());
    let result = validate_genome(&genome);
    assert!(
        !result.warnings.iter().any(|w| w.contains("memory.capture")),
        "{:?}",
        result.warnings
    );

    // No capture subscription: no warning either.
    let plain = genome_with(vec![memory_area(id, 8, "plain", &[])]);
    assert!(!validate_genome(&plain)
        .warnings
        .iter()
        .any(|w| w.contains("memory.capture")));
}
