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
use feagi_api::common::agent_registration::auto_create_cortical_areas_from_device_registrations;
use feagi_api::common::{Json as ApiJson, State as ApiStateExtract};
use feagi_api::endpoints::agent::register_agent;
use feagi_api::transports::http::server::{create_http_server, ApiState};
use feagi_api::v1::AgentRegistrationRequest;
#[cfg(feature = "feagi-agent")]
use feagi_agent::registration::{AgentDescriptor, AuthToken};
use feagi_brain_development::ConnectomeManager;
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
use std::sync::{Arc, Mutex, OnceLock};
// tower::util::ServiceExt requires the "util" feature which may not be enabled
// Using axum's test utilities instead

/// Build ApiState with initialized components.
/// Each test gets a fresh, isolated manager (no singleton conflicts)
fn build_test_state() -> ApiState {
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
                frequency_hz: 0.0,
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
    let path = std::path::PathBuf::from(format!(
        "/tmp/feagi-config-{nanos}--temp.toml"
    ));
    let contents = format!(
        "[agent]\nauto_create_missing_cortical_areas = {}\n",
        auto_create
    );
    std::fs::write(&path, contents).expect("Failed to write temp config");

    let previous = std::env::var("FEAGI_CONFIG_PATH").ok();
    std::env::set_var("FEAGI_CONFIG_PATH", &path);

    ConfigEnvGuard { previous, path }
}

#[cfg(feature = "feagi-agent")]
fn sample_device_registrations() -> Value {
    json!({
        "input_units_and_encoder_properties": {
            "vision": [
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
            "rotary_motor": [
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

    // TODO: Update to use axum test utilities or enable tower util feature
    // For now, comment out the actual test logic until proper test utilities are set up
    /*
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
    */
    // Temporarily return default values until test is fully updated
    let _ = (app, request); // Suppress unused variable warnings
    (StatusCode::OK, json!(null))
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

    let (status, response) = request_json(app, "GET", "/v1/system/readiness_check", None).await;

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
    assert!(transport.get("session_id").is_some());
    let endpoints = transport
        .get("endpoints")
        .and_then(|value| value.as_object())
        .expect("Expected endpoints in transport payload");
    assert!(endpoints.contains_key("send_sensor_data"));
}

#[cfg(feature = "feagi-agent")]
#[tokio::test]
async fn test_auto_create_disabled_skips_creation() {
    let _lock = CONFIG_ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("Failed to lock config env");
    let _guard = set_temp_config(false);
    let state = build_test_state();

    auto_create_cortical_areas_from_device_registrations(
        &state,
        &sample_device_registrations(),
    )
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
    let _lock = CONFIG_ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("Failed to lock config env");
    let _guard = set_temp_config(true);
    let state = build_test_state();

    auto_create_cortical_areas_from_device_registrations(
        &state,
        &sample_device_registrations(),
    )
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
        "cortical_id": "test01",
        "name": "Test Area",
        "dimensions": {
            "width": 10,
            "height": 10,
            "depth": 1
        },
        "area_type": "memory"
    });

    let (status, response) =
        request_json(app, "POST", "/v1/connectome/areas", Some(create_request)).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(response["cortical_id"], "test01");
    assert_eq!(response["name"], "Test Area");
}

#[tokio::test]
async fn test_create_cortical_area_invalid_id() {
    let app = create_test_server().await;

    // Invalid ID (not 6 characters)
    let create_request = json!({
        "cortical_id": "test",
        "name": "Test Area",
        "dimensions": {
            "width": 10,
            "height": 10,
            "depth": 1
        },
        "area_type": "memory"
    });

    let (status, _response) =
        request_json(app, "POST", "/v1/connectome/areas", Some(create_request)).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_cortical_area_not_found() {
    let app = create_test_server().await;

    let (status, _response) = request_json(app, "GET", "/v1/connectome/areas/notfnd", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_cortical_areas_empty() {
    let app = create_test_server().await;

    let (status, response) = request_json(app, "GET", "/v1/connectome/areas", None).await;

    assert_eq!(status, StatusCode::OK);
    assert!(response.is_array());
    assert_eq!(response.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_create_and_get_cortical_area() {
    let app = create_test_server().await;

    // Create
    let create_request = json!({
        "cortical_id": "area01",
        "name": "Area 1",
        "dimensions": {"width": 5, "height": 5, "depth": 1},
        "area_type": "memory"
    });

    let (status, _) = request_json(app, "POST", "/v1/connectome/areas", Some(create_request)).await;
    assert_eq!(status, StatusCode::CREATED);

    // Get - need to recreate app because oneshot consumes it
    let app2 = create_test_server().await;
    let (status2, _response2) =
        request_json(app2, "GET", "/v1/connectome/areas/area01", None).await;

    // This will fail because each test gets a fresh manager
    // This demonstrates the isolation - which is good for parallel testing
    assert_eq!(status2, StatusCode::NOT_FOUND);
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
    let (status, response) = request_json(app, "GET", "/v1/connectome/areas/notfnd", None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    // Should have some error information
    assert!(response.is_object() || response.is_string());
}

#[test]
fn test_compilation() {
    // This test just ensures the code compiles
}
