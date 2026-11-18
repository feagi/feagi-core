// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Handler-level tests for `POST /v1/genome/amalgamation_destination` (augment).
//!
//! The harness builds an `ApiState` over the real `ConnectomeServiceImpl` and
//! `GenomeServiceImpl` (they share one `RuntimeGenome`), gives it a host brain (root region,
//! one host region, two host areas, one host modulator), queues a guest genome as the pending
//! amalgamation and calls the handler directly. Fixtures are neutral and generic.

use feagi_api::amalgamation::{
    AmalgamationPending, AmalgamationPendingSummary, AmalgamationSource,
};
use feagi_api::common::{ApiError, Json, Query, State};
use feagi_api::endpoints::genome::post_amalgamation_destination;
use feagi_api::transports::http::server::ApiState;
use feagi_brain_development::ConnectomeManager;
use feagi_evolutionary::templates::create_genome_with_core_areas;
use feagi_evolutionary::{
    GenomeMetadata, GenomeSignatures, GenomeStats, ModulatorInstance, ModulatorRegistry,
    ModulatorWrite, Morphology, MorphologyParameters, MorphologyRegistry, MorphologyType,
    PhysiologyConfig, RuntimeGenome,
};
use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::{DynamicNPU, RustNPU, TracingMutex};
use feagi_npu_runtime::StdRuntime;
use feagi_services::impls::{
    AnalyticsServiceImpl, ConnectomeServiceImpl, GenomeServiceImpl, NeuronServiceImpl,
    SystemServiceImpl,
};
use feagi_services::types::{CreateBrainRegionParams, CreateCorticalAreaParams};
use feagi_services::{ConnectomeService, GenomeService};
use feagi_structures::genomic::brain_regions::{
    BrainRegion, RegionID, RegionType, ROOT_BRAIN_REGION_NAME,
};
use feagi_structures::genomic::classifiers::{Classifier, ClassifierClassBinding};
use feagi_structures::genomic::cortical_area::{
    CorticalArea, CorticalAreaDimensions, CorticalAreaType, CorticalID, CustomCorticalType,
    MemoryCorticalType,
};
use feagi_structures::genomic::descriptors::GenomeCoordinate3D;
use feagi_structures::genomic::ModulatorKind;
use parking_lot::RwLock;
use serde_json::{json, Value};
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

type HandlerResponse = HashMap<String, Value>;

// ---------------------------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------------------------

/// Runtime service that is never exercised by augment.
struct UnusedRuntimeService;

fn unused<T>() -> feagi_services::ServiceResult<T> {
    Err(feagi_services::ServiceError::NotImplemented(
        "UnusedRuntimeService".to_string(),
    ))
}

#[async_trait::async_trait]
impl feagi_services::RuntimeService for UnusedRuntimeService {
    async fn start(&self) -> feagi_services::ServiceResult<()> {
        unused()
    }
    async fn stop(&self) -> feagi_services::ServiceResult<()> {
        unused()
    }
    async fn pause(&self) -> feagi_services::ServiceResult<()> {
        unused()
    }
    async fn resume(&self) -> feagi_services::ServiceResult<()> {
        unused()
    }
    async fn step(&self) -> feagi_services::ServiceResult<()> {
        unused()
    }
    async fn get_status(&self) -> feagi_services::ServiceResult<feagi_services::RuntimeStatus> {
        unused()
    }
    async fn set_frequency(&self, _frequency_hz: f64) -> feagi_services::ServiceResult<()> {
        unused()
    }
    async fn get_burst_count(&self) -> feagi_services::ServiceResult<u64> {
        unused()
    }
    async fn reset_burst_count(&self) -> feagi_services::ServiceResult<()> {
        unused()
    }
    async fn get_fcl_snapshot(&self) -> feagi_services::ServiceResult<Vec<(u64, f32)>> {
        unused()
    }
    async fn get_fcl_snapshot_with_cortical_idx(
        &self,
    ) -> feagi_services::ServiceResult<Vec<(u64, u32, f32)>> {
        unused()
    }
    async fn get_fire_queue_sample(
        &self,
    ) -> feagi_services::ServiceResult<
        HashMap<u32, (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<f32>)>,
    > {
        unused()
    }
    async fn get_fire_ledger_configs(&self) -> feagi_services::ServiceResult<Vec<(u32, usize)>> {
        unused()
    }
    async fn configure_fire_ledger_window(
        &self,
        _cortical_idx: u32,
        _window_size: usize,
    ) -> feagi_services::ServiceResult<()> {
        unused()
    }
    async fn get_fcl_sampler_config(&self) -> feagi_services::ServiceResult<(f64, u32)> {
        unused()
    }
    async fn set_fcl_sampler_config(
        &self,
        _sample_rate: Option<f64>,
        _max_samples: Option<u32>,
    ) -> feagi_services::ServiceResult<()> {
        unused()
    }
    async fn get_area_fcl_sample_rate(&self, _area_id: u32) -> feagi_services::ServiceResult<f64> {
        unused()
    }
    async fn set_area_fcl_sample_rate(
        &self,
        _area_id: u32,
        _sample_rate: f64,
    ) -> feagi_services::ServiceResult<()> {
        unused()
    }
    async fn inject_sensory_by_coordinates(
        &self,
        _cortical_area_name: &str,
        _coordinates: &[(u32, u32, u32, f32)],
        _mode: feagi_services::traits::runtime_service::ManualStimulationMode,
    ) -> feagi_services::ServiceResult<usize> {
        unused()
    }
    async fn register_motor_subscriptions(
        &self,
        _agent_id: &str,
        _cortical_ids: Vec<String>,
        _rate_hz: f64,
    ) -> feagi_services::ServiceResult<()> {
        unused()
    }
    async fn register_visualization_subscriptions(
        &self,
        _agent_id: &str,
        _rate_hz: f64,
    ) -> feagi_services::ServiceResult<()> {
        unused()
    }
    fn unregister_motor_subscriptions(&self, _agent_id: &str) {}
    fn unregister_visualization_subscriptions(&self, _agent_id: &str) {}
    async fn reset_cortical_area_states(
        &self,
        _cortical_indices: &[u32],
    ) -> feagi_services::ServiceResult<Vec<(u32, usize)>> {
        unused()
    }
    fn clear_all_motor_subscriptions(&self) {}
    fn clear_all_visualization_subscriptions(&self) {}
}

const GUEST_TITLE: &str = "Guest Block";

/// A host brain behind a real `ApiState`.
struct TestBrain {
    state: ApiState,
    root_id: String,
    host_area_ids: Vec<String>,
}

/// Which host ids a guest may collide with.
fn host_area_id(tag: u8) -> CorticalID {
    cortical_id(b'c', tag)
}

fn cortical_id(prefix: u8, tag: u8) -> CorticalID {
    let mut bytes = *b"c_______";
    bytes[0] = prefix;
    bytes[1] = tag;
    CorticalID::try_from_bytes(&bytes).expect("cortical id")
}

fn reward_write() -> ModulatorWrite {
    modulator_write(ModulatorKind::Reward)
}

fn modulator_write(kind: ModulatorKind) -> ModulatorWrite {
    ModulatorWrite {
        kind: kind.as_str().to_string(),
        magnitude_percent: 100.0,
        effect_duration_bursts: 1,
        rest_bursts: 0,
        graded: false,
        full_scale_potential: None,
        driver_cortical_id: None,
    }
}

fn new_region_id() -> String {
    RegionID::new().to_string()
}

fn host_area_params(id: &CorticalID, name: &str, region_id: &str) -> CreateCorticalAreaParams {
    CreateCorticalAreaParams {
        cortical_id: id.as_base_64(),
        name: name.to_string(),
        dimensions: (1, 1, 1),
        position: (0, 0, 0),
        area_type: "Custom".to_string(),
        visible: Some(true),
        sub_group: None,
        neurons_per_voxel: Some(1),
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
        properties: Some(HashMap::from([(
            "parent_region_id".to_string(),
            json!(region_id),
        )])),
    }
}

/// Build a host brain: root region, one host region under it, two host areas in that region
/// (ids `host_area_id(b'h')` and `host_area_id(b'i')`) and one host modulator `shared`
/// (reward). `neuron_capacity` sizes the NPU.
async fn build_host(neuron_capacity: usize) -> TestBrain {
    let npu = RustNPU::new(
        StdRuntime,
        CPUBackend::new(),
        neuron_capacity,
        10_000_000,
        10,
    )
    .expect("npu");
    let npu = Arc::new(TracingMutex::new(
        DynamicNPU::F32(npu),
        "amalgamation-handler-test-npu",
    ));
    let manager = Arc::new(RwLock::new(ConnectomeManager::new_for_testing_with_npu(
        Arc::clone(&npu),
    )));
    let genome_service_impl = Arc::new(GenomeServiceImpl::new(Arc::clone(&manager)));
    let current_genome = genome_service_impl.get_current_genome_arc();
    let genome_load_counter = genome_service_impl.get_genome_load_counter_arc();
    let genome_load_timestamp = genome_service_impl.get_genome_load_timestamp_arc();
    *current_genome.write() = Some(create_genome_with_core_areas(
        "host".to_string(),
        "host".to_string(),
    ));
    let mut connectome_impl = ConnectomeServiceImpl::new(Arc::clone(&manager), current_genome);
    connectome_impl.set_genome_load_signals(genome_load_counter, genome_load_timestamp);
    let connectome_service: Arc<dyn ConnectomeService + Send + Sync> = Arc::new(connectome_impl);
    let genome_service: Arc<dyn GenomeService + Send + Sync> = genome_service_impl;

    let root_id = new_region_id();
    let host_region_id = new_region_id();
    for (id, name, parent) in [
        (&root_id, "root", None),
        (&host_region_id, "host_circuit", Some(root_id.clone())),
    ] {
        connectome_service
            .create_brain_region(CreateBrainRegionParams {
                region_id: id.clone(),
                name: name.to_string(),
                region_type: "Custom".to_string(),
                parent_id: parent,
                properties: None,
            })
            .await
            .expect("host region");
    }
    let host_ids = [host_area_id(b'h'), host_area_id(b'i')];
    genome_service
        .create_cortical_areas(vec![
            host_area_params(&host_ids[0], "host_a", &host_region_id),
            host_area_params(&host_ids[1], "host_b", &host_region_id),
        ])
        .await
        .expect("host areas");
    genome_service
        .create_modulator("shared".to_string(), reward_write())
        .await
        .expect("host modulator");

    let (genome_transition_lock, genome_transition_in_progress) =
        ApiState::init_genome_transition_controls();
    let state = ApiState {
        network_connection_info_provider: None,
        agent_service: None,
        analytics_service: Arc::new(AnalyticsServiceImpl::new(Arc::clone(&manager), None)),
        connectome_service,
        genome_service,
        neuron_service: Arc::new(NeuronServiceImpl::new(Arc::clone(&manager))),
        runtime_service: Arc::new(UnusedRuntimeService),
        system_service: Arc::new(SystemServiceImpl::new(
            Arc::clone(&manager),
            None,
            feagi_services::types::VersionInfo::default(),
        )),
        feagi_session_timestamp: 0,
        filesystem_data_root: ApiState::filesystem_data_root_from_config(std::path::Path::new("")),
        memory_stats_cache: None,
        amalgamation_state: ApiState::init_amalgamation_state(),
        genome_transition_lock,
        genome_transition_in_progress,
        last_failed_mutation: ApiState::init_last_failed_mutation(),
        change_ledger: None,
        agent_handler: None,
    };
    TestBrain {
        state,
        root_id,
        host_area_ids: host_ids.iter().map(CorticalID::as_base_64).collect(),
    }
}

impl TestBrain {
    /// Queue `guest` as the pending amalgamation; returns the amalgamation id.
    fn queue(&self, guest: &RuntimeGenome, source: AmalgamationSource) -> String {
        let amalgamation_id = new_region_id();
        let genome_json = feagi_evolutionary::save_genome_to_json(guest).expect("guest json");
        self.state.amalgamation_state.write().pending = Some(AmalgamationPending {
            summary: AmalgamationPendingSummary {
                amalgamation_id: amalgamation_id.clone(),
                genome_title: GUEST_TITLE.to_string(),
                circuit_size: [1, 1, 1],
            },
            source,
            genome_json,
        });
        amalgamation_id
    }

    /// Queue `guest` and confirm it under the root region at the origin.
    async fn augment(
        &self,
        guest: &RuntimeGenome,
        source: AmalgamationSource,
    ) -> (String, Result<HandlerResponse, ApiError>) {
        let id = self.queue(guest, source);
        let params = HashMap::from([
            ("amalgamation_id".to_string(), id.clone()),
            ("circuit_origin_x".to_string(), "0".to_string()),
            ("circuit_origin_y".to_string(), "0".to_string()),
            ("circuit_origin_z".to_string(), "0".to_string()),
        ]);
        let body = HashMap::from([("brain_region_id".to_string(), json!(self.root_id))]);
        let result =
            post_amalgamation_destination(State(self.state.clone()), Query(params), Json(body))
                .await
                .map(|json| json.0);
        (id, result)
    }

    fn pending_cleared(&self) -> bool {
        self.state.amalgamation_state.read().pending.is_none()
    }

    fn last_history_status(&self) -> Option<String> {
        self.state
            .amalgamation_state
            .read()
            .history
            .last()
            .map(|entry| entry.status.clone())
    }

    async fn snapshot(&self) -> HostSnapshot {
        let connectome = &self.state.connectome_service;
        let regions = connectome.list_brain_regions().await.expect("regions");
        let live_areas: BTreeSet<String> = connectome
            .get_cortical_area_ids()
            .await
            .expect("areas")
            .into_iter()
            .collect();
        HostSnapshot {
            areas: live_areas.clone(),
            regions: regions.iter().map(|r| r.region_id.clone()).collect(),
            region_members: regions
                .iter()
                .map(|r| {
                    let mut areas = r.cortical_areas.clone();
                    areas.sort();
                    (r.region_id.clone(), areas)
                })
                .collect(),
            modulators: self
                .state
                .genome_service
                .list_modulators()
                .await
                .expect("modulators")
                .as_object()
                .map(|o| o.keys().cloned().collect())
                .unwrap_or_default(),
            morphologies: connectome
                .get_morphologies()
                .await
                .expect("morphologies")
                .keys()
                .cloned()
                .collect(),
            classifiers: connectome
                .list_classifiers()
                .await
                .expect("classifiers")
                .into_iter()
                .map(|c| c.classifier_id)
                .collect(),
        }
    }

    async fn region_by_name(&self, name: &str) -> feagi_services::types::BrainRegionInfo {
        self.state
            .connectome_service
            .list_brain_regions()
            .await
            .expect("regions")
            .into_iter()
            .find(|r| r.name == name)
            .unwrap_or_else(|| panic!("region '{name}' not found"))
    }

    /// Id of the area named `name` (guest Custom/Memory ids are always remapped on import).
    async fn area_id(&self, name: &str) -> String {
        self.state
            .connectome_service
            .list_cortical_areas()
            .await
            .expect("areas")
            .into_iter()
            .find(|a| a.name == name)
            .unwrap_or_else(|| panic!("area '{name}' not found"))
            .cortical_id
    }

    async fn modulator_names(&self) -> BTreeSet<String> {
        self.snapshot().await.modulators
    }
}

#[derive(Debug, PartialEq, Eq)]
struct HostSnapshot {
    areas: BTreeSet<String>,
    regions: BTreeSet<String>,
    region_members: std::collections::BTreeMap<String, Vec<String>>,
    modulators: BTreeSet<String>,
    morphologies: BTreeSet<String>,
    classifiers: BTreeSet<String>,
}

// ---------------------------------------------------------------------------------------------
// Guest builder
// ---------------------------------------------------------------------------------------------

fn function_morphology() -> Morphology {
    Morphology {
        morphology_type: MorphologyType::Functions,
        parameters: MorphologyParameters::Functions {},
        class: "core".to_string(),
    }
}

fn empty_guest() -> RuntimeGenome {
    let mut morphologies = MorphologyRegistry::new();
    morphologies.add_morphology("projector".to_string(), function_morphology());
    RuntimeGenome {
        metadata: GenomeMetadata {
            genome_id: "guest".to_string(),
            genome_title: GUEST_TITLE.to_string(),
            genome_description: String::new(),
            version: "3.0".to_string(),
            timestamp: 0.0,
            brain_regions_root: None,
        },
        cortical_areas: HashMap::new(),
        brain_regions: HashMap::new(),
        classifiers: HashMap::new(),
        morphologies,
        modulators: ModulatorRegistry::new(),
        physiology: PhysiologyConfig::default(),
        signatures: GenomeSignatures {
            genome: String::new(),
            blueprint: String::new(),
            physiology: String::new(),
            morphologies: None,
        },
        stats: GenomeStats::default(),
        change_history: Vec::new(),
    }
}

fn guest_area(id: CorticalID, name: &str, kind: CorticalAreaType, x: i32) -> CorticalArea {
    let mut area = CorticalArea::new(
        id,
        0,
        name.to_string(),
        CorticalAreaDimensions::new(1, 1, 1).expect("dimensions"),
        GenomeCoordinate3D::new(x, 0, 0),
        kind,
    )
    .expect("guest area");
    if matches!(kind, CorticalAreaType::Memory(_)) {
        area.properties
            .insert("is_mem_type".to_string(), json!(true));
        area.properties
            .insert("temporal_depth".to_string(), json!(1));
    }
    area
}

fn custom_kind() -> CorticalAreaType {
    CorticalAreaType::Custom(CustomCorticalType::LeakyIntegrateFire)
}

fn memory_kind() -> CorticalAreaType {
    CorticalAreaType::Memory(MemoryCorticalType::Memory)
}

fn add_area(genome: &mut RuntimeGenome, area: CorticalArea) {
    genome.cortical_areas.insert(area.cortical_id, area);
}

/// Add one mapping rule `src -> dst` (stored on the source area as `cortical_mapping_dst`).
fn add_mapping(
    genome: &mut RuntimeGenome,
    src: &CorticalID,
    dst: &CorticalID,
    morphology: &str,
    modulators: &[&str],
) {
    if !genome.morphologies.contains(morphology) {
        genome
            .morphologies
            .add_morphology(morphology.to_string(), function_morphology());
    }
    let area = genome.cortical_areas.get_mut(src).expect("source area");
    let entry = area
        .properties
        .entry("cortical_mapping_dst".to_string())
        .or_insert_with(|| json!({}));
    let mut rule = json!({ "morphology_id": morphology });
    if !modulators.is_empty() {
        rule["modulators"] = json!(modulators);
    }
    entry[dst.as_base_64()] = json!([rule]);
}

/// A guest region with the given parent (None for the guest root) and member areas.
fn add_region(
    genome: &mut RuntimeGenome,
    name: &str,
    parent: Option<&str>,
    areas: &[&CorticalID],
) -> String {
    let id = new_region_id();
    let mut properties = HashMap::new();
    if let Some(parent) = parent {
        properties.insert("parent_region_id".to_string(), json!(parent));
    }
    let region = BrainRegion::new(
        RegionID::from_string(&id).expect("region id"),
        name.to_string(),
        RegionType::Undefined,
    )
    .expect("region")
    .with_areas(areas.iter().map(|a| **a))
    .with_properties(properties);
    genome.brain_regions.insert(id.clone(), region);
    id
}

fn add_guest_root(genome: &mut RuntimeGenome, areas: &[&CorticalID]) -> String {
    let id = new_region_id();
    let region = BrainRegion::new(
        RegionID::from_string(&id).expect("root id"),
        ROOT_BRAIN_REGION_NAME.to_string(),
        RegionType::Undefined,
    )
    .expect("root")
    .with_areas(areas.iter().map(|a| **a));
    genome.brain_regions.insert(id.clone(), region);
    genome.metadata.brain_regions_root = Some(id.clone());
    id
}

/// Guest-owned modulator `name` of `kind` with driver serial 0, plus an area `src` that
/// subscribes to it, with rules `src -> dst` and `src -> driver` carrying the modulator.
fn add_owned_modulator(
    genome: &mut RuntimeGenome,
    name: &str,
    kind: ModulatorKind,
    src: &CorticalID,
    dst: &CorticalID,
    subscribe_area: bool,
) {
    let driver = CorticalID::modulator_driver(0);
    add_area(
        genome,
        guest_area(driver, name, CorticalAreaType::Modulator, 0),
    );
    genome.modulators.insert(
        name.to_string(),
        ModulatorInstance {
            kind,
            magnitude_percent: 100.0,
            effect_duration_bursts: 1,
            rest_bursts: 0,
            graded: false,
            full_scale_potential: None,
            driver_cortical_id: driver,
        },
    );
    add_mapping(genome, src, dst, "projector", &[name]);
    add_mapping(genome, src, &driver, "projector", &[name]);
    if subscribe_area {
        genome
            .cortical_areas
            .get_mut(src)
            .expect("subscriber")
            .properties
            .insert("modulators".to_string(), json!([name]));
    }
}

fn count(response: &HandlerResponse, key: &str) -> u64 {
    response
        .get(key)
        .and_then(Value::as_u64)
        .unwrap_or_else(|| panic!("response lacks numeric '{key}': {response:?}"))
}

// ---------------------------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------------------------

/// (1) Nested guest regions and a memory area: the tree is recreated under the amalgamation
/// region and every area lands in its mapped region.
#[tokio::test]
async fn nested_regions_with_memory_area_are_recreated() {
    let brain = build_host(1_000_000).await;
    let top_area = cortical_id(b'c', b'a');
    let left_area = cortical_id(b'c', b'b');
    let deep_memory = cortical_id(b'm', b'c');
    let mut guest = empty_guest();
    add_area(
        &mut guest,
        guest_area(top_area, "top_area", custom_kind(), 0),
    );
    add_area(
        &mut guest,
        guest_area(left_area, "left_area", custom_kind(), 4),
    );
    add_area(
        &mut guest,
        guest_area(deep_memory, "deep_memory", memory_kind(), 8),
    );
    add_mapping(&mut guest, &top_area, &left_area, "projector", &[]);
    let root = add_guest_root(&mut guest, &[]);
    let top = add_region(&mut guest, "top", Some(&root), &[&top_area]);
    add_region(&mut guest, "left", Some(&top), &[&left_area]);
    let right = add_region(&mut guest, "right", Some(&top), &[]);
    add_region(&mut guest, "deep", Some(&right), &[&deep_memory]);

    let before = brain.snapshot().await;
    let (amalgamation_id, result) = brain
        .augment(&guest, AmalgamationSource::ExternalGenome)
        .await;
    let response = result.expect("augment succeeds");

    assert_eq!(count(&response, "created_sub_region_count"), 3);
    assert_eq!(count(&response, "imported_new_area_count"), 3);
    assert_eq!(count(&response, "imported_cortical_mappings"), 1);
    assert_eq!(count(&response, "skipped_cortical_mappings"), 0);
    assert!(brain.pending_cleared());
    assert_eq!(brain.last_history_status().as_deref(), Some("confirmed"));

    let amalgamation = brain
        .state
        .connectome_service
        .get_brain_region(&amalgamation_id)
        .await
        .unwrap();
    assert_eq!(
        amalgamation.parent_id.as_deref(),
        Some(brain.root_id.as_str())
    );
    assert_eq!(amalgamation.name, GUEST_TITLE);
    // The single top-level guest region maps onto the amalgamation region itself.
    let top_id = brain.area_id("top_area").await;
    let left_id = brain.area_id("left_area").await;
    let deep_id = brain.area_id("deep_memory").await;
    assert!(
        amalgamation.cortical_areas.contains(&top_id),
        "{amalgamation:?}"
    );

    let left = brain.region_by_name("left").await;
    let right = brain.region_by_name("right").await;
    let deep = brain.region_by_name("deep").await;
    assert_eq!(left.parent_id.as_deref(), Some(amalgamation_id.as_str()));
    assert_eq!(right.parent_id.as_deref(), Some(amalgamation_id.as_str()));
    assert_eq!(deep.parent_id.as_deref(), Some(right.region_id.as_str()));
    assert_eq!(left.cortical_areas, vec![left_id.clone()]);
    assert_eq!(deep.cortical_areas, vec![deep_id.clone()]);

    let memory = brain
        .state
        .connectome_service
        .get_cortical_area(&deep_id)
        .await
        .unwrap();
    assert_eq!(memory.cortical_type, "memory");

    // Nothing was placed in the host root by the import.
    let after = brain.snapshot().await;
    assert_eq!(
        after.region_members[&brain.root_id],
        before.region_members[&brain.root_id]
    );
    // The host is otherwise unchanged.
    assert!(before.areas.is_subset(&after.areas));
    assert_eq!(before.modulators, after.modulators);
}

/// (2) A guest modulator whose name collides with a host modulator is renamed
/// `name (Title)`; every reference follows and no guest mapping is dropped.
#[tokio::test]
async fn colliding_modulator_is_renamed_and_no_mapping_is_dropped() {
    let brain = build_host(1_000_000).await;
    let src = cortical_id(b'c', b'a');
    let dst = cortical_id(b'c', b'b');
    let mut guest = empty_guest();
    add_area(&mut guest, guest_area(src, "src", custom_kind(), 0));
    add_area(&mut guest, guest_area(dst, "dst", custom_kind(), 4));
    add_owned_modulator(
        &mut guest,
        "shared",
        ModulatorKind::Reward,
        &src,
        &dst,
        false,
    );
    add_region(&mut guest, "block", None, &[&src, &dst]);

    let (_, result) = brain
        .augment(&guest, AmalgamationSource::ExternalGenome)
        .await;
    let response = result.expect("augment succeeds");

    let renamed = format!("shared ({GUEST_TITLE})");
    let notices = response["modulator_notices"].as_array().expect("notices");
    assert!(
        notices
            .iter()
            .any(|n| n.as_str().is_some_and(|s| s.contains(&renamed))),
        "{notices:?}"
    );
    assert_eq!(count(&response, "imported_modulator_count"), 1);
    // src -> dst and src -> driver: both land.
    assert_eq!(count(&response, "imported_cortical_mappings"), 2);
    assert_eq!(count(&response, "skipped_cortical_mappings"), 0);
    assert_eq!(
        brain.modulator_names().await,
        BTreeSet::from(["shared".to_string(), renamed.clone()])
    );
    let area = brain
        .state
        .connectome_service
        .get_cortical_area(&brain.area_id("src").await)
        .await
        .unwrap();
    let mapping = area.properties["cortical_mapping_dst"].to_string();
    assert!(mapping.contains(&renamed), "{mapping}");
}

/// (3) A same-brain clone reuses the host modulator of the same name (no new modulator, no
/// rename) and still maps into the host driver.
#[tokio::test]
async fn same_brain_clone_reuses_host_modulator() {
    let brain = build_host(1_000_000).await;
    let src = cortical_id(b'c', b'a');
    let dst = cortical_id(b'c', b'b');
    let mut guest = empty_guest();
    add_area(&mut guest, guest_area(src, "src", custom_kind(), 0));
    add_area(&mut guest, guest_area(dst, "dst", custom_kind(), 4));
    add_owned_modulator(
        &mut guest,
        "shared",
        ModulatorKind::Reward,
        &src,
        &dst,
        false,
    );
    add_region(&mut guest, "block", None, &[&src, &dst]);
    let host_driver = brain
        .state
        .genome_service
        .get_modulator("shared")
        .await
        .unwrap()["driver_cortical_id"]
        .as_str()
        .unwrap()
        .to_string();

    let (_, result) = brain
        .augment(&guest, AmalgamationSource::SameBrainClone)
        .await;
    let response = result.expect("augment succeeds");

    assert_eq!(count(&response, "imported_modulator_count"), 0);
    assert_eq!(count(&response, "skipped_cortical_mappings"), 0);
    assert_eq!(count(&response, "imported_cortical_mappings"), 2);
    assert_eq!(
        brain.modulator_names().await,
        BTreeSet::from(["shared".to_string()])
    );
    let notices = response["modulator_notices"].as_array().unwrap();
    assert!(
        notices.iter().any(|n| n
            .as_str()
            .is_some_and(|s| s.starts_with("Reused existing modulator"))),
        "{notices:?}"
    );
    let area = brain
        .state
        .connectome_service
        .get_cortical_area(&brain.area_id("src").await)
        .await
        .unwrap();
    assert!(
        area.properties["cortical_mapping_dst"]
            .get(&host_driver)
            .is_some(),
        "src must map into the host driver {host_driver}: {}",
        area.properties["cortical_mapping_dst"]
    );
}

fn classifier_guest(sample_input: CorticalID, class_input: CorticalID) -> RuntimeGenome {
    let sample_memory = cortical_id(b'm', b's');
    let class_memory = cortical_id(b'm', b'k');
    let error = cortical_id(b'c', b'e');
    let correct = cortical_id(b'c', b'g');
    let class_output = cortical_id(b'c', b'o');
    let mut guest = empty_guest();
    add_area(
        &mut guest,
        guest_area(sample_input, "sample_in", custom_kind(), 0),
    );
    add_area(
        &mut guest,
        guest_area(class_input, "class_in", custom_kind(), 4),
    );
    for (id, name, kind, role) in [
        (
            sample_memory,
            "sample_memory",
            memory_kind(),
            "sample_memory",
        ),
        (class_memory, "class_memory", memory_kind(), "class_memory"),
        (error, "error", custom_kind(), "error"),
        (correct, "correct", custom_kind(), "correct"),
        (class_output, "class_out", custom_kind(), "class_output"),
    ] {
        let mut area = guest_area(id, name, kind, 8);
        area.properties
            .insert("classifier_assembly".to_string(), json!(true));
        area.properties
            .insert("classifier_role".to_string(), json!(role));
        add_area(&mut guest, area);
    }
    add_mapping(
        &mut guest,
        &sample_input,
        &sample_memory,
        "classifier_sample",
        &[],
    );
    add_mapping(
        &mut guest,
        &class_input,
        &class_memory,
        "classifier_class",
        &[],
    );
    add_mapping(
        &mut guest,
        &sample_memory,
        &class_memory,
        "associative_memory",
        &[],
    );
    // An associative mapping must carry its plasticity keys.
    let rule = &mut guest
        .cortical_areas
        .get_mut(&sample_memory)
        .expect("sample memory")
        .properties
        .get_mut("cortical_mapping_dst")
        .expect("dst map")[class_memory.as_base_64()][0];
    for (key, value) in [
        ("plasticity_constant", json!(1.0)),
        ("ltp_multiplier", json!(1.0)),
        ("ltd_multiplier", json!(1.0)),
        ("plasticity_window", json!(10)),
    ] {
        rule[key] = value;
    }
    let region = add_region(
        &mut guest,
        "block",
        None,
        &[
            &sample_input,
            &class_input,
            &sample_memory,
            &class_memory,
            &error,
            &correct,
            &class_output,
        ],
    );
    let classifier_id = new_region_id();
    guest.classifiers.insert(
        classifier_id.clone(),
        Classifier {
            classifier_id,
            name: "guest_classifier".to_string(),
            parent_region_id: region,
            coordinates_3d: [0, 0, 0],
            sample_area_ids: vec![sample_input.as_base_64()],
            class_bindings: vec![ClassifierClassBinding {
                class_area_id: class_input.as_base_64(),
                class_output_area_id: class_output.as_base_64(),
            }],
            sample_memory_id: sample_memory.as_base_64(),
            class_memory_id: class_memory.as_base_64(),
            error_area_id: error.as_base_64(),
            correct_area_id: correct.as_base_64(),
            reward_training: false,
            answer_feedback_area_id: None,
            reward_modulator_ids: Vec::new(),
            answer_latency_bursts: 0,
            learn_area_id: None,
            confidence_area_id: None,
            properties: HashMap::new(),
        },
    );
    guest
}

/// (4) A guest classifier gets a fresh classifier id and remapped areas, and the
/// `classifier_sample` / `classifier_class` mapping rules are accepted although the classifier
/// record does not exist yet (the handler upserts classifiers last).
#[tokio::test]
async fn classifier_is_imported_with_fresh_id_and_remapped_areas() {
    let brain = build_host(1_000_000).await;
    let guest = classifier_guest(cortical_id(b'c', b'a'), cortical_id(b'c', b'b'));
    let guest_classifier = guest.classifiers.values().next().unwrap().clone();

    let (amalgamation_id, result) = brain
        .augment(&guest, AmalgamationSource::ExternalGenome)
        .await;
    let response = result.expect("augment succeeds");

    // Every guest Custom/Memory area (7) is given a fresh id.
    assert_eq!(count(&response, "guest_custom_memory_id_remap_count"), 7);
    assert_eq!(count(&response, "imported_classifier_count"), 1);
    let classifiers = brain
        .state
        .connectome_service
        .list_classifiers()
        .await
        .unwrap();
    assert_eq!(classifiers.len(), 1);
    let imported = &classifiers[0];
    assert_ne!(imported.classifier_id, guest_classifier.classifier_id);
    assert_eq!(imported.parent_region_id, amalgamation_id);
    assert_ne!(imported.sample_memory_id, guest_classifier.sample_memory_id);
    assert_ne!(imported.class_memory_id, guest_classifier.class_memory_id);
    assert_ne!(imported.sample_area_ids, guest_classifier.sample_area_ids);
    assert_eq!(
        imported.sample_area_ids,
        vec![brain.area_id("sample_in").await]
    );
    assert_eq!(
        imported.sample_memory_id,
        brain.area_id("sample_memory").await
    );
    assert_eq!(
        imported.class_memory_id,
        brain.area_id("class_memory").await
    );
    assert_eq!(imported.error_area_id, brain.area_id("error").await);
    assert_eq!(
        imported.class_bindings,
        vec![ClassifierClassBinding {
            class_area_id: brain.area_id("class_in").await,
            class_output_area_id: brain.area_id("class_out").await,
        }]
    );
    // The host's own areas are untouched.
    for id in &brain.host_area_ids {
        assert!(brain
            .state
            .connectome_service
            .cortical_area_exists(id)
            .await
            .unwrap());
    }
    // Finding (4): update_cortical_mapping accepted the classifier_sample / classifier_class
    // rules before the classifier existed: all three guest rules were imported, none skipped.
    assert_eq!(count(&response, "imported_cortical_mappings"), 3);
    assert_eq!(count(&response, "skipped_cortical_mappings"), 0);
}

fn colliding_modulator_guest() -> RuntimeGenome {
    // Guest `shared` is a leak modulator; the host `shared` is a reward modulator.
    let src = cortical_id(b'c', b'a');
    let dst = cortical_id(b'c', b'b');
    let mut guest = empty_guest();
    add_area(&mut guest, guest_area(src, "src", custom_kind(), 0));
    add_area(&mut guest, guest_area(dst, "dst", custom_kind(), 4));
    add_owned_modulator(&mut guest, "shared", ModulatorKind::Leak, &src, &dst, true);
    add_region(&mut guest, "block", None, &[&src, &dst]);
    guest
}

/// (5a) A same-brain clone whose modulator has another type than the host one cannot reuse it.
/// The plan fails before anything is created: pending is cleared, the host is unchanged.
#[tokio::test]
async fn type_mismatched_reuse_fails_cleanly() {
    let brain = build_host(1_000_000).await;
    let before = brain.snapshot().await;
    let (_, result) = brain
        .augment(
            &colliding_modulator_guest(),
            AmalgamationSource::SameBrainClone,
        )
        .await;
    let error = result.expect_err("type mismatch must fail");
    assert!(
        error.message.contains("cannot be reused"),
        "{}",
        error.message
    );
    assert!(brain.pending_cleared());
    assert_eq!(brain.last_history_status().as_deref(), Some("cancelled"));
    assert_eq!(brain.snapshot().await, before);
}

/// (5b) A failure after the import started undoes everything created before it: the
/// amalgamation region, its sub-regions, the owned modulator and the areas. Pending is cleared
/// and the host is exactly as before. The failure is a guest area that subscribes to a synaptic
/// modulator, which the service rejects only after regions, modulators and areas exist.
#[tokio::test]
async fn failure_mid_import_rolls_everything_back() {
    let brain = build_host(1_000_000).await;
    let src = cortical_id(b'c', b'a');
    let dst = cortical_id(b'c', b'b');
    let mut guest = empty_guest();
    add_area(&mut guest, guest_area(src, "src", custom_kind(), 0));
    add_area(&mut guest, guest_area(dst, "dst", custom_kind(), 4));
    add_owned_modulator(&mut guest, "boost", ModulatorKind::Reward, &src, &dst, true);
    let root = add_guest_root(&mut guest, &[]);
    let top = add_region(&mut guest, "top", Some(&root), &[&src]);
    add_region(&mut guest, "inner", Some(&top), &[&dst]);

    let before = brain.snapshot().await;
    let (_, result) = brain
        .augment(&guest, AmalgamationSource::ExternalGenome)
        .await;
    let error = result.expect_err("import must fail");
    assert!(error.message.contains("subscribe"), "{}", error.message);
    assert!(
        !error.message.contains("Rollback could not remove"),
        "{}",
        error.message
    );
    assert!(brain.pending_cleared());
    assert_eq!(brain.last_history_status().as_deref(), Some("cancelled"));
    assert_eq!(brain.snapshot().await, before, "host must be unchanged");
}

/// (6) A guest with Custom and Memory areas listed in its root region: they go into the
/// amalgamation region (never the host root), the memory twin resolves, and nothing is
/// "recovered" (the stray-area repair only runs on genome load and connectome apply).
#[tokio::test]
async fn guest_root_custom_memory_areas_land_in_circuit_and_twin_resolves() {
    let brain = build_host(1_000_000).await;
    let upstream = cortical_id(b'c', b'a');
    let memory = cortical_id(b'm', b'b');
    let mut guest = empty_guest();
    add_area(
        &mut guest,
        guest_area(upstream, "upstream", custom_kind(), 0),
    );
    add_area(&mut guest, guest_area(memory, "memory", memory_kind(), 4));
    add_mapping(&mut guest, &upstream, &memory, "episodic_memory", &[]);
    add_guest_root(&mut guest, &[&upstream, &memory]);

    let (amalgamation_id, result) = brain
        .augment(&guest, AmalgamationSource::ExternalGenome)
        .await;
    let response = result.expect("augment succeeds");

    assert_eq!(count(&response, "created_sub_region_count"), 0);
    assert_eq!(count(&response, "imported_cortical_mappings"), 1);
    let circuit = brain
        .state
        .connectome_service
        .get_brain_region(&amalgamation_id)
        .await
        .unwrap();
    let upstream_id = brain.area_id("upstream").await;
    let memory_id = brain.area_id("memory").await;
    assert!(circuit.cortical_areas.contains(&upstream_id));
    assert!(circuit.cortical_areas.contains(&memory_id));
    let host_root = brain
        .state
        .connectome_service
        .get_brain_region(&brain.root_id)
        .await
        .unwrap();
    for id in [&upstream_id, &memory_id] {
        assert!(
            !host_root.cortical_areas.contains(id),
            "{id} joined the host root"
        );
    }
    let regions = brain
        .state
        .connectome_service
        .list_brain_regions()
        .await
        .unwrap();
    assert!(
        regions.iter().all(|r| r.name != "Recovered Areas"),
        "augment must not need the stray repair"
    );

    let memory_info = brain
        .state
        .connectome_service
        .get_cortical_area(&memory_id)
        .await
        .unwrap();
    let twin_id = memory_info.properties["memory_twin_areas"][upstream_id.as_str()]
        .as_str()
        .expect("twin is indexed on the memory area")
        .to_string();
    assert!(brain
        .state
        .connectome_service
        .cortical_area_exists(&twin_id)
        .await
        .unwrap());
    let twin_home = regions
        .iter()
        .find(|r| r.cortical_areas.contains(&twin_id))
        .expect("twin belongs to a region");
    assert_ne!(twin_home.region_id, brain.root_id);
}

/// Probe: deleting a modulator (as the rollback does) must not leave its driver listed in the
/// host root region.
#[tokio::test]
async fn deleted_modulator_driver_leaves_no_stale_root_member() {
    let brain = build_host(1_000_000).await;
    brain
        .state
        .genome_service
        .create_modulator("scratch".to_string(), reward_write())
        .await
        .unwrap();
    brain
        .state
        .genome_service
        .delete_modulator("scratch", true)
        .await
        .unwrap();
    let root = brain
        .state
        .connectome_service
        .get_brain_region(&brain.root_id)
        .await
        .unwrap();
    let live: BTreeSet<String> = brain
        .state
        .connectome_service
        .get_cortical_area_ids()
        .await
        .unwrap()
        .into_iter()
        .collect();
    let stale: Vec<_> = root
        .cortical_areas
        .iter()
        .filter(|id| !live.contains(*id))
        .collect();
    assert!(stale.is_empty(), "stale root members {stale:?}");
}
