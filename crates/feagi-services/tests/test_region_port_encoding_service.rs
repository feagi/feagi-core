// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Region `port_encoding` through the service layer: the live connectome and the RuntimeGenome
//! (the source of truth for save) stay in step across designation, mapping, author set and clear,
//! area deletion, and a save/reload round trip.

use std::collections::HashMap;
use std::sync::Arc;

use feagi_brain_development::ConnectomeManager;
use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::{DynamicNPU, RustNPU, TracingMutex};
use feagi_npu_runtime::StdRuntime;
use feagi_services::impls::ConnectomeServiceImpl;
use feagi_services::traits::ConnectomeService;
use feagi_services::types::ServiceError;
use feagi_structures::genomic::brain_regions::{
    region_port_encoding, BrainRegion, PortEncodingSource, RegionID, RegionType, PORT_ENCODING_KEY,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use feagi_structures::genomic::cortical_area::{
    CorticalArea, CorticalAreaDimensions, CorticalAreaType, CorticalID, CustomCorticalType,
    IOCorticalAreaConfigurationFlag,
};
use parking_lot::RwLock;
use serde_json::{json, Value};

/// The StateManager is process-global; tests in this file take turns.
static TEST_LOCK: std::sync::LazyLock<tokio::sync::Mutex<()>> =
    std::sync::LazyLock::new(|| tokio::sync::Mutex::new(()));

const SENSOR_FLAG: IOCorticalAreaConfigurationFlag = IOCorticalAreaConfigurationFlag::Percentage(
    FrameChangeHandling::Absolute,
    PercentageNeuronPositioning::Linear,
);

struct Rig {
    svc: ConnectomeServiceImpl,
    connectome: Arc<RwLock<ConnectomeManager>>,
    genome: Arc<RwLock<Option<feagi_evolutionary::RuntimeGenome>>>,
    circuit: String,
    ipu: CorticalID,
    port: CorticalID,
}

fn area(id: CorticalID, name: &str, kind: CorticalAreaType) -> CorticalArea {
    CorticalArea::new(
        id,
        0,
        name.to_string(),
        CorticalAreaDimensions::new(1, 1, 1).unwrap(),
        (0, 0, 0).into(),
        kind,
    )
    .unwrap()
}

fn rig() -> Rig {
    let npu = Arc::new(TracingMutex::new(
        DynamicNPU::F32(RustNPU::new(StdRuntime, CPUBackend::new(), 10_000, 10_000, 10).unwrap()),
        "PortEncodingNPU",
    ));
    let mut manager = ConnectomeManager::new_for_testing_with_npu(npu);
    manager.setup_core_morphologies_for_testing();

    let ipu = SENSOR_FLAG.as_io_cortical_id(true, *b"psv", 0u16.into(), 0u8.into());
    let port = CorticalID::try_from_bytes(b"cprtsvc1").unwrap();
    manager
        .add_cortical_area(area(
            ipu,
            "sensor",
            CorticalAreaType::BrainInput(SENSOR_FLAG),
        ))
        .unwrap();
    manager
        .add_cortical_area(area(
            port,
            "port",
            CorticalAreaType::Custom(CustomCorticalType::LeakyIntegrateFire),
        ))
        .unwrap();

    let root_id = RegionID::new();
    let circuit_id = RegionID::new();
    let circuit = circuit_id.to_string();
    manager
        .add_brain_region(
            BrainRegion::new(root_id, "Root".to_string(), RegionType::Undefined)
                .unwrap()
                .with_areas([ipu]),
            None,
        )
        .unwrap();
    manager
        .add_brain_region(
            BrainRegion::new(circuit_id, "Circuit".to_string(), RegionType::Undefined)
                .unwrap()
                .with_areas([port]),
            Some(root_id.to_string()),
        )
        .unwrap();

    // The genome store mirrors the live state, as after a genome load.
    let mut genome = feagi_evolutionary::RuntimeGenome {
        metadata: feagi_evolutionary::GenomeMetadata {
            genome_id: "g".to_string(),
            genome_title: "g".to_string(),
            genome_description: String::new(),
            version: "3.0".to_string(),
            timestamp: 0.0,
            brain_regions_root: None,
        },
        cortical_areas: HashMap::new(),
        brain_regions: HashMap::new(),
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
    };
    for id in [ipu, port] {
        genome
            .cortical_areas
            .insert(id, manager.get_cortical_area(&id).unwrap().clone());
    }
    for region_id in manager.get_brain_region_ids() {
        genome.brain_regions.insert(
            region_id.to_string(),
            manager.get_brain_region(region_id).unwrap().clone(),
        );
    }

    let connectome = Arc::new(RwLock::new(manager));
    let genome = Arc::new(RwLock::new(Some(genome)));
    let svc = ConnectomeServiceImpl::new(Arc::clone(&connectome), Arc::clone(&genome));
    Rig {
        svc,
        connectome,
        genome,
        circuit,
        ipu,
        port,
    }
}

fn rule() -> Value {
    json!({ "morphology_id": "projector" })
}

fn patch(value: Value) -> HashMap<String, Value> {
    value
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

impl Rig {
    fn live_port_encoding(&self) -> Option<Value> {
        self.connectome
            .read()
            .get_brain_region(&self.circuit)
            .unwrap()
            .properties
            .get(PORT_ENCODING_KEY)
            .cloned()
    }

    fn genome_port_encoding(&self) -> Option<Value> {
        self.genome
            .read()
            .as_ref()
            .unwrap()
            .brain_regions
            .get(&self.circuit)
            .unwrap()
            .properties
            .get(PORT_ENCODING_KEY)
            .cloned()
    }

    /// Live and genome copies must match; returns the shared value.
    fn both(&self) -> Option<Value> {
        let live = self.live_port_encoding();
        assert_eq!(
            live,
            self.genome_port_encoding(),
            "genome copy differs from the live region"
        );
        live
    }
}

#[tokio::test]
async fn wiring_author_set_clear_and_delete_keep_both_copies_in_step() {
    let _guard = TEST_LOCK.lock().await;
    let rig = rig();
    let port_key = rig.port.as_base_64();

    // Designating an unwired port stores nothing yet.
    rig.svc
        .update_brain_region(
            &rig.circuit,
            patch(json!({ "designated_inputs": [port_key.clone()] })),
        )
        .await
        .unwrap();
    assert_eq!(rig.both(), None);

    // Wiring an IPU to it derives a `wiring` entry, in the live region and the genome.
    rig.svc
        .update_cortical_mapping(rig.ipu.as_base_64(), port_key.clone(), vec![rule()])
        .await
        .unwrap();
    let derived = rig.both().expect("entry after wiring");
    assert_eq!(
        derived[&port_key],
        json!({
            "kind": "Percentage", "signed": false, "frame": "Absolute",
            "positioning": "Linear", "source": "wiring"
        })
    );

    // An author set stores the merged entry, not the PATCH delta, in both copies.
    rig.svc
        .update_brain_region(
            &rig.circuit,
            patch(json!({ PORT_ENCODING_KEY: { port_key.clone(): {"kind": "Boolean", "signed": false} } })),
        )
        .await
        .unwrap();
    let authored = rig.both().expect("author entry");
    assert_eq!(
        authored[&port_key],
        json!({
            "kind": "Boolean", "signed": false, "frame": null,
            "positioning": null, "source": "author"
        })
    );

    // Rewiring (mapping removed and re-added) leaves the author entry alone.
    rig.svc
        .update_cortical_mapping(rig.ipu.as_base_64(), port_key.clone(), vec![])
        .await
        .unwrap();
    rig.svc
        .update_cortical_mapping(rig.ipu.as_base_64(), port_key.clone(), vec![rule()])
        .await
        .unwrap();
    assert_eq!(rig.both(), Some(authored));

    // Clearing resumes wiring derivation.
    rig.svc
        .update_brain_region(
            &rig.circuit,
            patch(json!({ PORT_ENCODING_KEY: { port_key.clone(): null } })),
        )
        .await
        .unwrap();
    assert_eq!(rig.both(), Some(derived.clone()));

    // Save and reload keep the entry.
    let saved = {
        let genome = rig.genome.read();
        feagi_evolutionary::save_genome_to_json(genome.as_ref().unwrap()).unwrap()
    };
    let reloaded = feagi_evolutionary::load_genome_from_json(&saved).unwrap();
    let region = reloaded
        .brain_regions
        .values()
        .find(|r| r.name == "Circuit")
        .expect("circuit region survives reload");
    let entries = region_port_encoding(region).unwrap();
    assert_eq!(entries[&port_key].kind.as_deref(), Some("Percentage"));
    assert_eq!(entries[&port_key].source, PortEncodingSource::Wiring);

    // Deleting the port area drops its entry from both copies.
    rig.svc.delete_cortical_area(&port_key).await.unwrap();
    assert_eq!(rig.both(), None);
}

#[tokio::test]
async fn invalid_author_entries_are_refused_and_change_nothing() {
    let _guard = TEST_LOCK.lock().await;
    let rig = rig();
    let port_key = rig.port.as_base_64();
    rig.svc
        .update_brain_region(
            &rig.circuit,
            patch(json!({ "designated_inputs": [port_key.clone()] })),
        )
        .await
        .unwrap();

    // Not a designated port: Conflict. Bad vocabulary: InvalidInput.
    let other = rig.ipu.as_base_64();
    let err = rig
        .svc
        .update_brain_region(
            &rig.circuit,
            patch(json!({ PORT_ENCODING_KEY: { other: {"kind": "Boolean"} } })),
        )
        .await
        .expect_err("not a designated port");
    assert!(matches!(err, ServiceError::Conflict(_)), "{err:?}");
    let err = rig
        .svc
        .update_brain_region(
            &rig.circuit,
            patch(json!({ PORT_ENCODING_KEY: { port_key: {"kind": "Wobbly"} } })),
        )
        .await
        .expect_err("bad vocabulary");
    assert!(matches!(err, ServiceError::InvalidInput(_)), "{err:?}");
    assert_eq!(rig.both(), None);
}

#[tokio::test]
async fn import_keeps_sources_refuses_undesignated_ports_and_wiring_rederives() {
    let _guard = TEST_LOCK.lock().await;
    let rig = rig();
    let port_key = rig.port.as_base_64();
    let stale_wiring = json!({
        "kind": "Misc", "signed": null, "frame": "Incremental",
        "positioning": null, "source": "wiring"
    });
    let entries = |entry: &Value| {
        json!({ port_key.clone(): entry.clone() })
            .as_object()
            .unwrap()
            .clone()
    };

    // Not designated yet: refused, nothing stored.
    let err = rig
        .svc
        .import_region_port_encoding(&rig.circuit, entries(&stale_wiring))
        .await
        .expect_err("port is not designated");
    assert!(matches!(err, ServiceError::Conflict(_)), "{err:?}");
    assert_eq!(rig.both(), None);

    rig.svc
        .update_brain_region(
            &rig.circuit,
            patch(json!({ "designated_inputs": [port_key.clone()] })),
        )
        .await
        .unwrap();

    // A `wiring` entry arrives as `wiring`, in the live region and the genome.
    rig.svc
        .import_region_port_encoding(&rig.circuit, entries(&stale_wiring))
        .await
        .unwrap();
    assert_eq!(rig.both(), Some(json!({ port_key.clone(): stale_wiring })));

    // The next recompute (a mapping change) updates the wiring entry from the real wiring.
    rig.svc
        .update_cortical_mapping(rig.ipu.as_base_64(), port_key.clone(), vec![rule()])
        .await
        .unwrap();
    assert_eq!(
        rig.both().unwrap()[&port_key]["kind"],
        "Percentage",
        "wiring entry re-derived"
    );

    // An `author` entry arrives as `author` and survives that wiring.
    let author = json!({
        "kind": "Boolean", "signed": false, "frame": null,
        "positioning": null, "source": "author"
    });
    rig.svc
        .import_region_port_encoding(&rig.circuit, entries(&author))
        .await
        .unwrap();
    assert_eq!(rig.both(), Some(json!({ port_key.clone(): author })));

    // A malformed entry is refused.
    let err = rig
        .svc
        .import_region_port_encoding(
            &rig.circuit,
            entries(&json!({"kind": "Wobbly", "source": "author"})),
        )
        .await
        .expect_err("bad vocabulary");
    assert!(matches!(err, ServiceError::InvalidInput(_)), "{err:?}");
}
