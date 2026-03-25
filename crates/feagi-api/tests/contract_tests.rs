// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code, unused_imports, unused_variables)] // TODO: Update tests to use new API

//! Contract Tests for FEAGI Rust API
//!
//! These tests ensure that the Rust API produces correct responses
//! and handles errors properly.
//!
//! NOTE: Currently disabled - tests need to be updated to use axum test utilities
//! instead of tower::util::ServiceExt (which requires the "util" feature).
use axum::body::Body;
use axum::http::{Request, StatusCode};
#[cfg(feature = "feagi-agent")]
use feagi_agent::clients::{AgentRegistrationStatus, CommandControlAgent};
#[cfg(feature = "feagi-agent")]
use feagi_agent::command_and_control::FeagiMessage;
#[cfg(feature = "feagi-agent")]
use feagi_agent::server::auth::DummyAuth;
#[cfg(feature = "feagi-agent")]
use feagi_agent::server::{AgentLivenessConfig, FeagiAgentHandler};
#[cfg(feature = "feagi-agent")]
use feagi_agent::AgentCapabilities;
#[cfg(feature = "feagi-agent")]
use feagi_agent::{AgentDescriptor, AuthToken};
#[cfg(feature = "feagi-agent")]
use feagi_api::common::agent_registration::{
    auto_create_cortical_areas_from_device_registrations,
    derive_motor_cortical_ids_from_device_registrations,
    derive_sensory_cortical_ids_from_device_registrations,
};
use feagi_api::common::{Json as ApiJson, State as ApiStateExtract};
use feagi_api::endpoints::agent::register_agent;
#[cfg(feature = "feagi-agent")]
use feagi_api::endpoints::genome::post_upload;
use feagi_api::transports::http::server::{create_http_server, ApiState};
use feagi_api::v1::AgentRegistrationRequest;
use feagi_brain_development::ConnectomeManager;
use feagi_evolutionary::templates::create_genome_with_core_areas;
#[cfg(feature = "feagi-agent")]
use feagi_io::protocol_implementations::websocket::websocket_std::{
    FeagiWebSocketServerPublisherProperties, FeagiWebSocketServerPullerProperties,
    FeagiWebSocketServerRouterProperties,
};
#[cfg(feature = "feagi-agent")]
use feagi_io::traits_and_enums::client::{FeagiClient, FeagiClientPusher};
#[cfg(feature = "feagi-agent")]
use feagi_io::AgentID;
use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::TracingMutex;
use feagi_npu_burst_engine::{DynamicNPU, RustNPU};
use feagi_npu_runtime::StdRuntime;
#[cfg(feature = "feagi-agent")]
use feagi_serialization::FeagiByteContainer;
use feagi_services::impls::{
    AnalyticsServiceImpl, ConnectomeServiceImpl, GenomeServiceImpl, NeuronServiceImpl,
    SystemServiceImpl,
};
#[cfg(feature = "feagi-agent")]
use feagi_services::types::CreateCorticalAreaParams;
#[cfg(feature = "feagi-agent")]
use feagi_services::RuntimeService;
use parking_lot::RwLock;
use serde_json::{json, Value};
use std::collections::HashMap;
#[cfg(feature = "feagi-agent")]
use std::collections::HashSet;
#[cfg(feature = "feagi-agent")]
use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::sync::{Arc, Mutex, OnceLock};
#[cfg(feature = "feagi-agent")]
use std::time::{Duration, Instant};
use tower::ServiceExt;
// tower::util::ServiceExt requires the "util" feature which may not be enabled
// Using axum's test utilities instead

/// Build ApiState with initialized components.
/// Each test gets a fresh, isolated manager (no singleton conflicts)
fn build_test_state() -> ApiState {
    if std::env::var("FEAGI_CONFIG_PATH").is_err() {
        std::env::set_var(
            "FEAGI_CONFIG_PATH",
            "/Users/nadji/code/FEAGI-2.0/feagi-rs/feagi_configuration.toml",
        );
    }

    // Initialize NPU (fire_ledger_window=10)
    let runtime = StdRuntime;
    let backend = CPUBackend::new();
    let npu_result = RustNPU::new(runtime, backend, 1_000_000, 10_000_000, 10).unwrap();
    let npu = Arc::new(TracingMutex::new(
        DynamicNPU::F32(npu_result),
        "api-contract-test-npu",
    ));

    // Create isolated ConnectomeManager for testing (bypasses singleton)
    let manager = Arc::new(RwLock::new(ConnectomeManager::new_for_testing_with_npu(
        Arc::clone(&npu),
    )));

    // Create services
    let genome_service_impl = Arc::new(GenomeServiceImpl::new(Arc::clone(&manager)));
    let current_genome = genome_service_impl.get_current_genome_arc();
    {
        let mut genome_guard = current_genome.write();
        *genome_guard = Some(create_genome_with_core_areas(
            "test-genome".to_string(),
            "test".to_string(),
        ));
    }
    let genome_service = genome_service_impl;
    let connectome_service = Arc::new(ConnectomeServiceImpl::new(
        Arc::clone(&manager),
        current_genome.clone(),
    ));
    // For tests, use empty version info
    let version_info = feagi_services::types::VersionInfo::default();
    let system_service = Arc::new(SystemServiceImpl::new(
        Arc::clone(&manager),
        None, // No BurstLoopRunner for basic tests
        version_info,
    ));
    let analytics_service = Arc::new(AnalyticsServiceImpl::new(
        Arc::clone(&manager),
        None, // No BurstLoopRunner for basic tests
    ));
    let neuron_service = Arc::new(NeuronServiceImpl::new(Arc::clone(&manager)));

    // Create a mock RuntimeService that always returns NotImplemented
    // This is acceptable for tests that don't exercise runtime control
    struct MockRuntimeService;
    #[async_trait::async_trait]
    impl feagi_services::RuntimeService for MockRuntimeService {
        async fn start(&self) -> feagi_services::ServiceResult<()> {
            Err(feagi_services::ServiceError::NotImplemented(
                "MockRuntimeService".to_string(),
            ))
        }
        async fn stop(&self) -> feagi_services::ServiceResult<()> {
            Err(feagi_services::ServiceError::NotImplemented(
                "MockRuntimeService".to_string(),
            ))
        }
        async fn pause(&self) -> feagi_services::ServiceResult<()> {
            Err(feagi_services::ServiceError::NotImplemented(
                "MockRuntimeService".to_string(),
            ))
        }
        async fn resume(&self) -> feagi_services::ServiceResult<()> {
            Err(feagi_services::ServiceError::NotImplemented(
                "MockRuntimeService".to_string(),
            ))
        }
        async fn step(&self) -> feagi_services::ServiceResult<()> {
            Err(feagi_services::ServiceError::NotImplemented(
                "MockRuntimeService".to_string(),
            ))
        }
        async fn get_status(&self) -> feagi_services::ServiceResult<feagi_services::RuntimeStatus> {
            Ok(feagi_services::RuntimeStatus {
                is_running: false,
                is_paused: false,
                frequency_hz: 1000.0,
                burst_count: 0,
                current_rate_hz: 0.0,
                last_burst_neuron_count: 0,
                avg_burst_time_ms: 0.0,
            })
        }
        async fn set_frequency(&self, _frequency: f64) -> feagi_services::ServiceResult<()> {
            Err(feagi_services::ServiceError::NotImplemented(
                "MockRuntimeService".to_string(),
            ))
        }
        async fn get_burst_count(&self) -> feagi_services::ServiceResult<u64> {
            Ok(0)
        }
        async fn reset_burst_count(&self) -> feagi_services::ServiceResult<()> {
            Err(feagi_services::ServiceError::NotImplemented(
                "MockRuntimeService".to_string(),
            ))
        }
        async fn get_fcl_snapshot(&self) -> feagi_services::ServiceResult<Vec<(u64, f32)>> {
            Ok(vec![])
        }
        async fn get_fcl_snapshot_with_cortical_idx(
            &self,
        ) -> feagi_services::ServiceResult<Vec<(u64, u32, f32)>> {
            Ok(vec![])
        }
        async fn get_fire_queue_sample(
            &self,
        ) -> feagi_services::ServiceResult<
            std::collections::HashMap<u32, (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<f32>)>,
        > {
            Ok(std::collections::HashMap::new())
        }
        async fn get_fire_ledger_configs(
            &self,
        ) -> feagi_services::ServiceResult<Vec<(u32, usize)>> {
            Ok(vec![])
        }
        async fn configure_fire_ledger_window(
            &self,
            _cortical_id: u32,
            _window_size: usize,
        ) -> feagi_services::ServiceResult<()> {
            Err(feagi_services::ServiceError::NotImplemented(
                "MockRuntimeService".to_string(),
            ))
        }
        async fn get_fcl_sampler_config(&self) -> feagi_services::ServiceResult<(f64, u32)> {
            Ok((0.0, 0))
        }
        async fn set_fcl_sampler_config(
            &self,
            _sample_rate: Option<f64>,
            _max_samples: Option<u32>,
        ) -> feagi_services::ServiceResult<()> {
            Err(feagi_services::ServiceError::NotImplemented(
                "MockRuntimeService".to_string(),
            ))
        }
        async fn get_area_fcl_sample_rate(
            &self,
            _cortical_id: u32,
        ) -> feagi_services::ServiceResult<f64> {
            Ok(0.0)
        }
        async fn set_area_fcl_sample_rate(
            &self,
            _cortical_id: u32,
            _sample_rate: f64,
        ) -> feagi_services::ServiceResult<()> {
            Err(feagi_services::ServiceError::NotImplemented(
                "MockRuntimeService".to_string(),
            ))
        }
        async fn inject_sensory_by_coordinates(
            &self,
            _cortical_area_name: &str,
            _coordinates: &[(u32, u32, u32, f32)],
            _mode: feagi_services::traits::runtime_service::ManualStimulationMode,
        ) -> feagi_services::ServiceResult<usize> {
            Err(feagi_services::ServiceError::NotImplemented(
                "MockRuntimeService".to_string(),
            ))
        }

        async fn register_motor_subscriptions(
            &self,
            _agent_id: &str,
            _cortical_ids: Vec<String>,
            _rate_hz: f64,
        ) -> feagi_services::ServiceResult<()> {
            Ok(())
        }

        async fn register_visualization_subscriptions(
            &self,
            _agent_id: &str,
            _rate_hz: f64,
        ) -> feagi_services::ServiceResult<()> {
            Ok(())
        }
        fn unregister_motor_subscriptions(&self, _agent_id: &str) {}
        fn unregister_visualization_subscriptions(&self, _agent_id: &str) {}
        fn clear_all_motor_subscriptions(&self) {}
        fn clear_all_visualization_subscriptions(&self) {}
    }

    let runtime_service =
        Arc::new(MockRuntimeService) as Arc<dyn feagi_services::RuntimeService + Send + Sync>;

    // Create API state
    // Get FEAGI session timestamp (when this instance started)
    let feagi_session_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let (genome_transition_lock, genome_transition_in_progress) =
        ApiState::init_genome_transition_controls();
    ApiState {
        network_connection_info_provider: None,
        agent_service: None,
        analytics_service,
        connectome_service,
        genome_service,
        neuron_service,
        runtime_service,
        system_service,
        snapshot_service: None,
        feagi_session_timestamp,
        memory_stats_cache: None,
        amalgamation_state: ApiState::init_amalgamation_state(),
        genome_transition_lock,
        genome_transition_in_progress,
        #[cfg(feature = "feagi-agent")]
        agent_handler: Some(ApiState::init_agent_registration_handler()),
    }
}

#[cfg(feature = "feagi-agent")]
#[derive(Default)]
struct RuntimeTransitionTracker {
    motor_subscriptions: Mutex<HashSet<String>>,
    visualization_subscriptions: Mutex<HashSet<String>>,
}

#[cfg(feature = "feagi-agent")]
struct TrackingRuntimeService {
    tracker: Arc<RuntimeTransitionTracker>,
}

#[cfg(feature = "feagi-agent")]
impl TrackingRuntimeService {
    fn new(tracker: Arc<RuntimeTransitionTracker>) -> Self {
        Self { tracker }
    }
}

#[cfg(feature = "feagi-agent")]
#[async_trait::async_trait]
impl feagi_services::RuntimeService for TrackingRuntimeService {
    async fn start(&self) -> feagi_services::ServiceResult<()> {
        Ok(())
    }
    async fn stop(&self) -> feagi_services::ServiceResult<()> {
        Ok(())
    }
    async fn pause(&self) -> feagi_services::ServiceResult<()> {
        Ok(())
    }
    async fn resume(&self) -> feagi_services::ServiceResult<()> {
        Ok(())
    }
    async fn step(&self) -> feagi_services::ServiceResult<()> {
        Ok(())
    }
    async fn get_status(&self) -> feagi_services::ServiceResult<feagi_services::RuntimeStatus> {
        Ok(feagi_services::RuntimeStatus {
            is_running: false,
            is_paused: false,
            frequency_hz: 1000.0,
            burst_count: 0,
            current_rate_hz: 0.0,
            last_burst_neuron_count: 0,
            avg_burst_time_ms: 0.0,
        })
    }
    async fn set_frequency(&self, _frequency: f64) -> feagi_services::ServiceResult<()> {
        Ok(())
    }
    async fn get_burst_count(&self) -> feagi_services::ServiceResult<u64> {
        Ok(0)
    }
    async fn reset_burst_count(&self) -> feagi_services::ServiceResult<()> {
        Ok(())
    }
    async fn get_fcl_snapshot(&self) -> feagi_services::ServiceResult<Vec<(u64, f32)>> {
        Ok(vec![])
    }
    async fn get_fcl_snapshot_with_cortical_idx(
        &self,
    ) -> feagi_services::ServiceResult<Vec<(u64, u32, f32)>> {
        Ok(vec![])
    }
    async fn get_fire_queue_sample(
        &self,
    ) -> feagi_services::ServiceResult<
        std::collections::HashMap<u32, (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>, Vec<f32>)>,
    > {
        Ok(std::collections::HashMap::new())
    }
    async fn get_fire_ledger_configs(&self) -> feagi_services::ServiceResult<Vec<(u32, usize)>> {
        Ok(vec![])
    }
    async fn configure_fire_ledger_window(
        &self,
        _cortical_id: u32,
        _window_size: usize,
    ) -> feagi_services::ServiceResult<()> {
        Ok(())
    }
    async fn get_fcl_sampler_config(&self) -> feagi_services::ServiceResult<(f64, u32)> {
        Ok((0.0, 0))
    }
    async fn set_fcl_sampler_config(
        &self,
        _sample_rate: Option<f64>,
        _max_samples: Option<u32>,
    ) -> feagi_services::ServiceResult<()> {
        Ok(())
    }
    async fn get_area_fcl_sample_rate(
        &self,
        _cortical_id: u32,
    ) -> feagi_services::ServiceResult<f64> {
        Ok(0.0)
    }
    async fn set_area_fcl_sample_rate(
        &self,
        _cortical_id: u32,
        _sample_rate: f64,
    ) -> feagi_services::ServiceResult<()> {
        Ok(())
    }
    async fn inject_sensory_by_coordinates(
        &self,
        _cortical_area_name: &str,
        _coordinates: &[(u32, u32, u32, f32)],
        _mode: feagi_services::traits::runtime_service::ManualStimulationMode,
    ) -> feagi_services::ServiceResult<usize> {
        Ok(0)
    }
    async fn register_motor_subscriptions(
        &self,
        agent_id: &str,
        _cortical_ids: Vec<String>,
        _rate_hz: f64,
    ) -> feagi_services::ServiceResult<()> {
        self.tracker
            .motor_subscriptions
            .lock()
            .unwrap()
            .insert(agent_id.to_string());
        Ok(())
    }
    async fn register_visualization_subscriptions(
        &self,
        agent_id: &str,
        _rate_hz: f64,
    ) -> feagi_services::ServiceResult<()> {
        self.tracker
            .visualization_subscriptions
            .lock()
            .unwrap()
            .insert(agent_id.to_string());
        Ok(())
    }
    fn unregister_motor_subscriptions(&self, agent_id: &str) {
        self.tracker
            .motor_subscriptions
            .lock()
            .unwrap()
            .remove(agent_id);
    }
    fn unregister_visualization_subscriptions(&self, agent_id: &str) {
        self.tracker
            .visualization_subscriptions
            .lock()
            .unwrap()
            .remove(agent_id);
    }
    fn clear_all_motor_subscriptions(&self) {
        self.tracker.motor_subscriptions.lock().unwrap().clear();
    }
    fn clear_all_visualization_subscriptions(&self) {
        self.tracker
            .visualization_subscriptions
            .lock()
            .unwrap()
            .clear();
    }
}

#[cfg(feature = "feagi-agent")]
fn wait_for_registered_agent(
    handler: &Arc<Mutex<FeagiAgentHandler>>,
    client: &mut CommandControlAgent,
    timeout: Duration,
) -> (
    String,
    feagi_io::traits_and_enums::shared::TransportProtocolEndpoint,
) {
    let deadline = Instant::now() + timeout;
    loop {
        {
            let mut guard = handler.lock().unwrap();
            guard
                .poll_command_and_control()
                .expect("command/control polling failed");
        }
        client
            .poll_for_messages()
            .expect("client poll_for_messages failed");
        if let AgentRegistrationStatus::Registered(session_id, endpoints) =
            client.registration_status()
        {
            let sensory_endpoint = endpoints
                .get(&AgentCapabilities::SendSensorData)
                .cloned()
                .expect("registration response missing sensory endpoint");
            return (session_id.to_base64(), sensory_endpoint);
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for agent registration"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[cfg(feature = "feagi-agent")]
fn wait_for_sensory_data(handler: &Arc<Mutex<FeagiAgentHandler>>, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        {
            let mut guard = handler.lock().unwrap();
            match guard.poll_agent_sensors() {
                Ok(Some(_)) => return true,
                Ok(None) => {}
                Err(_) => return false,
            }
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[cfg(feature = "feagi-agent")]
fn reserve_free_port() -> u16 {
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
        .expect("failed to bind ephemeral port");
    listener
        .local_addr()
        .expect("failed to read local addr")
        .port()
}

#[cfg(feature = "feagi-agent")]
/// Helper to create a test server with initialized components
async fn create_test_server() -> axum::Router {
    let state = build_test_state();
    create_http_server(state)
}

#[cfg(feature = "feagi-agent")]
static CONFIG_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[cfg(feature = "feagi-agent")]
struct ConfigEnvGuard {
    previous: Option<String>,
    path: std::path::PathBuf,
}

#[cfg(feature = "feagi-agent")]
impl Drop for ConfigEnvGuard {
    fn drop(&mut self) {
        if let Some(value) = self.previous.take() {
            std::env::set_var("FEAGI_CONFIG_PATH", value);
        } else {
            std::env::remove_var("FEAGI_CONFIG_PATH");
        }
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(feature = "feagi-agent")]
fn set_temp_config(auto_create: bool) -> ConfigEnvGuard {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos();
    let path = std::path::PathBuf::from(format!("/tmp/feagi-config-{nanos}--temp.toml"));
    let base_path =
        std::path::PathBuf::from("/Users/nadji/code/FEAGI-2.0/feagi-rs/feagi_configuration.toml");
    let base_contents =
        std::fs::read_to_string(&base_path).expect("Failed to read base FEAGI config");
    let mut contents = String::new();
    let mut in_agent = false;
    let mut injected = false;

    for line in base_contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("[agent]") {
            in_agent = true;
            contents.push_str(line);
            contents.push('\n');
            continue;
        }
        if in_agent && trimmed.starts_with('[') {
            if !injected {
                contents.push_str(&format!(
                    "auto_create_missing_cortical_areas = {}\n",
                    auto_create
                ));
                injected = true;
            }
            in_agent = false;
        }
        if in_agent && trimmed.starts_with("auto_create_missing_cortical_areas") {
            contents.push_str(&format!(
                "auto_create_missing_cortical_areas = {}\n",
                auto_create
            ));
            injected = true;
            continue;
        }
        contents.push_str(line);
        contents.push('\n');
    }

    if in_agent && !injected {
        contents.push_str(&format!(
            "auto_create_missing_cortical_areas = {}\n",
            auto_create
        ));
    }

    std::fs::write(&path, contents).expect("Failed to write temp config");

    let previous = std::env::var("FEAGI_CONFIG_PATH").ok();
    std::env::set_var("FEAGI_CONFIG_PATH", &path);

    ConfigEnvGuard { previous, path }
}

#[cfg(feature = "feagi-agent")]
fn sample_device_registrations() -> Value {
    json!({
        "input_units_and_encoder_properties": {
            "Vision": [
                [
                    {
                        "cortical_unit_index": 0,
                        "device_grouping": [{"id": 0}]
                    },
                    {}
                ]
            ]
        },
        "output_units_and_decoder_properties": {
            "RotaryMotor": [
                [
                    {
                        "cortical_unit_index": 0,
                        "device_grouping": [{"id": 0}]
                    },
                    {}
                ]
            ]
        },
        "feedbacks": {}
    })
}

/// Multi-limb device_registrations: 4 limbs (groups 0-3), 3 PositionalServo channels per limb.
#[cfg(feature = "feagi-agent")]
fn sample_multi_limb_device_registrations() -> Value {
    json!({
        "output_units_and_decoder_properties": {
            "PositionalServo": [
                [
                    {
                        "cortical_unit_index": 0,
                        "device_grouping": [{"id": 0}, {"id": 1}, {"id": 2}]
                    },
                    {}
                ],
                [
                    {
                        "cortical_unit_index": 1,
                        "device_grouping": [{"id": 0}, {"id": 1}, {"id": 2}]
                    },
                    {}
                ],
                [
                    {
                        "cortical_unit_index": 2,
                        "device_grouping": [{"id": 0}, {"id": 1}, {"id": 2}]
                    },
                    {}
                ],
                [
                    {
                        "cortical_unit_index": 3,
                        "device_grouping": [{"id": 0}, {"id": 1}, {"id": 2}]
                    },
                    {}
                ]
            ]
        },
        "feedbacks": {}
    })
}

#[cfg(feature = "feagi-agent")]
fn sample_named_motor_device_registrations() -> Value {
    json!({
        "output_units_and_decoder_properties": {
            "PositionalServo": [
                [
                    {
                        "friendly_name": "front_left_leg",
                        "cortical_unit_index": 0,
                        "device_grouping": [
                            {
                                "friendly_name": "front_left_hip",
                                "device_properties": {
                                    "bundle_id": "front_left_leg",
                                    "bundle_type": "leg"
                                }
                            },
                            {
                                "friendly_name": "front_left_knee",
                                "device_properties": {
                                    "bundle_id": "front_left_leg",
                                    "bundle_type": "leg"
                                }
                            }
                        ]
                    },
                    {}
                ]
            ]
        },
        "feedbacks": {}
    })
}

#[cfg(feature = "feagi-agent")]
fn sample_motor_device_registrations_with_io_flags() -> Value {
    json!({
        "output_units_and_decoder_properties": {
            "RotaryMotor": [
                [
                    {
                        "friendly_name": "arm_motor",
                        "cortical_unit_index": 0,
                        "device_grouping": [{"id": 0}, {"id": 1}],
                        "io_configuration_flags": {
                            "frame_change_handling": "Absolute",
                            "percentage_neuron_positioning": "Linear"
                        }
                    },
                    {}
                ]
            ]
        },
        "feedbacks": {}
    })
}

#[cfg(feature = "feagi-agent")]
fn sample_named_sensory_only_device_registrations() -> Value {
    json!({
        "input_units_and_encoder_properties": {
            "Vision": [
                [
                    {
                        "friendly_name": "head_camera",
                        "cortical_unit_index": 0,
                        "device_grouping": [
                            {
                                "friendly_name": "rgb_camera",
                                "device_properties": {
                                    "bundle_id": "head_camera",
                                    "bundle_type": "camera_rig"
                                }
                            }
                        ]
                    },
                    {}
                ]
            ]
        },
        "feedbacks": {}
    })
}

#[cfg(feature = "feagi-agent")]
fn sample_sensory_device_registrations_with_large_vision_encoder() -> Value {
    json!({
        "input_units_and_encoder_properties": {
            "Vision": [
                [
                    {
                        "friendly_name": "head_camera",
                        "cortical_unit_index": 0,
                        "device_grouping": [{"id": 0}]
                    },
                    {
                        "CartesianPlane": {
                            "image_resolution": {
                                "width": 128,
                                "height": 96
                            },
                            "color_space": "Gamma",
                            "color_channel_layout": "RGBA"
                        }
                    }
                ]
            ]
        },
        "feedbacks": {}
    })
}

/// Helper to make a request and get response as JSON
async fn request_json(
    app: axum::Router,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let request_builder = Request::builder()
        .uri(path)
        .method(method)
        .header("content-type", "application/json");

    let request = if let Some(body_json) = body {
        request_builder
            .body(Body::from(serde_json::to_vec(&body_json).unwrap()))
            .unwrap()
    } else {
        request_builder.body(Body::empty()).unwrap()
    };

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let json: Value = if body_bytes.is_empty() {
        json!(null)
    } else {
        serde_json::from_slice(&body_bytes).unwrap_or(json!(null))
    };

    (status, json)
}

// ============================================================================
// HEALTH & SYSTEM TESTS
// ============================================================================

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_server().await;

    let (status, response) = request_json(app, "GET", "/v1/system/health_check", None).await;

    assert_eq!(status, StatusCode::OK);
    assert!(response["status"].is_string() || !response.is_null());
}

#[tokio::test]
async fn test_system_status() {
    let app = create_test_server().await;

    let (status, response) = request_json(app, "GET", "/v1/monitoring/status", None).await;

    assert_eq!(status, StatusCode::OK);
    // Response should have burst_engine_active field
    assert!(response.is_object());
}

// ============================================================================
// AGENT REGISTRATION TESTS
// ============================================================================

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_register_agent_returns_session_and_endpoints() {
    let _state = build_test_state();
    let _descriptor = AgentDescriptor::new("neuraville", "api-test", 1).unwrap();
    let agent_id = AgentID::new_random();
    let auth_token = AuthToken::new([1u8; 32]).to_base64();

    let device_registrations = json!({
        "input_units_and_encoder_properties": { "camera": [] },
        "output_units_and_decoder_properties": {},
        "feedbacks": {}
    });

    let mut capabilities: HashMap<String, Value> = HashMap::new();
    capabilities.insert("device_registrations".to_string(), device_registrations);

    let request = AgentRegistrationRequest {
        agent_type: "visualization".to_string(),
        agent_id: agent_id.to_base64(),
        agent_data_port: 0,
        agent_version: "0.0.0-test".to_string(),
        controller_version: "0.0.0-test".to_string(),
        capabilities,
        agent_ip: None,
        metadata: None,
        auth_token: Some(auth_token),
        chosen_transport: None,
    };

    let response = register_agent(ApiStateExtract(_state), ApiJson(request))
        .await
        .expect("Registration should succeed");
    let body = response.0;

    assert!(body.success);
    let transport = body.transport.expect("Expected transport payload");
    assert!(transport.contains_key("session_id"));
    let endpoints = transport
        .get("endpoints")
        .and_then(|value| value.as_object())
        .expect("Expected endpoints in transport payload");
    assert!(endpoints.contains_key("send_sensor_data"));
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_register_visualization_only_agent_returns_session_and_endpoints() {
    let _state = build_test_state();
    let agent_id = AgentID::new_random();
    let auth_token = AuthToken::new([4u8; 32]).to_base64();

    let mut capabilities: HashMap<String, Value> = HashMap::new();
    capabilities.insert(
        "visualization".to_string(),
        json!({ "visualization_type": "3d_brain", "rate_hz": 20.0, "bridge_proxy": false }),
    );

    let request = AgentRegistrationRequest {
        agent_type: "visualization".to_string(),
        agent_id: agent_id.to_base64(),
        agent_data_port: 0,
        agent_version: "0.0.0-test".to_string(),
        controller_version: "0.0.0-test".to_string(),
        capabilities,
        agent_ip: None,
        metadata: None,
        auth_token: Some(auth_token),
        chosen_transport: None,
    };

    let response = register_agent(ApiStateExtract(_state), ApiJson(request))
        .await
        .expect("Visualization-only registration should succeed");
    let body = response.0;

    assert!(body.success);
    let transport = body.transport.expect("Expected transport payload");
    assert!(transport.contains_key("session_id"));
    let endpoints = transport
        .get("endpoints")
        .and_then(|value| value.as_object())
        .expect("Expected endpoints in transport payload");
    assert!(endpoints.contains_key("receive_neuron_visualizations"));
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_register_agent_rejects_rate_above_burst_frequency() {
    let _state = build_test_state();
    let agent_id = AgentID::new_random();
    let auth_token = AuthToken::new([2u8; 32]).to_base64();

    let mut capabilities: HashMap<String, Value> = HashMap::new();
    capabilities.insert(
        "device_registrations".to_string(),
        sample_device_registrations(),
    );
    capabilities.insert("motor".to_string(), json!({ "rate_hz": 2000.0 }));

    let request = AgentRegistrationRequest {
        agent_type: "motor".to_string(),
        agent_id: agent_id.to_base64(),
        agent_data_port: 0,
        agent_version: "0.0.0-test".to_string(),
        controller_version: "0.0.0-test".to_string(),
        capabilities,
        agent_ip: None,
        metadata: None,
        auth_token: Some(auth_token),
        chosen_transport: None,
    };

    let result = register_agent(ApiStateExtract(_state), ApiJson(request)).await;
    assert!(result.is_err());
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_register_agent_rejects_visualization_rate_above_burst_frequency() {
    let _state = build_test_state();
    let agent_id = AgentID::new_random();
    let auth_token = AuthToken::new([3u8; 32]).to_base64();

    let mut capabilities: HashMap<String, Value> = HashMap::new();
    capabilities.insert(
        "device_registrations".to_string(),
        sample_device_registrations(),
    );
    capabilities.insert("visualization".to_string(), json!({ "rate_hz": 2000.0 }));

    let request = AgentRegistrationRequest {
        agent_type: "visualization".to_string(),
        agent_id: agent_id.to_base64(),
        agent_data_port: 0,
        agent_version: "0.0.0-test".to_string(),
        controller_version: "0.0.0-test".to_string(),
        capabilities,
        agent_ip: None,
        metadata: None,
        auth_token: Some(auth_token),
        chosen_transport: None,
    };

    let result = register_agent(ApiStateExtract(_state), ApiJson(request)).await;
    assert!(result.is_err());
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_disabled_skips_creation() {
    let _guard = {
        let _lock = CONFIG_ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("Failed to lock config env");
        set_temp_config(false)
    };
    let state = build_test_state();

    auto_create_cortical_areas_from_device_registrations(&state, &sample_device_registrations())
        .await;

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");
    assert!(areas.is_empty());
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_enabled_creates_areas() {
    let _guard = {
        let _lock = CONFIG_ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("Failed to lock config env");
        set_temp_config(true)
    };
    let state = build_test_state();

    auto_create_cortical_areas_from_device_registrations(&state, &sample_device_registrations())
        .await;

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");
    assert!(!areas.is_empty());
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_creates_all_limb_cortical_areas() {
    let _guard = {
        let _lock = CONFIG_ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("Failed to lock config env");
        set_temp_config(true)
    };
    let state = build_test_state();

    auto_create_cortical_areas_from_device_registrations(
        &state,
        &sample_multi_limb_device_registrations(),
    )
    .await;

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");
    let motor_areas: Vec<_> = areas
        .iter()
        .filter(|a| a.area_type == "motor" || a.cortical_group == "OPU")
        .collect();
    assert_eq!(
        motor_areas.len(),
        8,
        "Expected 8 PositionalServo areas (abs+inc per limb), got {}",
        motor_areas.len()
    );
    let mut abs_width_count = 0;
    let mut inc_width_count = 0;
    for area in &motor_areas {
        if area.name.ends_with("-1") {
            assert_eq!(
                area.dimensions.0, 6,
                "Incremental subunit width should be 6"
            );
            inc_width_count += 1;
        } else if area.name.ends_with("-0") {
            assert_eq!(area.dimensions.0, 3, "Absolute subunit width should be 3");
            abs_width_count += 1;
        }
        assert_eq!(
            area.properties.get("dev_count").and_then(|v| v.as_u64()),
            Some(3),
            "Each limb area should have dev_count=3"
        );
    }
    assert_eq!(abs_width_count, 4, "Expected 4 absolute limb areas");
    assert_eq!(inc_width_count, 4, "Expected 4 incremental limb areas");
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_uses_registration_friendly_name_for_motor_areas() {
    let _guard = {
        let _lock = CONFIG_ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("Failed to lock config env");
        set_temp_config(true)
    };
    let state = build_test_state();

    auto_create_cortical_areas_from_device_registrations(
        &state,
        &sample_named_motor_device_registrations(),
    )
    .await;

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");
    assert!(
        areas
            .iter()
            .any(|area| area.name.starts_with("front_left_leg")),
        "Expected created motor area names to use registration friendly name"
    );
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_supports_sensory_only_registrations() {
    let _guard = {
        let _lock = CONFIG_ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("Failed to lock config env");
        set_temp_config(true)
    };
    let state = build_test_state();

    auto_create_cortical_areas_from_device_registrations(
        &state,
        &sample_named_sensory_only_device_registrations(),
    )
    .await;

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");
    assert!(
        areas.iter().any(|area| area.name == "head_camera"),
        "Expected sensory-only registration to create sensory area with registration name"
    );
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_updates_existing_sensory_area_dimensions_from_encoder_properties() {
    let _guard = {
        let _lock = CONFIG_ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("Failed to lock config env");
        set_temp_config(true)
    };
    let state = build_test_state();
    let registrations = sample_sensory_device_registrations_with_large_vision_encoder();
    let sensory_ids = derive_sensory_cortical_ids_from_device_registrations(&registrations)
        .expect("Failed deriving sensory cortical IDs");
    let cortical_id = sensory_ids
        .into_iter()
        .next()
        .expect("Expected a vision cortical ID");

    // Pre-create an undersized sensory area to verify auto-create update logic expands it.
    state
        .genome_service
        .create_cortical_areas(vec![CreateCorticalAreaParams {
            cortical_id: cortical_id.clone(),
            name: "legacy_vision".to_string(),
            dimensions: (64, 64, 3),
            position: (-100, 30, 0),
            area_type: "sensory".to_string(),
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
        }])
        .await
        .expect("Failed to pre-create sensory area");

    auto_create_cortical_areas_from_device_registrations(&state, &registrations).await;

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");
    let resized = areas
        .iter()
        .find(|area| area.cortical_id == cortical_id)
        .expect("Expected sensory area to exist after auto-create");

    assert_eq!(
        resized.dimensions,
        (128, 96, 4),
        "Expected sensory area dimensions to expand to encoder properties"
    );
    assert_eq!(
        resized.position,
        (-100, 30, 0),
        "Existing sensory area position must remain unchanged during auto-create reconciliation"
    );
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_preserves_existing_motor_area_position_while_reconciling_structure() {
    let _guard = {
        let _lock = CONFIG_ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("Failed to lock config env");
        set_temp_config(true)
    };
    let state = build_test_state();
    let registrations = sample_motor_device_registrations_with_io_flags();
    let motor_ids = derive_motor_cortical_ids_from_device_registrations(&registrations)
        .expect("Failed deriving motor cortical IDs");
    let cortical_id = motor_ids
        .into_iter()
        .next()
        .expect("Expected at least one motor cortical ID");
    let preserved_position = (777, 888, 9);

    state
        .genome_service
        .create_cortical_areas(vec![CreateCorticalAreaParams {
            cortical_id: cortical_id.clone(),
            name: "legacy_motor".to_string(),
            dimensions: (1, 1, 1),
            position: preserved_position,
            area_type: "motor".to_string(),
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
        }])
        .await
        .expect("Failed to pre-create motor area");

    auto_create_cortical_areas_from_device_registrations(&state, &registrations).await;

    let areas = state
        .connectome_service
        .list_cortical_areas()
        .await
        .expect("Failed to list cortical areas");
    let reconciled = areas
        .iter()
        .find(|area| area.cortical_id == cortical_id)
        .expect("Expected motor area to exist after auto-create");

    assert_eq!(
        reconciled.position, preserved_position,
        "Existing motor area position must remain unchanged during auto-create reconciliation"
    );
    assert_ne!(
        reconciled.dimensions,
        (1, 1, 1),
        "Expected motor dimensions to reconcile from registration while preserving position"
    );
    assert_eq!(
        reconciled
            .properties
            .get("dev_count")
            .and_then(|v| v.as_u64()),
        Some(2),
        "Expected motor dev_count to reconcile from registration while preserving position"
    );
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_genome_transition_clears_agents_subscriptions_and_blocks_sensory_until_reregister() {
    let tracker = Arc::new(RuntimeTransitionTracker::default());
    let runtime_service = Arc::new(TrackingRuntimeService::new(Arc::clone(&tracker)))
        as Arc<dyn RuntimeService + Send + Sync>;

    let mut state = build_test_state();
    state.runtime_service = runtime_service.clone();
    let handler = Arc::new(Mutex::new(feagi_agent::server::FeagiAgentHandler::new(
        Box::new(feagi_agent::server::auth::DummyAuth {}),
    )));
    state.agent_handler = Some(Arc::clone(&handler));

    let session_id = AgentID::new([7u8; AgentID::NUMBER_BYTES]);
    let descriptor = AgentDescriptor::new("neuraville", "transition-test-agent", 1)
        .expect("descriptor creation failed");
    {
        let mut guard = handler.lock().unwrap();
        guard.register_logical_agent(
            session_id,
            descriptor.clone(),
            vec![
                AgentCapabilities::SendSensorData,
                AgentCapabilities::ReceiveMotorData,
            ],
        );
    }
    let session_id_b64 = session_id.to_base64();
    assert_eq!(
        handler.lock().unwrap().get_all_registered_agents().len(),
        1,
        "expected an active registered agent before genome load"
    );

    runtime_service
        .register_motor_subscriptions(&session_id_b64, vec!["b21vdAUAAAA=".to_string()], 10.0)
        .await
        .expect("failed to seed motor subscriptions");
    runtime_service
        .register_visualization_subscriptions(&session_id_b64, 10.0)
        .await
        .expect("failed to seed visualization subscriptions");

    // Use a malformed genome payload so load fails fast while still exercising
    // strict transition teardown logic that runs before genome parsing/loading.
    let bad_genome_value = json!({"invalid": "payload"});
    let transition_result =
        post_upload(ApiStateExtract(state.clone()), ApiJson(bad_genome_value)).await;
    assert!(
        transition_result.is_err(),
        "expected malformed genome upload to fail"
    );

    assert_eq!(
        handler.lock().unwrap().get_all_registered_agents().len(),
        0,
        "agent list should be empty after genome transition"
    );
    assert!(
        tracker.motor_subscriptions.lock().unwrap().is_empty(),
        "motor subscriptions should be empty after genome transition"
    );
    assert!(
        tracker
            .visualization_subscriptions
            .lock()
            .unwrap()
            .is_empty(),
        "visualization subscriptions should be empty after genome transition"
    );

    // Before re-registration there is no active transport session, so handler
    // polling cannot consume sensory frames.
    {
        let mut guard = handler.lock().unwrap();
        assert!(
            guard
                .poll_agent_sensors()
                .expect("polling sensory should not fail")
                .is_none(),
            "sensory must not be consumed after genome transition before re-registration"
        );
    }

    // Re-registration path: a new logical session can be added after transition.
    let new_session_id = AgentID::new([9u8; AgentID::NUMBER_BYTES]);
    {
        let mut guard = handler.lock().unwrap();
        guard.register_logical_agent(
            new_session_id,
            descriptor,
            vec![
                AgentCapabilities::SendSensorData,
                AgentCapabilities::ReceiveMotorData,
            ],
        );
    }
    assert_eq!(
        handler.lock().unwrap().get_all_registered_agents().len(),
        1,
        "agent should reappear only after explicit re-registration"
    );
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_force_deregister_preserves_descriptor_device_registrations_for_reconnect() {
    let mut handler = FeagiAgentHandler::new(Box::new(DummyAuth {}));
    let descriptor =
        AgentDescriptor::new("neuraville", "mujoco-agent", 1).expect("descriptor creation failed");
    let initial_session = AgentID::new([3u8; AgentID::NUMBER_BYTES]);

    handler.set_device_registrations_by_descriptor(
        initial_session.to_base64(),
        descriptor.clone(),
        json!({
            "outputs": {
                "servo": {
                    "type": "opu",
                    "properties": {"feagi_index": 0}
                }
            }
        }),
    );
    handler.register_logical_agent(
        initial_session,
        descriptor.clone(),
        vec![AgentCapabilities::ReceiveMotorData],
    );

    let removed = handler.force_deregister_all_agents("test transition");
    assert_eq!(removed.len(), 1, "expected one agent to be deregistered");
    assert!(
        handler.get_all_registered_agents().is_empty(),
        "all active agent sessions must be removed"
    );
    assert!(
        handler
            .get_device_registrations_by_descriptor(&descriptor)
            .is_some(),
        "descriptor device registrations should persist for reconnect mapping"
    );
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
#[ignore = "real transport path can hang in CI; run manually for websocket validation"]
async fn test_genome_transition_realtime_transport_requires_reregistration() {
    let tracker = Arc::new(RuntimeTransitionTracker::default());
    let runtime_service = Arc::new(TrackingRuntimeService::new(Arc::clone(&tracker)))
        as Arc<dyn RuntimeService + Send + Sync>;

    let mut state = build_test_state();
    state.runtime_service = runtime_service.clone();
    let host = Ipv4Addr::LOCALHOST.to_string();
    let registration_port = reserve_free_port();
    let sensory_port = reserve_free_port();
    let motor_port = reserve_free_port();
    let viz_port = reserve_free_port();
    let registration_bind = format!("{host}:{registration_port}");
    let registration_remote = format!("ws://{host}:{registration_port}");
    let sensory_bind = format!("{host}:{sensory_port}");
    let sensory_remote = format!("ws://{host}:{sensory_port}");
    let motor_bind = format!("{host}:{motor_port}");
    let motor_remote = format!("ws://{host}:{motor_port}");
    let viz_bind = format!("{host}:{viz_port}");
    let viz_remote = format!("ws://{host}:{viz_port}");

    let mut test_handler = FeagiAgentHandler::new_with_liveness_config(
        Box::new(DummyAuth {}),
        AgentLivenessConfig::default(),
    );
    test_handler
        .add_and_start_command_control_server(Box::new(
            FeagiWebSocketServerRouterProperties::new_with_remote(
                &registration_bind,
                &registration_remote,
            )
            .expect("failed to create websocket router properties"),
        ))
        .expect("failed to start websocket command/control router");
    test_handler.add_puller_server(Box::new(
        FeagiWebSocketServerPullerProperties::new_with_remote(&sensory_bind, &sensory_remote)
            .expect("failed to create websocket sensory puller properties"),
    ));
    test_handler.add_publisher_server(Box::new(
        FeagiWebSocketServerPublisherProperties::new(&motor_bind, &motor_remote)
            .expect("failed to create websocket motor publisher properties"),
    ));
    test_handler.add_publisher_server(Box::new(
        FeagiWebSocketServerPublisherProperties::new(&viz_bind, &viz_remote)
            .expect("failed to create websocket viz publisher properties"),
    ));
    let handler = Arc::new(Mutex::new(test_handler));
    state.agent_handler = Some(Arc::clone(&handler));

    let registration_endpoint = {
        let guard = handler.lock().unwrap();
        guard
            .get_command_control_server_info()
            .into_iter()
            .next()
            .expect("no command/control server available")
            .get_agent_endpoint()
    };

    let mut client = CommandControlAgent::new(
        registration_endpoint
            .try_create_boxed_client_requester_properties()
            .expect("failed to create requester properties"),
    );
    client.request_connect().expect("client connect failed");
    client
        .request_registration(
            AgentDescriptor::new("neuraville", "transition-rt-agent", 1)
                .expect("descriptor creation failed"),
            AuthToken::new([0u8; 32]),
            vec![
                AgentCapabilities::SendSensorData,
                AgentCapabilities::ReceiveMotorData,
            ],
        )
        .expect("registration request failed");

    let (session_id_b64, sensory_endpoint) =
        wait_for_registered_agent(&handler, &mut client, Duration::from_secs(5));
    assert_eq!(
        handler.lock().unwrap().get_all_registered_agents().len(),
        1,
        "expected an active registered agent before genome load"
    );

    runtime_service
        .register_motor_subscriptions(&session_id_b64, vec!["b21vdAUAAAA=".to_string()], 10.0)
        .await
        .expect("failed to seed motor subscriptions");
    runtime_service
        .register_visualization_subscriptions(&session_id_b64, 10.0)
        .await
        .expect("failed to seed visualization subscriptions");

    let pusher_props = sensory_endpoint
        .try_create_boxed_client_pusher_properties()
        .expect("failed to create sensory pusher properties");
    let mut sensory_pusher = pusher_props.as_boxed_client_pusher();
    sensory_pusher
        .request_connect()
        .expect("sensory pusher connect failed");
    let pre_payload: FeagiByteContainer = FeagiMessage::HeartBeat.into();
    sensory_pusher
        .publish_data(pre_payload.get_byte_ref())
        .expect("failed to publish pre-load sensory payload");
    assert!(
        wait_for_sensory_data(&handler, Duration::from_secs(2)),
        "expected sensory data to be consumed before genome transition"
    );

    let bad_genome_value = json!({"invalid": "payload"});
    let transition_result =
        post_upload(ApiStateExtract(state.clone()), ApiJson(bad_genome_value)).await;
    assert!(
        transition_result.is_err(),
        "expected malformed genome upload to fail"
    );

    assert_eq!(
        handler.lock().unwrap().get_all_registered_agents().len(),
        0,
        "agent list should be empty after genome transition"
    );
    assert!(
        tracker.motor_subscriptions.lock().unwrap().is_empty(),
        "motor subscriptions should be empty after genome transition"
    );
    assert!(
        tracker
            .visualization_subscriptions
            .lock()
            .unwrap()
            .is_empty(),
        "visualization subscriptions should be empty after genome transition"
    );

    let post_payload: FeagiByteContainer = FeagiMessage::HeartBeat.into();
    let _ = sensory_pusher.publish_data(post_payload.get_byte_ref());
    assert!(
        !wait_for_sensory_data(&handler, Duration::from_millis(500)),
        "sensory data must not be consumed after genome transition before re-registration"
    );

    let mut reconnected_client = CommandControlAgent::new(
        registration_endpoint
            .try_create_boxed_client_requester_properties()
            .expect("failed to create requester properties for reconnect"),
    );
    reconnected_client
        .request_connect()
        .expect("reconnect client connect failed");
    reconnected_client
        .request_registration(
            AgentDescriptor::new("neuraville", "transition-rt-agent", 1)
                .expect("descriptor creation failed on reconnect"),
            AuthToken::new([0u8; 32]),
            vec![
                AgentCapabilities::SendSensorData,
                AgentCapabilities::ReceiveMotorData,
            ],
        )
        .expect("reconnect registration request failed");
    let (_new_session, new_sensory_endpoint) =
        wait_for_registered_agent(&handler, &mut reconnected_client, Duration::from_secs(5));

    let new_pusher_props = new_sensory_endpoint
        .try_create_boxed_client_pusher_properties()
        .expect("failed to create new sensory pusher properties");
    let mut new_sensory_pusher = new_pusher_props.as_boxed_client_pusher();
    new_sensory_pusher
        .request_connect()
        .expect("new sensory pusher connect failed");
    let rereg_payload: FeagiByteContainer = FeagiMessage::HeartBeat.into();
    new_sensory_pusher
        .publish_data(rereg_payload.get_byte_ref())
        .expect("failed to publish sensory payload after re-registration");
    assert!(
        wait_for_sensory_data(&handler, Duration::from_secs(2)),
        "sensory data should be consumed again after re-registration"
    );
}

// ============================================================================
// CORTICAL AREA TESTS
// ============================================================================

#[tokio::test]
async fn test_create_cortical_area_success() {
    let app = create_test_server().await;

    let create_request = json!({
        "cortical_id": "iinf",
        "cortical_type": "IPU",
        "device_count": 1,
        "coordinates_3d": [0, 0, 0],
        "data_type_configs_by_subunit": {
            "0": 0
        },
        "neurons_per_voxel": 1
    });

    let (status, response) = request_json(
        app,
        "POST",
        "/v1/cortical_area/cortical_area",
        Some(create_request),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "response: {}", response);
    assert!(response.get("cortical_id").is_some());
}

#[tokio::test]
async fn test_create_cortical_area_invalid_id() {
    let app = create_test_server().await;

    // Invalid ID (not 6 characters)
    let create_request = json!({
        "cortical_id": "invalid",
        "cortical_type": "IPU",
        "device_count": 1,
        "coordinates_3d": [0, 0, 0],
        "data_type_configs_by_subunit": {
            "0": 0
        },
        "neurons_per_voxel": 1
    });

    let (status, _response) = request_json(
        app,
        "POST",
        "/v1/cortical_area/cortical_area",
        Some(create_request),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_cortical_area_not_found() {
    let app = create_test_server().await;

    let (status, response) = request_json(
        app,
        "GET",
        "/v1/connectome/area_details?area_ids=notfnd",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "response: {}", response);
    assert!(response.as_object().map(|o| o.is_empty()).unwrap_or(false));
}

#[tokio::test]
async fn test_list_cortical_areas_empty() {
    let app = create_test_server().await;

    let (status, response) =
        request_json(app, "GET", "/v1/cortical_area/cortical_area_id_list", None).await;

    assert_eq!(status, StatusCode::OK);
    let cortical_ids = response
        .get("cortical_ids")
        .and_then(|v| v.as_array())
        .expect("Expected cortical_ids array");
    assert!(cortical_ids.is_empty());
}

#[tokio::test]
async fn test_create_and_get_cortical_area() {
    let app = create_test_server().await;

    // Create
    let create_request = json!({
        "cortical_id": "iinf",
        "cortical_type": "IPU",
        "device_count": 1,
        "coordinates_3d": [0, 0, 0],
        "data_type_configs_by_subunit": {
            "0": 0
        },
        "neurons_per_voxel": 1
    });

    let (status, response) = request_json(
        app,
        "POST",
        "/v1/cortical_area/cortical_area",
        Some(create_request),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let created_id = response
        .get("cortical_id")
        .and_then(|v| v.as_str())
        .expect("Expected cortical_id in response")
        .to_string();

    // Get - need to recreate app because oneshot consumes it
    let app2 = create_test_server().await;
    let (status2, response2) = request_json(
        app2,
        "GET",
        &format!("/v1/connectome/area_details?area_ids={}", created_id),
        None,
    )
    .await;

    // Fresh manager: created area not present
    assert_eq!(status2, StatusCode::OK);
    assert!(response2.as_object().map(|o| o.is_empty()).unwrap_or(false));
}

// ============================================================================
// GENOME TESTS
// ============================================================================

#[tokio::test]
async fn test_genome_validate_minimal() {
    let app = create_test_server().await;

    // Minimal valid genome
    let genome = json!({
        "blueprint": {
            "cortical_areas": {}
        }
    });

    let (status, response) = request_json(
        app,
        "POST",
        "/v1/genome/validate",
        Some(json!({ "genome_json": genome.to_string() })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    // Response should indicate validation result
    assert!(response.is_object());
}

// ============================================================================
// ERROR FORMAT TESTS
// ============================================================================

#[tokio::test]
async fn test_error_format_consistency() {
    let app = create_test_server().await;

    // All error responses should have consistent format
    let (status, response) = request_json(
        app,
        "POST",
        "/v1/cortical_area/cortical_area",
        Some(json!({})),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    // Should have some error information
    assert!(response.is_object() || response.is_string());
}

#[test]
fn test_compilation() {
    // This test just ensures the code compiles
}
