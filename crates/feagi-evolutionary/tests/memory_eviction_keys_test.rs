// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! `memory_neuron_cap` and `ltm_eviction`: parse, validate, extract, and survive a genome save.

use std::collections::HashMap;

use feagi_evolutionary::{
    extract_memory_properties, load_genome_from_json, save_genome_to_json,
    validate_memory_mp_properties, GenomeParser, LtmEvictionMode, LTM_EVICTION_KEY,
    MEMORY_NEURON_CAP_KEY,
};
use feagi_structures::genomic::brain_regions::RegionID;
use feagi_structures::genomic::cortical_area::{
    CorticalArea, CorticalAreaDimensions, CorticalAreaType, CorticalID, MemoryCorticalType,
};
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

#[test]
fn defaults_are_no_cap_and_lru() {
    let props = HashMap::from([("is_mem_type".to_string(), json!(true))]);
    let mem = extract_memory_properties(&props).unwrap();
    assert_eq!(mem.memory_neuron_cap, 0);
    assert_eq!(mem.ltm_eviction, LtmEvictionMode::Lru);
}

#[test]
fn both_keys_survive_a_genome_save_and_load() {
    let capped_id = CorticalID::try_from_bytes(b"mmem0001").unwrap();
    let plain_id = CorticalID::try_from_bytes(b"mmem0002").unwrap();
    let genome = genome_with(vec![
        memory_area(
            capped_id,
            8,
            "capped",
            &[
                (MEMORY_NEURON_CAP_KEY, json!(250)),
                (LTM_EVICTION_KEY, json!("none")),
            ],
        ),
        memory_area(plain_id, 9, "plain", &[]),
    ]);

    let saved = save_genome_to_json(&genome).expect("save");
    let flat: Value = serde_json::from_str(&saved).unwrap();
    let blueprint = flat["blueprint"].as_object().unwrap();
    let capped = capped_id.as_base_64();
    assert_eq!(
        blueprint[&format!("_____10c-{capped}-cx-memcap-i")],
        json!(250)
    );
    assert_eq!(
        blueprint[&format!("_____10c-{capped}-cx-ltmevc-t")],
        json!("none")
    );

    let loaded = load_genome_from_json(&saved).expect("load");
    let capped_props = extract_memory_properties(&loaded.cortical_areas[&capped_id].properties)
        .expect("memory area");
    assert_eq!(capped_props.memory_neuron_cap, 250);
    assert_eq!(capped_props.ltm_eviction, LtmEvictionMode::None);
    let plain_props = extract_memory_properties(&loaded.cortical_areas[&plain_id].properties)
        .expect("memory area");
    assert_eq!(plain_props.memory_neuron_cap, 0);
    assert_eq!(plain_props.ltm_eviction, LtmEvictionMode::Lru);

    // A second save of the loaded genome keeps the values.
    let resaved: Value = serde_json::from_str(&save_genome_to_json(&loaded).unwrap()).unwrap();
    assert_eq!(
        resaved["blueprint"][&format!("_____10c-{capped}-cx-memcap-i")],
        json!(250)
    );
}

#[test]
fn hierarchical_genome_parser_reads_both_keys() {
    let id = CorticalID::try_from_bytes(b"mmem0001")
        .unwrap()
        .as_base_64();
    let text = json!({
        "version": "2.1",
        "blueprint": {
            id.clone(): {
                "cortical_name": "m",
                "block_boundaries": [2, 2, 1],
                "relative_coordinate": [0, 0, 0],
                "cortical_type": "MEMORY",
                "is_mem_type": true,
                "memory_neuron_cap": 40,
                "ltm_eviction": "none"
            }
        },
        "brain_regions": {}
    })
    .to_string();
    let parsed = GenomeParser::parse(&text).unwrap();
    let props = &parsed.cortical_areas[0].properties;
    assert_eq!(props["memory_neuron_cap"], json!(40));
    assert_eq!(props["ltm_eviction"], json!("none"));
    let mem = extract_memory_properties(props).unwrap();
    assert_eq!(
        (mem.memory_neuron_cap, mem.ltm_eviction),
        (40, LtmEvictionMode::None)
    );
}

#[test]
fn validation_rejects_bad_values() {
    let check = |entries: &[(&str, Value)]| {
        let props: HashMap<String, Value> = entries
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();
        validate_memory_mp_properties(&props)
    };
    assert!(check(&[(LTM_EVICTION_KEY, json!("lru"))]).is_ok());
    assert!(check(&[(LTM_EVICTION_KEY, json!("none"))]).is_ok());
    assert!(check(&[(LTM_EVICTION_KEY, json!("oldest"))]).is_err());
    assert!(check(&[(LTM_EVICTION_KEY, json!(1))]).is_err());
    assert!(check(&[(MEMORY_NEURON_CAP_KEY, json!(0))]).is_ok());
    assert!(check(&[(MEMORY_NEURON_CAP_KEY, json!(5000))]).is_ok());
    assert!(check(&[(MEMORY_NEURON_CAP_KEY, json!(-1))]).is_err());
    assert!(check(&[(MEMORY_NEURON_CAP_KEY, json!(2.5))]).is_err());
    assert!(check(&[(MEMORY_NEURON_CAP_KEY, json!("many"))]).is_err());
}
