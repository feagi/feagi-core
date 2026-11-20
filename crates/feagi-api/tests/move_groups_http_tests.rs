// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! HTTP-level tests for `POST /v1/cortical_area/move_groups` and the group moves inside
//! `PUT /v1/cortical_area/multi/cortical_area`. Requests go through the real router
//! (`create_http_server`) over real services and a live NPU. Fixtures are neutral.

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
use feagi_structures::genomic::cortical_area::descriptors::CorticalUnitIndex;
use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use feagi_structures::genomic::cortical_area::{
    CorticalArea, CorticalAreaDimensions, CorticalAreaType, CorticalID,
    IOCorticalAreaConfigurationFlag,
};
use feagi_structures::genomic::SensoryCorticalUnit;
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

fn servo_area_id(group: u16) -> CorticalID {
    SensoryCorticalUnit::get_cortical_ids_array_for_servo_with_parameters(
        FrameChangeHandling::Incremental,
        PercentageNeuronPositioning::Linear,
        CorticalUnitIndex::from(group),
    )[0]
}

fn io_area(id: CorticalID, name: &str) -> CorticalArea {
    CorticalArea::new(
        id,
        0,
        name.to_string(),
        CorticalAreaDimensions::new(1, 1, 1).unwrap(),
        (0, 0, 0).into(),
        CorticalAreaType::BrainInput(IOCorticalAreaConfigurationFlag::Boolean),
    )
    .unwrap()
}

struct Brain {
    state: ApiState,
    manager: Arc<RwLock<ConnectomeManager>>,
}

impl Brain {
    fn has(&self, id: &CorticalID) -> bool {
        self.manager.read().get_cortical_area(id).is_some()
    }
}

/// A brain with one servo input area per listed group, in the connectome and the genome store.
fn build_brain(groups: &[u16], agent_handler: Option<ApiHandler>) -> Brain {
    let npu = Arc::new(TracingMutex::new(
        DynamicNPU::F32(
            RustNPU::new(StdRuntime, CPUBackend::new(), 10_000, 10_000, 10).expect("npu"),
        ),
        "move-groups-http-test-npu",
    ));
    let mut connectome = ConnectomeManager::new_for_testing_with_npu(Arc::clone(&npu));
    connectome.setup_core_morphologies_for_testing();
    let mut genome = feagi_evolutionary::templates::create_genome_with_core_areas(
        "g".to_string(),
        "g".to_string(),
    );
    for group in groups {
        let id = servo_area_id(*group);
        let area = io_area(id, &format!("servo_{group}"));
        connectome.add_cortical_area(area.clone()).unwrap();
        genome.cortical_areas.insert(id, area);
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
        agent_handler,
    };
    Brain { state, manager }
}

type ApiHandler = Arc<std::sync::Mutex<feagi_agent::server::FeagiAgentHandler>>;

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

#[tokio::test]
async fn move_groups_returns_200_with_moved_pairs() {
    let brain = build_brain(&[60, 61], None);
    let (status, body) = call(
        &brain.state,
        "POST",
        "/v1/cortical_area/move_groups",
        json!({ "moves": [{ "cortical_id": servo_area_id(60).as_base_64(), "group_id": 70 }] }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let moved = body["moved"].as_array().expect("moved list");
    assert_eq!(moved.len(), 1);
    assert_eq!(moved[0]["old_id"], servo_area_id(60).as_base_64());
    assert_eq!(moved[0]["new_id"], servo_area_id(70).as_base_64());
    assert!(!brain.has(&servo_area_id(60)));
    assert!(brain.has(&servo_area_id(70)));
    assert!(brain.has(&servo_area_id(61)));
}

#[tokio::test]
async fn move_groups_returns_409_for_an_occupied_target_and_changes_nothing() {
    let brain = build_brain(&[60, 61], None);
    let (status, body) = call(
        &brain.state,
        "POST",
        "/v1/cortical_area/move_groups",
        json!({ "moves": [{ "cortical_id": servo_area_id(60).as_base_64(), "group_id": 61 }] }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert!(brain.has(&servo_area_id(60)));
    assert!(brain.has(&servo_area_id(61)));
}

#[tokio::test]
async fn move_groups_returns_400_for_a_malformed_body() {
    let brain = build_brain(&[60], None);
    // Wrong shape: `group_id` is not a number.
    let (status, _) = call(
        &brain.state,
        "POST",
        "/v1/cortical_area/move_groups",
        json!({ "moves": [{ "cortical_id": servo_area_id(60).as_base_64(), "group_id": "x" }] }),
    )
    .await;
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "malformed body gave {status}"
    );
    // Right shape, empty move list: refused by the handler itself.
    let (status, _) = call(
        &brain.state,
        "POST",
        "/v1/cortical_area/move_groups",
        json!({ "moves": [] }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(brain.has(&servo_area_id(60)));
}

#[tokio::test]
async fn multi_put_with_one_bad_group_change_changes_no_area() {
    let brain = build_brain(&[60, 61, 62], None);
    let a = servo_area_id(60).as_base_64();
    let b = servo_area_id(61).as_base_64();
    // A -> 70 is fine; B -> 62 is taken. The batch is all or nothing.
    let (status, body) = call(
        &brain.state,
        "PUT",
        "/v1/cortical_area/multi/cortical_area",
        json!({
            "cortical_id_list": [a, b],
            a.clone(): { "group_id": 70 },
            b.clone(): { "group_id": 62 }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    for group in [60, 61, 62] {
        assert!(brain.has(&servo_area_id(group)), "group {group} lost");
    }
    assert!(!brain.has(&servo_area_id(70)));
}

#[tokio::test]
async fn multi_put_applies_a_good_group_batch_then_the_other_updates_under_the_new_id() {
    let brain = build_brain(&[60, 61], None);
    let a = servo_area_id(60).as_base_64();
    let b = servo_area_id(61).as_base_64();
    let (status, body) = call(
        &brain.state,
        "PUT",
        "/v1/cortical_area/multi/cortical_area",
        json!({
            "cortical_id_list": [a, b],
            a.clone(): { "group_id": 70, "cortical_name": "moved_a" },
            b.clone(): { "group_id": 71 }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(!brain.has(&servo_area_id(60)) && !brain.has(&servo_area_id(61)));
    let manager = brain.manager.read();
    assert_eq!(
        manager.get_cortical_area(&servo_area_id(70)).unwrap().name,
        "moved_a"
    );
    assert!(manager.get_cortical_area(&servo_area_id(71)).is_some());
}

// ---------------------------------------------------------------------------------------------
// Ownership: an area another connected agent feeds is refused (409); the owner may move it.
// ---------------------------------------------------------------------------------------------

fn servo_registration(group: u16) -> Value {
    json!({
        "input_units_and_encoder_properties": {
            "Servo": [[
                {
                    "cortical_unit_index": group,
                    "io_configuration_flags": {
                        "frame_change_handling": "Incremental",
                        "percentage_neuron_positioning": "Linear"
                    },
                    "device_grouping": [
                        { "friendly_name": "a", "device_properties": { "bundle_id": "b" } }
                    ]
                },
                {}
            ]]
        }
    })
}

/// Register one agent over a real websocket and store its device registrations. Returns the
/// handler and the agent's session id (base64).
fn handler_with_registered_agent(group: u16) -> (ApiHandler, String) {
    use feagi_agent::clients::{AgentRegistrationStatus, CommandControlAgent};
    use feagi_agent::server::auth::DummyAuth;
    use feagi_agent::server::{AgentLivenessConfig, FeagiAgentHandler};
    use feagi_agent::{AgentCapabilities, AgentDescriptor, AuthToken};
    use feagi_io::protocol_implementations::websocket::websocket_std::FeagiWebSocketClientRequesterProperties;
    use feagi_io::protocol_implementations::websocket::websocket_std::{
        FeagiWebSocketServerPublisherProperties, FeagiWebSocketServerRouterProperties,
    };
    use std::time::{Duration, Instant};

    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let bind = format!("127.0.0.1:{port}");
    let remote = format!("ws://127.0.0.1:{port}");
    let mut handler = FeagiAgentHandler::new_with_liveness_config(
        Box::new(DummyAuth {}),
        AgentLivenessConfig::default(),
    );
    handler
        .add_and_start_command_control_server(Box::new(
            FeagiWebSocketServerRouterProperties::new_with_remote(&bind, &remote).unwrap(),
        ))
        .expect("router");
    let viz_port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    handler.add_publisher_server(Box::new(
        FeagiWebSocketServerPublisherProperties::new(
            &format!("127.0.0.1:{viz_port}"),
            &format!("ws://127.0.0.1:{viz_port}"),
        )
        .unwrap(),
    ));
    let (tx, rx) = std::sync::mpsc::channel();
    let client = std::thread::spawn(move || {
        let result: Result<feagi_io::AgentID, String> = (|| {
            let mut client = CommandControlAgent::new(Box::new(
                FeagiWebSocketClientRequesterProperties::new(&remote).map_err(|e| e.to_string())?,
            ));
            client.request_connect().map_err(|e| e.to_string())?;
            client
                .request_registration(
                    AgentDescriptor::new("neuraville", "ownership-test", 1)
                        .map_err(|e| e.to_string())?,
                    AuthToken::new([0u8; 32]),
                    vec![AgentCapabilities::ReceiveNeuronVisualizations],
                )
                .map_err(|e| e.to_string())?;
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                client.poll_for_messages().map_err(|e| e.to_string())?;
                if let AgentRegistrationStatus::Registered(id, _) = client.registration_status() {
                    return Ok(*id);
                }
                if Instant::now() >= deadline {
                    return Err("registration timed out".to_string());
                }
                std::thread::sleep(Duration::from_millis(2));
            }
        })();
        let _ = tx.send(result);
    });
    let deadline = Instant::now() + Duration::from_secs(10);
    let agent_id = loop {
        handler.poll_command_and_control().expect("poll");
        if let Ok(result) = rx.try_recv() {
            break result.expect("client registration");
        }
        assert!(
            Instant::now() < deadline,
            "server never saw the registration"
        );
        std::thread::sleep(Duration::from_millis(2));
    };
    client.join().unwrap();
    handler.set_device_registrations_by_agent(agent_id, servo_registration(group));
    (
        Arc::new(std::sync::Mutex::new(handler)),
        agent_id.to_base64(),
    )
}

#[tokio::test]
async fn move_groups_returns_409_for_an_area_another_agent_uses_and_lets_the_owner_move_it() {
    let (handler, owner_session) = handler_with_registered_agent(60);
    let brain = build_brain(&[60], Some(handler));
    let body =
        json!({ "moves": [{ "cortical_id": servo_area_id(60).as_base_64(), "group_id": 70 }] });

    // Anonymous requester: another agent uses the area.
    let (status, response) = call(
        &brain.state,
        "POST",
        "/v1/cortical_area/move_groups",
        body.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{response}");
    assert!(brain.has(&servo_area_id(60)));

    // A different requester is refused the same way.
    let mut other = body.clone();
    other["requester_agent_id"] = json!("someone-else");
    let (status, _) = call(&brain.state, "POST", "/v1/cortical_area/move_groups", other).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(brain.has(&servo_area_id(60)));

    // The owner moves its own area.
    let mut owner = body;
    owner["requester_agent_id"] = json!(owner_session);
    let (status, response) =
        call(&brain.state, "POST", "/v1/cortical_area/move_groups", owner).await;
    assert_eq!(status, StatusCode::OK, "{response}");
    assert!(brain.has(&servo_area_id(70)));
}

#[tokio::test]
async fn single_put_group_change_is_refused_for_another_agents_area_and_allowed_for_the_owner() {
    let (handler, owner_session) = handler_with_registered_agent(60);
    let brain = build_brain(&[60, 61], Some(handler));
    let id = servo_area_id(60).as_base_64();
    let path = "/v1/cortical_area/cortical_area";

    let (status, body) = call(
        &brain.state,
        "PUT",
        path,
        json!({ "cortical_id": id, "group_id": 70 }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    let (status, _) = call(
        &brain.state,
        "PUT",
        path,
        json!({ "cortical_id": id, "group_id": 70, "requester_agent_id": "someone-else" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(brain.has(&servo_area_id(60)));
    assert!(!brain.has(&servo_area_id(70)));

    // An area no connected agent feeds moves freely.
    let free = servo_area_id(61).as_base_64();
    let (status, body) = call(
        &brain.state,
        "PUT",
        path,
        json!({ "cortical_id": free, "group_id": 71 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(brain.has(&servo_area_id(71)));

    let (status, body) = call(
        &brain.state,
        "PUT",
        path,
        json!({ "cortical_id": id, "group_id": 70, "requester_agent_id": owner_session }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(brain.has(&servo_area_id(70)));
}

#[tokio::test]
async fn multi_put_group_change_is_refused_for_another_agents_area_and_allowed_for_the_owner() {
    let (handler, owner_session) = handler_with_registered_agent(60);
    let brain = build_brain(&[60, 61], Some(handler));
    let owned = servo_area_id(60).as_base_64();
    let free = servo_area_id(61).as_base_64();
    let path = "/v1/cortical_area/multi/cortical_area";

    // One refused area fails the whole batch: the free area does not move either.
    let (status, body) = call(
        &brain.state,
        "PUT",
        path,
        json!({
            "cortical_id_list": [owned, free],
            owned.clone(): { "group_id": 70 },
            free.clone(): { "group_id": 71 }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert!(brain.has(&servo_area_id(60)) && brain.has(&servo_area_id(61)));
    assert!(!brain.has(&servo_area_id(70)) && !brain.has(&servo_area_id(71)));

    // Only the free area: no connected agent feeds it.
    let (status, body) = call(
        &brain.state,
        "PUT",
        path,
        json!({ "cortical_id_list": [free], free.clone(): { "group_id": 71 } }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(brain.has(&servo_area_id(71)));

    let (status, body) = call(
        &brain.state,
        "PUT",
        path,
        json!({
            "cortical_id_list": [owned],
            "requester_agent_id": owner_session,
            owned.clone(): { "group_id": 70 }
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(brain.has(&servo_area_id(70)));
}
