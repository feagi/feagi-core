// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Two independent FEAGI service stacks: changes recorded on one are replayed on
//! the other through the change ledger, and both end in the same state.
//!
//! Instances are seeded directly rather than through `load_genome`, which develops
//! into the process-wide `ConnectomeManager` instead of the injected one.

use feagi_brain_development::ConnectomeManager;
use feagi_evolutionary::create_genome_with_core_morphologies;
use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::{DynamicNPU, RustNPU, TracingMutex};
use feagi_npu_runtime::StdRuntime;
use feagi_services::change_ledger::{
    apply_change, with_change_context, ApplyGenomeChangeOutcome, ApplyGenomeChangeRequest,
    ChangeContext, ChangeLedger, ChangeLedgerConfig, ChangeOrigin, GenomeChange,
    GenomeChangeOperation, RecordingConnectomeService, RecordingGenomeService,
};
use feagi_services::types::{
    CorticalAreaInfo, CreateBrainRegionParams, CreateCorticalAreaParams, GenomeInfo,
    LoadGenomeParams, SaveGenomeParams, ServiceError, ServiceResult,
};
use feagi_services::{ConnectomeService, ConnectomeServiceImpl, GenomeService, GenomeServiceImpl};
use feagi_structures::genomic::cortical_area::CorticalID;
use parking_lot::RwLock;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

const LEDGER_CONFIG: ChangeLedgerConfig = ChangeLedgerConfig {
    session_capacity: 1_000,
    max_persisted_entries: 1_000,
    agent_id_max_length: 64,
};

struct Instance {
    ledger: Arc<ChangeLedger>,
    genome: Arc<dyn GenomeService + Send + Sync>,
    connectome: Arc<dyn ConnectomeService + Send + Sync>,
}

const AREA_IDS: [&[u8; 8]; 2] = [b"cledgera", b"cledgerb"];

fn custom_area(id: &[u8; 8], name: &str, x: i32) -> CreateCorticalAreaParams {
    CreateCorticalAreaParams {
        cortical_id: CorticalID::try_from_bytes(id)
            .expect("cortical id")
            .as_base_64(),
        name: name.to_string(),
        dimensions: (2, 2, 1),
        position: (x, 0, 0),
        area_type: "Custom".to_string(),
        visible: None,
        sub_group: None,
        neurons_per_voxel: None,
        postsynaptic_current: None,
        plasticity_constant: None,
        degeneration: None,
        psp_uniform_distribution: None,
        firing_threshold_increment: None,
        firing_threshold_limit: None,
        consecutive_fire_count: None,
        snooze_period: None,
        refractory_period: None,
        leak_coefficient: None,
        leak_variability: None,
        burst_engine_active: None,
        properties: None,
    }
}

impl Instance {
    /// An isolated service stack holding the same two custom areas as every other
    /// instance (created before the ledger is attached, so they are not recorded).
    async fn with_shared_areas(name: &'static str) -> Self {
        let npu = RustNPU::new(StdRuntime, CPUBackend::new(), 100_000, 1_000_000, 10)
            .expect("create NPU");
        let npu = Arc::new(TracingMutex::new(DynamicNPU::F32(npu), name));
        let manager = Arc::new(RwLock::new(ConnectomeManager::new_for_testing_with_npu(
            npu,
        )));
        manager.write().setup_core_morphologies_for_testing();

        let genome_impl = Arc::new(GenomeServiceImpl::new(Arc::clone(&manager)));
        let current_genome = genome_impl.get_current_genome_arc();
        *current_genome.write() = Some(create_genome_with_core_morphologies(
            "shared".to_string(),
            "shared".to_string(),
        ));
        let mut connectome_impl =
            ConnectomeServiceImpl::new(Arc::clone(&manager), current_genome.clone());
        connectome_impl.set_genome_load_signals(
            genome_impl.get_genome_load_counter_arc(),
            genome_impl.get_genome_load_timestamp_arc(),
        );
        let connectome_impl: Arc<dyn ConnectomeService + Send + Sync> = Arc::new(connectome_impl);
        genome_impl
            .create_cortical_areas(vec![
                custom_area(AREA_IDS[0], "area a", 0),
                custom_area(AREA_IDS[1], "area b", 10),
            ])
            .await
            .expect("seed shared areas");

        let ledger = Arc::new(ChangeLedger::new(LEDGER_CONFIG, current_genome).expect("ledger"));
        let genome: Arc<dyn GenomeService + Send + Sync> = Arc::new(RecordingGenomeService::new(
            genome_impl,
            Arc::clone(&connectome_impl),
            Arc::clone(&ledger),
        ));
        let connectome: Arc<dyn ConnectomeService + Send + Sync> = Arc::new(
            RecordingConnectomeService::new(connectome_impl, Arc::clone(&ledger)),
        );
        Self {
            ledger,
            genome,
            connectome,
        }
    }

    fn local_changes(&self) -> Vec<GenomeChange> {
        self.ledger
            .changes_since(0, LEDGER_CONFIG.session_capacity)
            .expect("changes")
            .changes
            .into_iter()
            .filter(|c| c.replayable && c.origin == ChangeOrigin::Local)
            .collect()
    }

    async fn replay_from(&self, change: &GenomeChange) -> ApplyGenomeChangeOutcome {
        let request: ApplyGenomeChangeRequest =
            serde_json::from_value(serde_json::to_value(change).expect("serialize"))
                .expect("a peer's change deserializes as an apply request");
        apply_change(
            request,
            &self.ledger,
            self.genome.as_ref(),
            self.connectome.as_ref(),
        )
        .await
        .expect("apply")
    }

    async fn saved_history_ids(&self) -> Vec<String> {
        let saved = self
            .genome
            .save_genome(SaveGenomeParams {
                genome_id: None,
                genome_title: None,
            })
            .await
            .expect("save");
        let saved: Value = serde_json::from_str(&saved).expect("saved JSON");
        saved["change_history"]
            .as_array()
            .map(|entries| {
                entries
                    .iter()
                    .map(|e| e["change_id"].as_str().expect("change_id").to_string())
                    .collect()
            })
            .unwrap_or_default()
    }
}

fn member(agent_id: &str) -> ChangeContext {
    ChangeContext {
        agent_id: Some(agent_id.to_string()),
        group_id: Some(format!("{agent_id}-request")),
        replay: None,
    }
}

fn two_area_ids() -> (String, String) {
    let id = |raw: &[u8; 8]| CorticalID::try_from_bytes(raw).expect("id").as_base_64();
    (id(AREA_IDS[0]), id(AREA_IDS[1]))
}

/// Stand-in genome backend: the subject here is the recording wrapper, not loading.
struct LoadOnlyGenomeService;

#[async_trait::async_trait]
impl GenomeService for LoadOnlyGenomeService {
    async fn load_genome(&self, _params: LoadGenomeParams) -> ServiceResult<GenomeInfo> {
        Ok(GenomeInfo {
            genome_id: "loaded-id".to_string(),
            genome_title: "loaded title".to_string(),
            version: "3.0".to_string(),
            cortical_area_count: 0,
            brain_region_count: 0,
            simulation_timestep: 0.01,
            genome_num: None,
            genome_timestamp: None,
        })
    }
    async fn save_genome(&self, _params: SaveGenomeParams) -> ServiceResult<String> {
        Err(ServiceError::NotImplemented("save".to_string()))
    }
    async fn export_region_genome(&self, _region_id: String) -> ServiceResult<String> {
        Err(ServiceError::NotImplemented("export".to_string()))
    }
    async fn get_genome_info(&self) -> ServiceResult<GenomeInfo> {
        Err(ServiceError::NotImplemented("info".to_string()))
    }
    async fn validate_genome(&self, _json_str: String) -> ServiceResult<bool> {
        Ok(true)
    }
    async fn reset_connectome(&self) -> ServiceResult<()> {
        Ok(())
    }
    async fn update_cortical_area(
        &self,
        cortical_id: &str,
        _changes: HashMap<String, Value>,
    ) -> ServiceResult<CorticalAreaInfo> {
        Err(ServiceError::NotFound {
            resource: "cortical_area".to_string(),
            id: cortical_id.to_string(),
        })
    }
    async fn create_cortical_areas(
        &self,
        _params: Vec<CreateCorticalAreaParams>,
    ) -> ServiceResult<Vec<CorticalAreaInfo>> {
        Err(ServiceError::NotImplemented("create".to_string()))
    }
    async fn list_modulator_types(&self) -> ServiceResult<Vec<String>> {
        Ok(Vec::new())
    }
    async fn list_modulators(&self) -> ServiceResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
    async fn get_modulator(&self, id: &str) -> ServiceResult<serde_json::Value> {
        Err(ServiceError::NotFound {
            resource: "modulator".to_string(),
            id: id.to_string(),
        })
    }
    async fn create_modulator(
        &self,
        _id: String,
        _write: feagi_evolutionary::ModulatorWrite,
    ) -> ServiceResult<serde_json::Value> {
        Err(ServiceError::NotImplemented("create modulator".to_string()))
    }
    async fn update_modulator(
        &self,
        _id: &str,
        _write: feagi_evolutionary::ModulatorWrite,
    ) -> ServiceResult<serde_json::Value> {
        Err(ServiceError::NotImplemented("update modulator".to_string()))
    }
    async fn rename_modulator(&self, _old_id: &str, _new_id: String) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented("rename modulator".to_string()))
    }
    async fn modulator_usage(&self, _id: &str) -> ServiceResult<serde_json::Value> {
        Ok(serde_json::json!({ "areas": [], "mappings": [] }))
    }
    async fn delete_modulator(&self, _id: &str, _force: bool) -> ServiceResult<()> {
        Err(ServiceError::NotImplemented("delete modulator".to_string()))
    }
}

#[tokio::test]
async fn load_and_reset_are_recorded_as_non_replayable_markers() {
    let shared = Instance::with_shared_areas("ledger-markers").await;
    let genome = RecordingGenomeService::new(
        Arc::new(LoadOnlyGenomeService),
        Arc::clone(&shared.connectome),
        Arc::clone(&shared.ledger),
    );
    genome
        .load_genome(LoadGenomeParams {
            json_str: String::new(),
        })
        .await
        .expect("load");
    genome.reset_connectome().await.expect("reset");
    assert!(genome
        .update_cortical_area("missing", HashMap::new())
        .await
        .is_err());

    let page = shared.ledger.changes_since(0, 10).expect("page");
    assert_eq!(page.changes.len(), 2, "the failed update is not recorded");
    assert!(matches!(
        &page.changes[0].operation,
        GenomeChangeOperation::GenomeLoaded { genome_id, .. } if genome_id == "loaded-id"
    ));
    assert!(matches!(
        page.changes[1].operation,
        GenomeChangeOperation::ConnectomeReset
    ));
    assert!(page.changes.iter().all(|c| !c.replayable));
    assert!(shared.saved_history_ids().await.is_empty());
}

#[tokio::test]
async fn area_update_is_attributed_and_replayed_with_original_identity() {
    let a = Instance::with_shared_areas("ledger-update-a").await;
    let b = Instance::with_shared_areas("ledger-update-b").await;
    let (area_id, _) = two_area_ids();

    with_change_context(member("member-a"), async {
        a.genome
            .update_cortical_area(
                &area_id,
                HashMap::from([("cortical_name".to_string(), json!("renamed by a"))]),
            )
            .await
            .expect("update on A")
    })
    .await;

    let changes = a.local_changes();
    assert_eq!(changes.len(), 1);
    let change = &changes[0];
    assert_eq!(change.agent_id.as_deref(), Some("member-a"));
    assert_eq!(change.group_id, "member-a-request");
    assert!(change.before.is_some() && change.after.is_some());
    assert_ne!(change.before, change.after);

    let outcome = b.replay_from(change).await;
    assert!(matches!(outcome, ApplyGenomeChangeOutcome::Applied { .. }));
    let on_b = b
        .connectome
        .get_cortical_area(&area_id)
        .await
        .expect("area on B");
    assert_eq!(on_b.name, "renamed by a");

    let replayed = b.ledger.find(&change.change_id).expect("recorded on B");
    assert_eq!(replayed.origin, ChangeOrigin::Replayed);
    assert_eq!(replayed.agent_id.as_deref(), Some("member-a"));
    assert_eq!(replayed.timestamp_ms, change.timestamp_ms);
    assert!(
        b.local_changes().is_empty(),
        "replayed changes are not local"
    );

    assert_eq!(a.saved_history_ids().await, vec![change.change_id.clone()]);
    assert_eq!(b.saved_history_ids().await, vec![change.change_id.clone()]);
}

#[tokio::test]
async fn replaying_the_same_change_twice_is_a_no_op() {
    let a = Instance::with_shared_areas("ledger-idem-a").await;
    let b = Instance::with_shared_areas("ledger-idem-b").await;
    let (area_id, _) = two_area_ids();
    a.genome
        .update_cortical_area(
            &area_id,
            HashMap::from([("cortical_name".to_string(), json!("once"))]),
        )
        .await
        .expect("update on A");
    let change = a.local_changes().remove(0);

    assert!(matches!(
        b.replay_from(&change).await,
        ApplyGenomeChangeOutcome::Applied { .. }
    ));
    assert!(matches!(
        b.replay_from(&change).await,
        ApplyGenomeChangeOutcome::AlreadyApplied
    ));
    assert!(matches!(
        a.replay_from(&change).await,
        ApplyGenomeChangeOutcome::AlreadyApplied
    ));
    assert_eq!(b.saved_history_ids().await.len(), 1);
}

#[tokio::test]
async fn region_mapping_and_delete_replay_to_same_state() {
    let a = Instance::with_shared_areas("ledger-struct-a").await;
    let b = Instance::with_shared_areas("ledger-struct-b").await;
    let (src, dst) = two_area_ids();

    with_change_context(member("member-a"), async {
        a.connectome
            .create_brain_region(CreateBrainRegionParams {
                region_id: "6a1f0e8e-0000-4000-8000-000000000001".to_string(),
                name: "shared region".to_string(),
                region_type: "Undefined".to_string(),
                parent_id: None,
                properties: None,
            })
            .await
            .expect("create region on A");
        a.connectome
            .update_cortical_mapping(
                src.clone(),
                dst.clone(),
                vec![json!({
                    "morphology_id": "projector",
                    "morphology_scalar": [1, 1, 1],
                    "postSynapticCurrent_multiplier": 1.0,
                    "plasticity_flag": false
                })],
            )
            .await
            .expect("map on A");
        a.connectome
            .delete_cortical_area(&dst)
            .await
            .expect("delete on A");
    })
    .await;

    let changes = a.local_changes();
    assert_eq!(changes.len(), 3);
    let delete = &changes[2];
    assert!(
        delete.before.as_ref().expect("delete before")["incoming_mappings"]
            .get(&src)
            .is_some(),
        "delete snapshot keeps the mapping that pointed at the deleted area"
    );
    for change in &changes {
        assert!(matches!(
            b.replay_from(change).await,
            ApplyGenomeChangeOutcome::Applied { .. }
        ));
    }

    assert!(b
        .connectome
        .brain_region_exists("6a1f0e8e-0000-4000-8000-000000000001")
        .await
        .expect("region lookup"));
    assert!(!b
        .connectome
        .cortical_area_exists(&dst)
        .await
        .expect("area lookup"));
    let mut a_ids = a.connectome.get_cortical_area_ids().await.expect("ids");
    let mut b_ids = b.connectome.get_cortical_area_ids().await.expect("ids");
    a_ids.sort();
    b_ids.sort();
    assert_eq!(a_ids, b_ids);
    assert_eq!(a.saved_history_ids().await, b.saved_history_ids().await);
}

#[tokio::test]
async fn replay_of_change_to_missing_target_fails_and_is_not_recorded() {
    let a = Instance::with_shared_areas("ledger-conflict-a").await;
    let b = Instance::with_shared_areas("ledger-conflict-b").await;
    let (area_id, _) = two_area_ids();
    a.genome
        .update_cortical_area(
            &area_id,
            HashMap::from([("cortical_name".to_string(), json!("late edit"))]),
        )
        .await
        .expect("update on A");
    let change = a.local_changes().remove(0);
    b.connectome
        .delete_cortical_area(&area_id)
        .await
        .expect("delete on B");

    let request: ApplyGenomeChangeRequest =
        serde_json::from_value(serde_json::to_value(&change).expect("serialize")).expect("request");
    let result = apply_change(request, &b.ledger, b.genome.as_ref(), b.connectome.as_ref()).await;
    assert!(result.is_err());
    assert!(!b.ledger.contains_change_id(&change.change_id));
}

#[tokio::test]
async fn markers_cannot_be_replayed() {
    let b = Instance::with_shared_areas("ledger-marker").await;
    let request = ApplyGenomeChangeRequest {
        change_id: "marker".to_string(),
        group_id: "marker".to_string(),
        timestamp_ms: 0,
        agent_id: None,
        operation: GenomeChangeOperation::ConnectomeReset,
    };
    assert!(
        apply_change(request, &b.ledger, b.genome.as_ref(), b.connectome.as_ref())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn failed_mutation_is_not_recorded() {
    let a = Instance::with_shared_areas("ledger-fail").await;
    let before = a.ledger.latest_sequence();
    assert!(a
        .connectome
        .delete_cortical_area("bm9fc3VjaF9h")
        .await
        .is_err());
    assert_eq!(a.ledger.latest_sequence(), before);
}
