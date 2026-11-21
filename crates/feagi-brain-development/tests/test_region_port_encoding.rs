// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for the brain-region `port_encoding` property: derivation from the IO areas
//! wired to a designated port, author entries, and pruning.

use std::collections::HashMap;

use feagi_brain_development::{BduError, ConnectomeManager};
use feagi_structures::genomic::brain_regions::{
    region_port_encoding, PortEncoding, PortEncodingSource, RegionID, RegionType, PORT_ENCODING_KEY,
};
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalSubUnitIndex, CorticalUnitIndex,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, IOCorticalAreaConfigurationFlag as Flag, PercentageNeuronPositioning,
};
use feagi_structures::genomic::cortical_area::{
    CorticalArea, CorticalAreaDimensions, CorticalAreaType, CorticalID, CustomCorticalType,
};
use feagi_structures::genomic::BrainRegion;
use serde_json::{json, Value};

const ABS_LINEAR: Flag = Flag::Percentage(
    FrameChangeHandling::Absolute,
    PercentageNeuronPositioning::Linear,
);
const INCR_LINEAR: Flag = Flag::Percentage(
    FrameChangeHandling::Incremental,
    PercentageNeuronPositioning::Linear,
);
const SIGNED_ABS_FRACTIONAL: Flag = Flag::SignedPercentage(
    FrameChangeHandling::Absolute,
    PercentageNeuronPositioning::Fractional,
);

fn io_id(flag: Flag, is_input: bool, tag: &[u8; 3]) -> CorticalID {
    flag.as_io_cortical_id(
        is_input,
        *tag,
        CorticalUnitIndex::from(0u16),
        CorticalSubUnitIndex::from(0u8),
    )
}

fn area(id: CorticalID, name: &str, area_type: CorticalAreaType) -> CorticalArea {
    CorticalArea::new(
        id,
        0,
        name.to_string(),
        CorticalAreaDimensions::new(1, 1, 1).unwrap(),
        (0, 0, 0).into(),
        area_type,
    )
    .unwrap()
}

fn custom(bytes: &[u8; 8], name: &str) -> (CorticalArea, CorticalID) {
    let id = CorticalID::try_from_bytes(bytes).unwrap();
    (
        area(
            id,
            name,
            CorticalAreaType::Custom(CustomCorticalType::LeakyIntegrateFire),
        ),
        id,
    )
}

fn projector_rule() -> Value {
    json!({
        "morphology_id": "projector",
        "postSynapticCurrent_multiplier": 1.0,
        "synapse_attractivity": 100
    })
}

/// A root region and one circuit region with a designated-input candidate and an output candidate.
struct Fixture {
    manager: ConnectomeManager,
    circuit: String,
    port_in: CorticalID,
    port_out: CorticalID,
}

fn fixture() -> Fixture {
    let mut manager = ConnectomeManager::new_for_testing();
    let (a_in, port_in) = custom(b"cprt_in1", "port_in");
    let (a_out, port_out) = custom(b"cprt_ou1", "port_out");
    manager.add_cortical_area(a_in).unwrap();
    manager.add_cortical_area(a_out).unwrap();
    let root_id = RegionID::new();
    let circuit_id = RegionID::new();
    let root_key = root_id.to_string();
    let circuit = circuit_id.to_string();
    manager
        .add_brain_region(
            BrainRegion::new(root_id, "Root".to_string(), RegionType::Undefined).unwrap(),
            None,
        )
        .unwrap();
    manager
        .add_brain_region(
            BrainRegion::new(circuit_id, "Circuit".to_string(), RegionType::Undefined)
                .unwrap()
                .with_areas([port_in, port_out]),
            Some(root_key),
        )
        .unwrap();
    Fixture {
        manager,
        circuit,
        port_in,
        port_out,
    }
}

impl Fixture {
    /// Add an IPU or OPU area to the root region.
    fn add_io(&mut self, flag: Flag, is_input: bool, tag: &[u8; 3], name: &str) -> CorticalID {
        let id = io_id(flag, is_input, tag);
        let area_type = if is_input {
            CorticalAreaType::BrainInput(flag)
        } else {
            CorticalAreaType::BrainOutput(flag)
        };
        self.manager
            .add_cortical_area(area(id, name, area_type))
            .unwrap();
        let root = self.manager.get_root_region_id().unwrap();
        self.manager
            .get_brain_region_mut(&root)
            .unwrap()
            .add_area(id);
        id
    }

    fn map(&mut self, src: &CorticalID, dst: &CorticalID, rules: Vec<Value>) {
        self.manager
            .update_cortical_mapping(src, dst, rules)
            .unwrap();
    }

    fn patch(&mut self, patch: Value) -> Result<(), BduError> {
        let properties: HashMap<String, Value> = patch
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        self.manager
            .update_brain_region_properties(&self.circuit, properties)
            .map(|_| ())
    }

    fn designate(&mut self, inputs: &[CorticalID], outputs: &[CorticalID]) {
        let to_b64 = |ids: &[CorticalID]| ids.iter().map(|i| i.as_base_64()).collect::<Vec<_>>();
        self.patch(json!({
            "designated_inputs": to_b64(inputs),
            "designated_outputs": to_b64(outputs),
        }))
        .unwrap();
    }

    fn recompute(&mut self) {
        self.manager.recompute_brain_region_io_registry().unwrap();
    }

    fn entry(&self, port: &CorticalID) -> Option<PortEncoding> {
        let region = self.manager.get_brain_region(&self.circuit).unwrap();
        region_port_encoding(region)
            .unwrap()
            .remove(&port.as_base_64())
    }

    fn entry_count(&self) -> usize {
        let region = self.manager.get_brain_region(&self.circuit).unwrap();
        region_port_encoding(region).unwrap().len()
    }
}

fn assert_values(entry: &PortEncoding, kind: &str, signed: bool, frame: &str, positioning: &str) {
    assert_eq!(entry.kind.as_deref(), Some(kind));
    assert_eq!(entry.signed, Some(signed));
    assert_eq!(entry.frame.as_deref(), Some(frame));
    assert_eq!(entry.positioning.as_deref(), Some(positioning));
}

#[test]
fn designated_input_wired_to_an_ipu_stores_four_values_with_source_wiring() {
    let mut f = fixture();
    let ipu = f.add_io(ABS_LINEAR, true, b"sea", "sensor_a");
    let port_in = f.port_in;
    f.map(&ipu, &port_in, vec![projector_rule()]);
    f.designate(&[port_in], &[]);

    let entry = f.entry(&port_in).expect("entry for the wired port");
    assert_values(&entry, "Percentage", false, "Absolute", "Linear");
    assert_eq!(entry.source, PortEncodingSource::Wiring);
    assert_eq!(f.entry_count(), 1);
}

#[test]
fn designated_output_wired_to_an_opu_stores_values() {
    let mut f = fixture();
    let opu = f.add_io(SIGNED_ABS_FRACTIONAL, false, b"mot", "motor_a");
    let port_out = f.port_out;
    f.map(&port_out, &opu, vec![projector_rule()]);
    f.designate(&[], &[port_out]);

    let entry = f.entry(&port_out).expect("entry for the wired port");
    assert_values(&entry, "SignedPercentage", true, "Absolute", "Fractional");
    assert_eq!(entry.source, PortEncodingSource::Wiring);
}

#[test]
fn rewiring_to_a_different_ipu_updates_the_entry() {
    let mut f = fixture();
    let first = f.add_io(ABS_LINEAR, true, b"sea", "sensor_a");
    let second = f.add_io(SIGNED_ABS_FRACTIONAL, true, b"seb", "sensor_b");
    let port_in = f.port_in;
    f.map(&first, &port_in, vec![projector_rule()]);
    f.designate(&[port_in], &[]);
    assert_values(
        &f.entry(&port_in).unwrap(),
        "Percentage",
        false,
        "Absolute",
        "Linear",
    );

    f.map(&first, &port_in, vec![]);
    f.map(&second, &port_in, vec![projector_rule()]);
    f.recompute();
    let entry = f.entry(&port_in).unwrap();
    assert_values(&entry, "SignedPercentage", true, "Absolute", "Fractional");
    assert_eq!(entry.source, PortEncodingSource::Wiring);
}

#[test]
fn two_disagreeing_ipus_give_null_only_on_the_differing_field() {
    let mut f = fixture();
    let absolute = f.add_io(ABS_LINEAR, true, b"sea", "sensor_a");
    let incremental = f.add_io(INCR_LINEAR, true, b"seb", "sensor_b");
    let port_in = f.port_in;
    f.map(&absolute, &port_in, vec![projector_rule()]);
    f.map(&incremental, &port_in, vec![projector_rule()]);
    f.designate(&[port_in], &[]);

    let entry = f.entry(&port_in).unwrap();
    assert_eq!(entry.kind.as_deref(), Some("Percentage"));
    assert_eq!(entry.signed, Some(false));
    assert_eq!(entry.frame, None);
    assert_eq!(entry.positioning.as_deref(), Some("Linear"));
    assert_eq!(entry.source, PortEncodingSource::Wiring);
}

#[test]
fn mapping_with_no_rules_is_not_a_peer() {
    let mut f = fixture();
    let ipu = f.add_io(ABS_LINEAR, true, b"sea", "sensor_a");
    let port_in = f.port_in;
    f.map(&ipu, &port_in, vec![]);
    f.designate(&[port_in], &[]);
    assert!(f.entry(&port_in).is_none());
}

#[test]
fn author_entry_survives_rewiring_and_clearing_resumes_derivation() {
    let mut f = fixture();
    let first = f.add_io(ABS_LINEAR, true, b"sea", "sensor_a");
    let second = f.add_io(SIGNED_ABS_FRACTIONAL, true, b"seb", "sensor_b");
    let port_in = f.port_in;
    f.map(&first, &port_in, vec![projector_rule()]);
    f.designate(&[port_in], &[]);

    let key = port_in.as_base_64();
    f.patch(json!({ PORT_ENCODING_KEY: { key.clone(): {"kind": "Boolean", "signed": false} } }))
        .unwrap();
    let authored = f.entry(&port_in).unwrap();
    assert_eq!(authored.source, PortEncodingSource::Author);
    assert_eq!(authored.kind.as_deref(), Some("Boolean"));
    assert_eq!(authored.frame, None);

    // Rewire to another IPU: the author entry is untouched.
    f.map(&first, &port_in, vec![]);
    f.map(&second, &port_in, vec![projector_rule()]);
    f.recompute();
    assert_eq!(f.entry(&port_in).unwrap(), authored);

    // Clearing removes only the author entry; derivation resumes from the current wiring.
    f.patch(json!({ PORT_ENCODING_KEY: { key: null } }))
        .unwrap();
    let derived = f.entry(&port_in).unwrap();
    assert_eq!(derived.source, PortEncodingSource::Wiring);
    assert_values(&derived, "SignedPercentage", true, "Absolute", "Fractional");
}

#[test]
fn clearing_a_wiring_entry_leaves_it_and_clearing_a_missing_entry_is_a_no_op() {
    let mut f = fixture();
    let ipu = f.add_io(ABS_LINEAR, true, b"sea", "sensor_a");
    let (port_in, port_out) = (f.port_in, f.port_out);
    f.map(&ipu, &port_in, vec![projector_rule()]);
    f.designate(&[port_in], &[port_out]);
    let before = f.entry(&port_in).unwrap();
    f.patch(json!({ PORT_ENCODING_KEY: {
        port_in.as_base_64(): null,
        port_out.as_base_64(): null,
    }}))
    .unwrap();
    assert_eq!(f.entry(&port_in).unwrap(), before);
    assert!(f.entry(&port_out).is_none());
}

#[test]
fn zero_peers_keeps_the_stored_entry_unchanged() {
    let mut f = fixture();
    let ipu = f.add_io(ABS_LINEAR, true, b"sea", "sensor_a");
    let port_in = f.port_in;
    f.map(&ipu, &port_in, vec![projector_rule()]);
    f.designate(&[port_in], &[]);
    let stored = f.entry(&port_in).unwrap();

    // The wiring goes away (as in a region export): the stored entry stays.
    f.map(&ipu, &port_in, vec![]);
    f.recompute();
    assert_eq!(f.entry(&port_in).unwrap(), stored);
}

#[test]
fn port_that_is_itself_an_io_area_derives_from_its_own_flags() {
    let mut manager = ConnectomeManager::new_for_testing();
    let ipu = io_id(INCR_LINEAR, true, b"own");
    manager
        .add_cortical_area(area(ipu, "own", CorticalAreaType::BrainInput(INCR_LINEAR)))
        .unwrap();
    let root_id = RegionID::new();
    let circuit_id = RegionID::new();
    let circuit = circuit_id.to_string();
    manager
        .add_brain_region(
            BrainRegion::new(root_id, "Root".to_string(), RegionType::Undefined).unwrap(),
            None,
        )
        .unwrap();
    manager
        .add_brain_region(
            BrainRegion::new(circuit_id, "Circuit".to_string(), RegionType::Undefined)
                .unwrap()
                .with_areas([ipu]),
            Some(root_id.to_string()),
        )
        .unwrap();
    let mut patch = HashMap::new();
    patch.insert("designated_inputs".to_string(), json!([ipu.as_base_64()]));
    manager
        .update_brain_region_properties(&circuit, patch)
        .unwrap();
    let region = manager.get_brain_region(&circuit).unwrap();
    let entry = region_port_encoding(region)
        .unwrap()
        .remove(&ipu.as_base_64())
        .unwrap();
    assert_values(&entry, "Percentage", false, "Incremental", "Linear");
    assert_eq!(entry.source, PortEncodingSource::Wiring);
}

#[test]
fn undesignating_a_port_drops_its_entry() {
    let mut f = fixture();
    let ipu = f.add_io(ABS_LINEAR, true, b"sea", "sensor_a");
    let (port_in, port_out) = (f.port_in, f.port_out);
    f.map(&ipu, &port_in, vec![projector_rule()]);
    f.designate(&[port_in], &[port_out]);
    // An author entry on the output is dropped with its designation too.
    f.patch(json!({ PORT_ENCODING_KEY: { port_out.as_base_64(): {"kind": "Misc"} } }))
        .unwrap();
    assert_eq!(f.entry_count(), 2);

    f.designate(&[], &[port_out]);
    assert!(f.entry(&port_in).is_none());
    assert!(f.entry(&port_out).is_some());

    f.designate(&[], &[]);
    assert_eq!(f.entry_count(), 0);
    let region = f.manager.get_brain_region(&f.circuit).unwrap();
    assert!(!region.properties.contains_key(PORT_ENCODING_KEY));
}

#[test]
fn removing_the_area_from_the_region_drops_its_entry() {
    let mut f = fixture();
    let ipu = f.add_io(ABS_LINEAR, true, b"sea", "sensor_a");
    let port_in = f.port_in;
    f.map(&ipu, &port_in, vec![projector_rule()]);
    f.designate(&[port_in], &[]);
    assert_eq!(f.entry_count(), 1);

    let circuit = f.circuit.clone();
    f.manager
        .get_brain_region_mut(&circuit)
        .unwrap()
        .remove_area(&port_in);
    f.recompute();
    assert_eq!(f.entry_count(), 0);
}

#[test]
fn author_set_is_validated() {
    let mut f = fixture();
    let (port_in, port_out) = (f.port_in, f.port_out);
    f.designate(&[port_in], &[]);

    // The port must be designated.
    let err = f
        .patch(json!({ PORT_ENCODING_KEY: { port_out.as_base_64(): {"kind": "Boolean"} } }))
        .unwrap_err();
    assert!(
        matches!(err, BduError::RegionIoPolicyViolation(_)),
        "{err:?}"
    );

    // Vocabulary, types, unknown fields, source, key and shape errors.
    let key = port_in.as_base_64();
    for bad in [
        json!({ PORT_ENCODING_KEY: { key.clone(): {"kind": "Wobbly"} } }),
        json!({ PORT_ENCODING_KEY: { key.clone(): {"frame": "Sideways"} } }),
        json!({ PORT_ENCODING_KEY: { key.clone(): {"positioning": "Diagonal"} } }),
        json!({ PORT_ENCODING_KEY: { key.clone(): {"signed": "yes"} } }),
        json!({ PORT_ENCODING_KEY: { key.clone(): {"source": "wiring"} } }),
        json!({ PORT_ENCODING_KEY: { key.clone(): {"colour": "red"} } }),
        json!({ PORT_ENCODING_KEY: { key.clone(): "Boolean" } }),
        json!({ PORT_ENCODING_KEY: { "not base64!": {"kind": "Boolean"} } }),
        json!({ PORT_ENCODING_KEY: [] }),
    ] {
        let err = f.patch(bad.clone()).unwrap_err();
        assert!(matches!(err, BduError::InvalidArea(_)), "{bad}: {err:?}");
    }
    // A rejected PATCH changes nothing.
    assert_eq!(f.entry_count(), 0);

    // A designation and its entry may arrive in one PATCH.
    f.patch(json!({
        "designated_outputs": [port_out.as_base_64()],
        PORT_ENCODING_KEY: { port_out.as_base_64(): {"kind": "Misc", "frame": "Incremental"} },
    }))
    .unwrap();
    let entry = f.entry(&port_out).unwrap();
    assert_eq!(entry.kind.as_deref(), Some("Misc"));
    assert_eq!(entry.source, PortEncodingSource::Author);
}

#[test]
fn port_encoding_patch_is_a_delta_that_keeps_other_entries() {
    let mut f = fixture();
    let (port_in, port_out) = (f.port_in, f.port_out);
    f.designate(&[port_in], &[port_out]);
    f.patch(json!({ PORT_ENCODING_KEY: { port_in.as_base_64(): {"kind": "Boolean"} } }))
        .unwrap();
    f.patch(json!({ PORT_ENCODING_KEY: { port_out.as_base_64(): {"kind": "Misc"} } }))
        .unwrap();
    assert_eq!(f.entry_count(), 2);
}

/// Genome load runs the same derivation: a designated port wired to an IPU in the genome gets its
/// `wiring` entry without any later mapping change.
#[test]
fn genome_load_derives_port_encoding() {
    use feagi_brain_development::Neuroembryogenesis;
    use feagi_npu_burst_engine::{backend::CPUBackend, DynamicNPU, RustNPU, TracingMutex};
    use parking_lot::RwLock;
    use std::sync::Arc;

    let npu = RustNPU::new(
        feagi_npu_runtime::StdRuntime,
        CPUBackend::new(),
        100_000,
        1_000_000,
        10,
    )
    .expect("npu");
    let manager = Arc::new(RwLock::new(ConnectomeManager::new_for_testing_with_npu(
        Arc::new(TracingMutex::new(
            DynamicNPU::F32(npu),
            "PortEncodingLoadNPU",
        )),
    )));
    let mut neuro = Neuroembryogenesis::new(manager.clone());

    let mut genome = feagi_evolutionary::create_genome_with_core_morphologies(
        "port_encoding_load".to_string(),
        "Port Encoding Load".to_string(),
    );
    let ipu = io_id(ABS_LINEAR, true, b"ldi");
    let (port_area, port) = custom(b"cldport1", "port");
    let mut ipu_area = area(ipu, "sensor", CorticalAreaType::BrainInput(ABS_LINEAR));
    ipu_area.properties.insert(
        "cortical_mapping_dst".to_string(),
        json!({ port.as_base_64(): [projector_rule()] }),
    );
    genome.cortical_areas.insert(ipu, ipu_area);
    genome.cortical_areas.insert(port, port_area);

    let root_id = RegionID::new();
    let circuit_id = RegionID::new();
    let root_key = root_id.to_string();
    let circuit_key = circuit_id.to_string();
    let root = BrainRegion::new(
        root_id,
        "Root Brain Region".to_string(),
        RegionType::Undefined,
    )
    .unwrap()
    .with_areas([ipu]);
    let mut circuit = BrainRegion::new(circuit_id, "Circuit".to_string(), RegionType::Undefined)
        .unwrap()
        .with_areas([port]);
    circuit
        .properties
        .insert("parent_region_id".to_string(), json!(root_key));
    circuit
        .properties
        .insert("designated_inputs".to_string(), json!([port.as_base_64()]));
    genome.brain_regions.insert(root_key, root);
    genome.brain_regions.insert(circuit_key.clone(), circuit);

    neuro.develop_from_genome(&genome).expect("genome loads");

    let loaded = manager.read();
    let region = loaded.get_brain_region(&circuit_key).expect("circuit");
    let entry = region_port_encoding(region)
        .unwrap()
        .remove(&port.as_base_64())
        .expect("derived at load");
    assert_values(&entry, "Percentage", false, "Absolute", "Linear");
    assert_eq!(entry.source, PortEncodingSource::Wiring);
}
