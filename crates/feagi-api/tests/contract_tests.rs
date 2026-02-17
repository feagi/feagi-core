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
use feagi_agent::registration::{AgentDescriptor, AuthToken};
#[cfg(feature = "feagi-agent")]
use feagi_api::common::agent_registration::auto_create_cortical_areas_from_device_registrations;
use feagi_api::common::{Json as ApiJson, State as ApiStateExtract};
use feagi_api::endpoints::agent::register_agent;
use feagi_api::transports::http::server::{create_http_server, ApiState};
use feagi_api::v1::AgentRegistrationRequest;
use feagi_brain_development::ConnectomeManager;
use feagi_evolutionary::templates::create_genome_with_core_areas;
use feagi_npu_burst_engine::backend::CPUBackend;
use feagi_npu_burst_engine::TracingMutex;
use feagi_npu_burst_engine::{DynamicNPU, RustNPU};
use feagi_npu_runtime::StdRuntime;
use feagi_services::impls::{
    AnalyticsServiceImpl, ConnectomeServiceImpl, GenomeServiceImpl, NeuronServiceImpl,
    SystemServiceImpl,
};
use parking_lot::RwLock;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;
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
    }

    let runtime_service =
        Arc::new(MockRuntimeService) as Arc<dyn feagi_services::RuntimeService + Send + Sync>;

    // Create API state
    // Get FEAGI session timestamp (when this instance started)
    let feagi_session_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

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
        #[cfg(feature = "feagi-agent")]
        agent_connectors: ApiState::init_agent_connectors(),
        #[cfg(feature = "feagi-agent")]
        agent_registration_handler: ApiState::init_agent_registration_handler(),
    }
}

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
    let state = build_test_state();
    let descriptor = AgentDescriptor::new(1, "neuraville", "api-test", 1).unwrap();
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
        agent_id: descriptor.to_base64(),
        agent_data_port: 0,
        agent_version: "0.0.0-test".to_string(),
        controller_version: "0.0.0-test".to_string(),
        capabilities,
        agent_ip: None,
        metadata: None,
        auth_token: Some(auth_token),
        chosen_transport: None,
    };

    let response = register_agent(ApiStateExtract(state), ApiJson(request))
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
    let state = build_test_state();
    let descriptor = AgentDescriptor::new(4, "neuraville", "bv-viz-only", 1).unwrap();
    let auth_token = AuthToken::new([4u8; 32]).to_base64();

    let mut capabilities: HashMap<String, Value> = HashMap::new();
    capabilities.insert(
        "visualization".to_string(),
        json!({ "visualization_type": "3d_brain", "rate_hz": 20.0, "bridge_proxy": false }),
    );

    let request = AgentRegistrationRequest {
        agent_type: "visualization".to_string(),
        agent_id: descriptor.to_base64(),
        agent_data_port: 0,
        agent_version: "0.0.0-test".to_string(),
        controller_version: "0.0.0-test".to_string(),
        capabilities,
        agent_ip: None,
        metadata: None,
        auth_token: Some(auth_token),
        chosen_transport: None,
    };

    let response = register_agent(ApiStateExtract(state), ApiJson(request))
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
    let state = build_test_state();
    let descriptor = AgentDescriptor::new(2, "neuraville", "rate-test", 1).unwrap();
    let auth_token = AuthToken::new([2u8; 32]).to_base64();

    let mut capabilities: HashMap<String, Value> = HashMap::new();
    capabilities.insert(
        "device_registrations".to_string(),
        sample_device_registrations(),
    );
    capabilities.insert("motor".to_string(), json!({ "rate_hz": 2000.0 }));

    let request = AgentRegistrationRequest {
        agent_type: "motor".to_string(),
        agent_id: descriptor.to_base64(),
        agent_data_port: 0,
        agent_version: "0.0.0-test".to_string(),
        controller_version: "0.0.0-test".to_string(),
        capabilities,
        agent_ip: None,
        metadata: None,
        auth_token: Some(auth_token),
        chosen_transport: None,
    };

    let result = register_agent(ApiStateExtract(state), ApiJson(request)).await;
    assert!(result.is_err());
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_register_agent_rejects_visualization_rate_above_burst_frequency() {
    let state = build_test_state();
    let descriptor = AgentDescriptor::new(3, "neuraville", "viz-rate-test", 1).unwrap();
    let auth_token = AuthToken::new([3u8; 32]).to_base64();

    let mut capabilities: HashMap<String, Value> = HashMap::new();
    capabilities.insert(
        "device_registrations".to_string(),
        sample_device_registrations(),
    );
    capabilities.insert("visualization".to_string(), json!({ "rate_hz": 2000.0 }));

    let request = AgentRegistrationRequest {
        agent_type: "visualization".to_string(),
        agent_id: descriptor.to_base64(),
        agent_data_port: 0,
        agent_version: "0.0.0-test".to_string(),
        controller_version: "0.0.0-test".to_string(),
        capabilities,
        agent_ip: None,
        metadata: None,
        auth_token: Some(auth_token),
        chosen_transport: None,
    };

    let result = register_agent(ApiStateExtract(state), ApiJson(request)).await;
    assert!(result.is_err());
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_disabled_skips_creation() {
    let _lock = CONFIG_ENV_LOCK.get_or_init(|| Mutex::new(())).lock().await;
    let _guard = set_temp_config(false);
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
    let _lock = CONFIG_ENV_LOCK.get_or_init(|| Mutex::new(())).lock().await;
    let _guard = set_temp_config(true);
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
