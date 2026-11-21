// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! HTTP-level tests for the brain-region `port_encoding` property: it is returned by region
//! reads, derived from wiring, and set or cleared as an author entry through
//! `PUT /v1/region/region`. Requests go through the real router (`create_http_server`) over real
//! services. Fixtures are neutral.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use feagi_api::transports::http::server::{create_http_server, ApiState};
use feagi_brain_development::ConnectomeManager;
use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::{DynamicNPU, RustNPU, TracingMutex};
use feagi_npu_runtime::StdRuntime;
use feagi_services::impls::{
    AnalyticsServiceImpl, ConnectomeServiceImpl, GenomeServiceImpl, NeuronServiceImpl,
    SystemServiceImpl,
};
use feagi_services::{ConnectomeService, GenomeService};
use feagi_structures::genomic::brain_regions::{BrainRegion, RegionID, RegionType};
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use feagi_structures::genomic::cortical_area::{
    CorticalArea, CorticalAreaDimensions, CorticalAreaType, CorticalID, CustomCorticalType,
    IOCorticalAreaConfigurationFlag,
};
use parking_lot::RwLock;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tower::ServiceExt;

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

const SENSOR_FLAG: IOCorticalAreaConfigurationFlag = IOCorticalAreaConfigurationFlag::Percentage(
    FrameChangeHandling::Absolute,
    PercentageNeuronPositioning::Linear,
);

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

struct Brain {
    state: ApiState,
    circuit: String,
    ipu: CorticalID,
    port: CorticalID,
}

/// A root region holding one sensory area and a circuit region holding one custom area.
fn build_brain() -> Brain {
    let npu = Arc::new(TracingMutex::new(
        DynamicNPU::F32(
            RustNPU::new(StdRuntime, CPUBackend::new(), 10_000, 10_000, 10).expect("npu"),
        ),
        "region-port-encoding-http-test-npu",
    ));
    let mut connectome = ConnectomeManager::new_for_testing_with_npu(Arc::clone(&npu));
    connectome.setup_core_morphologies_for_testing();
    let ipu = SENSOR_FLAG.as_io_cortical_id(true, *b"hts", 0u16.into(), 0u8.into());
    let port = CorticalID::try_from_bytes(b"chttport").unwrap();
    connectome
        .add_cortical_area(area(
            ipu,
            "sensor",
            CorticalAreaType::BrainInput(SENSOR_FLAG),
        ))
        .unwrap();
    connectome
        .add_cortical_area(area(
            port,
            "port",
            CorticalAreaType::Custom(CustomCorticalType::LeakyIntegrateFire),
        ))
        .unwrap();
    let root_id = RegionID::new();
    let circuit_id = RegionID::new();
    let circuit = circuit_id.to_string();
    connectome
        .add_brain_region(
            BrainRegion::new(root_id, "Root".to_string(), RegionType::Undefined)
                .unwrap()
                .with_areas([ipu]),
            None,
        )
        .unwrap();
    connectome
        .add_brain_region(
            BrainRegion::new(circuit_id, "Circuit".to_string(), RegionType::Undefined)
                .unwrap()
                .with_areas([port]),
            Some(root_id.to_string()),
        )
        .unwrap();

    let mut genome = feagi_evolutionary::templates::create_genome_with_core_areas(
        "g".to_string(),
        "g".to_string(),
    );
    for id in [ipu, port] {
        genome
            .cortical_areas
            .insert(id, connectome.get_cortical_area(&id).unwrap().clone());
    }
    for region_id in connectome.get_brain_region_ids() {
        genome.brain_regions.insert(
            region_id.to_string(),
            connectome.get_brain_region(region_id).unwrap().clone(),
        );
    }

    let manager = Arc::new(RwLock::new(connectome));
    let genome_service_impl = Arc::new(GenomeServiceImpl::new(Arc::clone(&manager)));
    let current_genome = genome_service_impl.get_current_genome_arc();
    let genome_load_counter = genome_service_impl.get_genome_load_counter_arc();
    let genome_load_timestamp = genome_service_impl.get_genome_load_timestamp_arc();
    *current_genome.write() = Some(genome);
    let mut connectome_impl = ConnectomeServiceImpl::new(Arc::clone(&manager), current_genome);
    connectome_impl.set_genome_load_signals(genome_load_counter, genome_load_timestamp);
    let connectome_service: Arc<dyn ConnectomeService + Send + Sync> = Arc::new(connectome_impl);
    let genome_service: Arc<dyn GenomeService + Send + Sync> = genome_service_impl;
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
    Brain {
        state,
        circuit,
        ipu,
        port,
    }
}

async fn call(state: &ApiState, method: &str, path: &str, body: Value) -> (StatusCode, Value) {
    let request = Request::builder()
        .uri(path)
        .method(method)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    let response = create_http_server(state.clone())
        .oneshot(request)
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

impl Brain {
    async fn put_region(&self, body: Value) -> (StatusCode, Value) {
        let mut body = body;
        body["region_id"] = json!(self.circuit);
        call(&self.state, "PUT", "/v1/region/region", body).await
    }

    /// The circuit's `port_encoding` as `GET /v1/region/regions_members` returns it.
    async fn members_port_encoding(&self) -> Value {
        let (status, body) = call(
            &self.state,
            "GET",
            "/v1/region/regions_members",
            Value::Null,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        body[&self.circuit]["port_encoding"].clone()
    }
}

#[tokio::test]
async fn region_reads_return_port_encoding_set_clear_and_validation() {
    let brain = build_brain();
    let port = brain.port.as_base_64();

    // No designation yet: reads return an empty object, not a missing key.
    assert_eq!(brain.members_port_encoding().await, json!({}));

    let (status, body) = brain
        .put_region(json!({ "designated_inputs": [port.clone()] }))
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // Wiring an IPU to the designated port: the read shows the derived entry.
    brain
        .state
        .connectome_service
        .update_cortical_mapping(
            brain.ipu.as_base_64(),
            port.clone(),
            vec![json!({ "morphology_id": "projector" })],
        )
        .await
        .unwrap();
    let derived = json!({
        port.clone(): {
            "kind": "Percentage", "signed": false, "frame": "Absolute",
            "positioning": "Linear", "source": "wiring"
        }
    });
    assert_eq!(brain.members_port_encoding().await, derived);

    // Set an author entry.
    let (status, body) = brain
        .put_region(json!({ "port_encoding": {
            port.clone(): { "kind": "Boolean", "signed": false }
        }}))
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        brain.members_port_encoding().await,
        json!({ port.clone(): {
            "kind": "Boolean", "signed": false, "frame": null,
            "positioning": null, "source": "author"
        }})
    );

    // The single-region read carries it too.
    let (status, body) = call(
        &brain.state,
        "GET",
        &format!("/v1/region/region/{}", brain.circuit),
        Value::Null,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["port_encoding"][&port]["source"], "author");

    // Clear: derivation resumes.
    let (status, body) = brain
        .put_region(json!({ "port_encoding": { port.clone(): null } }))
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(brain.members_port_encoding().await, derived);
}

#[tokio::test]
async fn invalid_port_encoding_requests_are_refused_and_change_nothing() {
    let brain = build_brain();
    let port = brain.port.as_base_64();
    let (status, _) = brain
        .put_region(json!({ "designated_inputs": [port.clone()] }))
        .await;
    assert_eq!(status, StatusCode::OK);

    for bad in [
        json!({ port.clone(): { "kind": "Wobbly" } }),
        json!({ port.clone(): { "frame": "Sideways" } }),
        json!({ port.clone(): { "positioning": "Diagonal" } }),
        json!({ port.clone(): { "signed": "yes" } }),
        json!({ port.clone(): { "source": "wiring" } }),
        json!({ port.clone(): { "colour": "red" } }),
        json!({ "not base64!": { "kind": "Boolean" } }),
    ] {
        let (status, body) = brain
            .put_region(json!({ "port_encoding": bad.clone() }))
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{bad}: {body}");
    }

    // An area that is not a designated port of the region.
    let (status, body) = brain
        .put_region(json!({ "port_encoding": {
            brain.ipu.as_base_64(): { "kind": "Boolean" }
        }}))
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");

    assert_eq!(brain.members_port_encoding().await, json!({}));
}

#[tokio::test]
async fn undesignating_the_port_drops_its_entry_from_reads() {
    let brain = build_brain();
    let port = brain.port.as_base_64();
    let (status, _) = brain
        .put_region(json!({
            "designated_inputs": [port.clone()],
            "port_encoding": { port.clone(): { "kind": "Misc", "frame": "Incremental" } }
        }))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(brain.members_port_encoding().await[&port]["kind"], "Misc");

    let (status, _) = brain.put_region(json!({ "designated_inputs": [] })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(brain.members_port_encoding().await, json!({}));
}
