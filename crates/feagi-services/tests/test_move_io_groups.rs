// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Atomic moves of IO cortical areas between group (unit) numbers, through the service layer
//! with a live NPU: validation before any write, swaps, kept synapses, resolved references,
//! injection at the new id, and a save/reload round trip.

use std::collections::HashMap;
use std::sync::Arc;

use feagi_brain_development::ConnectomeManager;
use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::{DynamicNPU, RustNPU, TracingMutex};
use feagi_npu_runtime::StdRuntime;
use feagi_services::impls::GenomeServiceImpl;
use feagi_services::traits::GenomeService;
use feagi_services::types::{IoGroupMove, SaveGenomeParams, ServiceError};
use feagi_structures::genomic::brain_regions::{BrainRegion, RegionID, RegionType};
use feagi_structures::genomic::classifiers::Classifier;
use feagi_structures::genomic::cortical_area::{
    CorticalArea, CorticalAreaDimensions, CorticalAreaType, CorticalID, CustomCorticalType,
    IOCorticalAreaConfigurationFlag,
};
use parking_lot::RwLock;

/// The StateManager is process-global; tests in this file take turns.
static TEST_LOCK: std::sync::LazyLock<tokio::sync::Mutex<()>> =
    std::sync::LazyLock::new(|| tokio::sync::Mutex::new(()));

type SynapseRow = (u32, f32, f32, u8, u8, u16);

fn io_id(is_input: bool, unit: &[u8; 3], group: u16) -> CorticalID {
    IOCorticalAreaConfigurationFlag::Boolean.as_io_cortical_id(
        is_input,
        *unit,
        group.into(),
        0u8.into(),
    )
}

fn io_area(id: CorticalID, idx: u32, name: &str) -> CorticalArea {
    let kind = if id.as_bytes()[0] == b'i' {
        CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean)
    } else {
        CorticalAreaType::BrainOutput(IOCorticalAreaConfigurationFlag::Boolean)
    };
    CorticalArea::new(
        id,
        idx,
        name.to_string(),
        CorticalAreaDimensions::new(1, 1, 1).unwrap(),
        (0, 0, 0).into(),
        kind,
    )
    .unwrap()
}

struct Rig {
    svc: GenomeServiceImpl,
    connectome: Arc<RwLock<ConnectomeManager>>,
    npu: Arc<TracingMutex<DynamicNPU>>,
    /// Input area in group 1, feeding `dst`.
    src: CorticalID,
    /// Input area in group 2 (occupies the group a move to 2 would take).
    other: CorticalID,
    /// Custom area downstream of `src`.
    dst: CorticalID,
    src_neuron: u64,
    dst_neuron: u64,
    region_id: String,
}

fn rig() -> Rig {
    let npu = Arc::new(TracingMutex::new(
        DynamicNPU::F32(RustNPU::new(StdRuntime, CPUBackend::new(), 10_000, 10_000, 10).unwrap()),
        "TestNPU",
    ));
    let mut manager = ConnectomeManager::new_for_testing_with_npu(Arc::clone(&npu));
    manager.setup_core_morphologies_for_testing();

    let src = io_id(true, b"tst", 1);
    let other = io_id(true, b"tst", 2);
    let dst = CorticalID::try_from_bytes(b"cdstmv01").unwrap();
    manager.add_cortical_area(io_area(src, 0, "src")).unwrap();
    manager
        .add_cortical_area(io_area(other, 0, "other"))
        .unwrap();
    manager
        .add_cortical_area(
            CorticalArea::new(
                dst,
                0,
                "dst".to_string(),
                CorticalAreaDimensions::new(1, 1, 1).unwrap(),
                (0, 0, 0).into(),
                CorticalAreaType::Custom(CustomCorticalType::LeakyIntegrateFire),
            )
            .unwrap(),
        )
        .unwrap();
    let src_neuron = manager
        .add_neuron(
            &src,
            0,
            0,
            0,
            0.5,
            f32::MAX,
            0.0,
            0.0,
            0,
            0,
            1.0,
            3,
            1,
            false,
        )
        .unwrap();
    let dst_neuron = manager
        .add_neuron(
            &dst,
            0,
            0,
            0,
            0.5,
            f32::MAX,
            0.0,
            0.0,
            0,
            0,
            1.0,
            3,
            1,
            false,
        )
        .unwrap();
    manager
        .create_synapse(src_neuron, dst_neuron, 255.0, 255.0, 0)
        .unwrap();
    npu.lock().unwrap().rebuild_synapse_index();

    // Reference carriers: mapping rule field, memory twin properties, region IO list, classifier.
    let src_b64 = src.as_base_64();
    let dst_b64 = dst.as_base_64();
    manager
        .get_cortical_area_mut(&src)
        .unwrap()
        .properties
        .insert(
            "cortical_mapping_dst".to_string(),
            serde_json::json!({ dst_b64.clone(): [
            { "morphology_id": "projector", "gate_source_area": src_b64.clone() }
        ] }),
        );
    {
        let dst_area = manager.get_cortical_area_mut(&dst).unwrap();
        dst_area
            .properties
            .insert("memory_twin_of".to_string(), serde_json::json!(src_b64));
        dst_area.properties.insert(
            "memory_twin_areas".to_string(),
            serde_json::json!({ src_b64.clone(): dst_b64.clone() }),
        );
    }
    let region_id = RegionID::new();
    let region_key = region_id.to_string();
    let mut region = BrainRegion::new(region_id, "r".to_string(), RegionType::Undefined).unwrap();
    region.cortical_areas.insert(src);
    region.cortical_areas.insert(dst);
    region.properties.insert(
        "designated_inputs".to_string(),
        serde_json::json!([src_b64.clone()]),
    );
    manager.add_brain_region(region, None).unwrap();
    manager.upsert_classifier(Classifier {
        classifier_id: "cls".to_string(),
        name: "cls".to_string(),
        parent_region_id: region_key.clone(),
        coordinates_3d: [0, 0, 0],
        sample_area_ids: vec![src_b64.clone()],
        class_bindings: Vec::new(),
        sample_memory_id: String::new(),
        class_memory_id: String::new(),
        error_area_id: String::new(),
        correct_area_id: String::new(),
        reward_training: false,
        answer_feedback_area_id: None,
        reward_modulator_ids: Vec::new(),
        answer_latency_bursts: 0,
        learn_area_id: None,
        confidence_area_id: None,
        properties: HashMap::new(),
    });

    let connectome = Arc::new(RwLock::new(manager));
    let svc = GenomeServiceImpl::new(Arc::clone(&connectome));
    // The genome store mirrors the live areas, as after a genome load.
    {
        let manager = connectome.read();
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
        for id in [src, other, dst] {
            genome
                .cortical_areas
                .insert(id, manager.get_cortical_area(&id).unwrap().clone());
        }
        genome.brain_regions.insert(
            region_key.clone(),
            manager.get_brain_region(&region_key).unwrap().clone(),
        );
        genome.classifiers.insert(
            "cls".to_string(),
            manager.get_classifier("cls").unwrap().clone(),
        );
        *svc.get_current_genome_arc().write() = Some(genome);
    }
    Rig {
        svc,
        connectome,
        npu,
        src,
        other,
        dst,
        src_neuron,
        dst_neuron,
        region_id: region_key,
    }
}

fn mv(id: &CorticalID, group: u16) -> IoGroupMove {
    IoGroupMove {
        cortical_id: id.as_base_64(),
        group_id: group,
    }
}

fn genome_ids(rig: &Rig) -> Vec<String> {
    let guard = rig.svc.get_current_genome_arc();
    let genome = guard.read();
    let mut ids: Vec<String> = genome
        .as_ref()
        .unwrap()
        .cortical_areas
        .keys()
        .map(|id| id.as_base_64())
        .collect();
    ids.sort();
    ids
}

fn synapse_snapshot(rig: &Rig) -> (usize, Vec<SynapseRow>) {
    let npu = rig.npu.lock().unwrap();
    (
        npu.get_synapse_count(),
        npu.get_outgoing_synapses_detailed(rig.src_neuron as u32),
    )
}

/// Inject at `id` and burst twice; returns `(injected, fired neuron ids over both bursts)`.
fn inject_and_burst(rig: &Rig, id: &CorticalID) -> (usize, Vec<u32>) {
    let mut npu = rig.npu.lock().unwrap();
    let injected = npu.inject_sensory_xyzp_by_id(id, &[(0, 0, 0, 10.0)]);
    let mut fired = Vec::new();
    for _ in 0..3 {
        let result = npu.process_burst().unwrap();
        fired.extend(result.fired_neurons.iter().map(|n| n.0));
    }
    (injected, fired)
}

#[tokio::test]
async fn occupied_target_is_refused_and_genome_is_byte_identical() {
    let _guard = TEST_LOCK.lock().await;
    let rig = rig();
    let before = rig
        .svc
        .save_genome(SaveGenomeParams {
            genome_id: None,
            genome_title: None,
        })
        .await
        .unwrap();

    let err = rig
        .svc
        .move_io_groups(vec![mv(&rig.src, 2)])
        .await
        .expect_err("group 2 is taken by another area");
    assert!(matches!(err, ServiceError::Conflict(_)), "{err:?}");

    let after = rig
        .svc
        .save_genome(SaveGenomeParams {
            genome_id: None,
            genome_title: None,
        })
        .await
        .unwrap();
    assert_eq!(
        before, after,
        "refused move must leave the genome byte-identical"
    );
    assert!(rig.connectome.read().get_cortical_area(&rig.src).is_some());
}

#[tokio::test]
async fn batch_with_one_bad_entry_changes_nothing() {
    let _guard = TEST_LOCK.lock().await;
    let rig = rig();
    let ids_before = genome_ids(&rig);
    let missing = io_id(true, b"tst", 9);

    // First entry is valid (group 5 is free); second names an area that does not exist.
    let err = rig
        .svc
        .move_io_groups(vec![mv(&rig.src, 5), mv(&missing, 6)])
        .await
        .expect_err("a missing source fails the whole batch");
    assert!(matches!(err, ServiceError::Conflict(_)), "{err:?}");
    assert_eq!(
        genome_ids(&rig),
        ids_before,
        "0 areas changed in the genome"
    );
    {
        let manager = rig.connectome.read();
        assert!(manager.get_cortical_area(&rig.src).is_some());
        assert!(manager.get_cortical_area(&io_id(true, b"tst", 5)).is_none());
    }

    // A non-IO area is refused the same way.
    let err = rig
        .svc
        .move_io_groups(vec![mv(&rig.dst, 3)])
        .await
        .expect_err("custom areas have no group number");
    assert!(matches!(err, ServiceError::Conflict(_)), "{err:?}");
    assert_eq!(genome_ids(&rig), ids_before);
}

#[tokio::test]
async fn swap_succeeds_and_keeps_synapses_and_references() {
    let _guard = TEST_LOCK.lock().await;
    let rig = rig();
    let before = synapse_snapshot(&rig);
    assert_eq!(before.0, 1);
    let (src, other) = (rig.src, rig.other);

    let moved = rig
        .svc
        .move_io_groups(vec![mv(&src, 2), mv(&other, 1)])
        .await
        .expect("a swap is allowed");
    assert_eq!(moved.len(), 2);

    // The ids swapped: the old `src` id now names the area that was `other`.
    let after = synapse_snapshot(&rig);
    assert_eq!(before, after, "synapse count and weights: 0 difference");
    let manager = rig.connectome.read();
    assert_eq!(
        manager.get_cortical_area(&src).unwrap().name,
        "other",
        "the id that used to be `src` now names the former `other`"
    );
    assert_eq!(manager.get_cortical_area(&other).unwrap().name, "src");
}

#[tokio::test]
async fn move_keeps_synapses_and_resolves_every_reference() {
    let _guard = TEST_LOCK.lock().await;
    let rig = rig();
    let before = synapse_snapshot(&rig);
    let old = rig.src.as_base_64();
    let new_id = io_id(true, b"tst", 7);
    let new = new_id.as_base_64();

    let moved = rig
        .svc
        .move_io_groups(vec![mv(&rig.src, 7)])
        .await
        .expect("group 7 is free");
    assert_eq!(moved.len(), 1);
    assert_eq!(
        (moved[0].old_id.as_str(), moved[0].new_id.as_str()),
        (old.as_str(), new.as_str())
    );

    assert_eq!(before, synapse_snapshot(&rig), "0 difference in synapses");

    // Dangling references: the old id must appear nowhere.
    let manager = rig.connectome.read();
    assert!(manager.get_cortical_area(&rig.src).is_none());
    let moved_area = manager.get_cortical_area(&new_id).expect("area at new id");
    assert_eq!(moved_area.properties["group_id"], serde_json::json!(7));
    let rule = &moved_area.properties["cortical_mapping_dst"][rig.dst.as_base_64()][0];
    assert_eq!(
        rule["gate_source_area"],
        serde_json::json!(new),
        "mapping rule field"
    );
    let dst_area = manager.get_cortical_area(&rig.dst).unwrap();
    assert_eq!(
        dst_area.properties["memory_twin_of"],
        serde_json::json!(new)
    );
    assert_eq!(
        dst_area.properties["memory_twin_areas"],
        serde_json::json!({ new.clone(): rig.dst.as_base_64() })
    );
    let region = manager.get_brain_region(&rig.region_id).unwrap();
    assert!(region.cortical_areas.contains(&new_id) && !region.cortical_areas.contains(&rig.src));
    assert_eq!(
        region.properties["designated_inputs"],
        serde_json::json!([new.clone()])
    );
    assert_eq!(
        manager.get_classifier("cls").unwrap().sample_area_ids,
        vec![new.clone()]
    );
    drop(manager);

    let genome = rig.svc.get_current_genome_arc();
    let genome = genome.read();
    let genome = genome.as_ref().unwrap();
    let text = serde_json::to_string(&(
        genome
            .cortical_areas
            .values()
            .map(|a| &a.properties)
            .collect::<Vec<_>>(),
        genome
            .brain_regions
            .values()
            .map(|r| &r.properties)
            .collect::<Vec<_>>(),
        &genome.classifiers,
    ))
    .unwrap();
    assert!(
        !text.contains(&old),
        "0 dangling references to the old id in the genome"
    );
    assert!(text.contains(&new));
}

#[tokio::test]
async fn injection_resolves_new_id_and_not_old_and_survives_reload() {
    let _guard = TEST_LOCK.lock().await;
    let rig = rig();

    // Baseline at the old id.
    let (injected, fired) = inject_and_burst(&rig, &rig.src);
    assert!(injected > 0, "baseline injection at the old id works");
    assert!(
        fired.contains(&(rig.dst_neuron as u32)),
        "baseline: downstream fires"
    );

    let new_id = io_id(true, b"tst", 7);
    rig.svc.move_io_groups(vec![mv(&rig.src, 7)]).await.unwrap();

    let (injected_old, _) = inject_and_burst(&rig, &rig.src);
    assert_eq!(injected_old, 0, "injection at the old id returns 0");
    let (injected_new, fired) = inject_and_burst(&rig, &new_id);
    assert!(injected_new > 0, "injection at the new id returns > 0");
    assert!(
        fired.contains(&(rig.dst_neuron as u32)),
        "the downstream area fires after injection at the new id"
    );

    // Save and reload: every area at its new id, none at the old.
    let json = rig
        .svc
        .save_genome(SaveGenomeParams {
            genome_id: None,
            genome_title: None,
        })
        .await
        .unwrap();
    let reloaded = feagi_evolutionary::load_genome_from_json(&json).expect("reload");
    let at_new = [new_id, rig.other, rig.dst]
        .iter()
        .filter(|id| reloaded.cortical_areas.contains_key(*id))
        .count();
    let at_old = [rig.src]
        .iter()
        .filter(|id| reloaded.cortical_areas.contains_key(*id))
        .count();
    assert_eq!(
        (at_new, at_old),
        (3, 0),
        "3 of 3 areas at new ids, 0 at the old"
    );
}

/// A move takes genome -> connectome -> NPU. Run 250 moves while another thread bursts and
/// injects sensory data under the NPU lock and a third reads genome then connectome (the order
/// the services use). Must finish well inside the timeout: a lock-order inversion would hang.
#[test]
fn moves_complete_while_bursts_and_injection_run() {
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::time::Duration;

    const MOVES: usize = 250;
    let _guard = TEST_LOCK.blocking_lock();
    let rig = Arc::new(rig());
    let stop = Arc::new(AtomicBool::new(false));
    let bursts = Arc::new(AtomicU64::new(0));
    let injected = Arc::new(AtomicU64::new(0));
    let reads = Arc::new(AtomicU64::new(0));

    let burster = {
        let (rig, stop, bursts, injected) = (
            Arc::clone(&rig),
            Arc::clone(&stop),
            Arc::clone(&bursts),
            Arc::clone(&injected),
        );
        std::thread::spawn(move || {
            let ids = [io_id(true, b"tst", 1), io_id(true, b"tst", 3)];
            while !stop.load(Ordering::Relaxed) {
                let mut npu = rig.npu.lock().unwrap();
                for id in &ids {
                    injected.fetch_add(
                        npu.inject_sensory_xyzp_by_id(id, &[(0, 0, 0, 10.0)]) as u64,
                        Ordering::Relaxed,
                    );
                }
                npu.process_burst().unwrap();
                bursts.fetch_add(1, Ordering::Relaxed);
                drop(npu);
                // The real loop paces itself by the burst rate and releases the lock between
                // bursts; a std mutex is not fair, so a spin would starve every other thread.
                std::thread::sleep(Duration::from_micros(300));
            }
        })
    };
    let reader = {
        let (rig, stop, reads) = (Arc::clone(&rig), Arc::clone(&stop), Arc::clone(&reads));
        std::thread::spawn(move || {
            let genome = rig.svc.get_current_genome_arc();
            while !stop.load(Ordering::Relaxed) {
                // Same order as the services: genome, then connectome.
                let genome_guard = genome.read();
                let manager = rig.connectome.read();
                let _ = (
                    genome_guard.as_ref().map(|g| g.cortical_areas.len()),
                    manager.get_cortical_area_count(),
                );
                reads.fetch_add(1, Ordering::Relaxed);
                drop(manager);
                drop(genome_guard);
                std::thread::sleep(Duration::from_micros(300));
            }
        })
    };

    let (done_tx, done_rx) = std::sync::mpsc::channel();
    let mover = {
        let rig = Arc::clone(&rig);
        std::thread::spawn(move || {
            let mut current = rig.src;
            for i in 0..MOVES {
                let group = if i % 2 == 0 { 3 } else { 1 };
                let renamed = rig
                    .svc
                    .apply_io_group_moves(&[(current, group)])
                    .expect("move");
                current = renamed[0].1;
            }
            let _ = done_tx.send(current);
        })
    };
    let finished = done_rx.recv_timeout(Duration::from_secs(60));
    stop.store(true, Ordering::Relaxed);
    let finished = finished.expect("moves did not finish in 60s: possible deadlock");
    mover.join().unwrap();
    burster.join().unwrap();
    reader.join().unwrap();

    assert_eq!(
        finished,
        io_id(true, b"tst", 1),
        "an even count of moves returns to group 1"
    );
    assert!(
        bursts.load(Ordering::Relaxed) > 0,
        "no burst ran during the moves"
    );
    assert!(
        injected.load(Ordering::Relaxed) > 0,
        "no injection landed during the moves"
    );
    assert!(reads.load(Ordering::Relaxed) > 0);
    assert_eq!(
        synapse_snapshot(&rig).0,
        1,
        "the synapse survived every move"
    );
}

struct NoViz;
impl feagi_npu_burst_engine::VisualizationPublisher for NoViz {
    fn publish_raw_fire_queue_for_agent(
        &self,
        _agent_id: &str,
        _fire_data: feagi_npu_burst_engine::RawFireQueueSnapshot,
    ) -> Result<(), String> {
        Ok(())
    }
}
struct NoMotor;
impl feagi_npu_burst_engine::MotorPublisher for NoMotor {
    fn publish_motor(&self, _agent_id: &str, _data: &[u8]) -> Result<(), String> {
        Ok(())
    }
}

/// Pending scheduled stimuli are keyed by area id; a move re-keys them in the same step.
#[tokio::test]
async fn pending_scheduled_stimuli_follow_a_moved_area() {
    use feagi_npu_burst_engine::stimulus_schedule::ScheduledStimulus;

    let _guard = TEST_LOCK.lock().await;
    let mut rig = rig();
    let runner = Arc::new(RwLock::new(feagi_npu_burst_engine::BurstLoopRunner::new::<
        NoViz,
        NoMotor,
    >(Arc::clone(&rig.npu), None, None, 10.0)));
    rig.svc.set_burst_runner(Arc::clone(&runner));
    let store = runner.read().stimulus_schedules();
    store
        .add(vec![
            ScheduledStimulus {
                burst: 500,
                areas: vec![
                    (rig.src, vec![(0, 0, 0, 1.0)]),
                    (rig.other, vec![(0, 0, 0, 2.0)]),
                ],
            },
            ScheduledStimulus {
                burst: 900,
                areas: vec![(rig.src, vec![(0, 0, 0, 3.0)])],
            },
        ])
        .unwrap();

    rig.svc.move_io_groups(vec![mv(&rig.src, 7)]).await.unwrap();

    let new_src = io_id(true, b"tst", 7);
    let due = store.take_due(10_000);
    assert_eq!(due.len(), 2);
    let ids: Vec<CorticalID> = due
        .iter()
        .flat_map(|entry| entry.areas.iter().map(|(id, _)| *id))
        .collect();
    assert!(!ids.contains(&rig.src), "an entry still targets the old id");
    assert_eq!(ids.iter().filter(|id| **id == new_src).count(), 2);
    assert_eq!(ids.iter().filter(|id| **id == rig.other).count(), 1);
}

/// A designated port that is itself an IO area carries a `port_encoding` entry keyed by its id;
/// moving the area re-keys the entry in the live region and in the genome store.
#[tokio::test]
async fn move_remaps_port_encoding_keys_in_live_region_and_genome() {
    use feagi_structures::genomic::brain_regions::PORT_ENCODING_KEY;

    let _guard = TEST_LOCK.lock().await;
    let rig = rig();
    let old = rig.src.as_base_64();
    let new_id = io_id(true, b"tst", 7);
    let new = new_id.as_base_64();

    // `src` is a designated input and an IPU: the recompute derives its entry from its own flag.
    // Mirror that into the genome store, as the service write-back does.
    rig.connectome
        .write()
        .recompute_brain_region_io_registry()
        .unwrap();
    let entry = {
        let manager = rig.connectome.read();
        let live = manager.get_brain_region(&rig.region_id).unwrap();
        live.properties
            .get(PORT_ENCODING_KEY)
            .cloned()
            .expect("entry derived for the designated IPU")
    };
    assert_eq!(entry[&old]["kind"], "Boolean");
    {
        let genome = rig.svc.get_current_genome_arc();
        let mut genome = genome.write();
        genome
            .as_mut()
            .unwrap()
            .brain_regions
            .get_mut(&rig.region_id)
            .unwrap()
            .properties
            .insert(PORT_ENCODING_KEY.to_string(), entry.clone());
    }

    rig.svc.move_io_groups(vec![mv(&rig.src, 7)]).await.unwrap();

    let expected = serde_json::json!({ new.clone(): entry[&old].clone() });
    let manager = rig.connectome.read();
    assert_eq!(
        manager.get_brain_region(&rig.region_id).unwrap().properties[PORT_ENCODING_KEY],
        expected,
        "live region keys follow the move"
    );
    drop(manager);
    let genome = rig.svc.get_current_genome_arc();
    let genome = genome.read();
    assert_eq!(
        genome.as_ref().unwrap().brain_regions[&rig.region_id].properties[PORT_ENCODING_KEY],
        expected,
        "genome region keys follow the move"
    );
}
