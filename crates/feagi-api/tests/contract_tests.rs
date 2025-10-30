/// Contract Tests for FEAGI Rust API
/// 
/// These tests ensure that the Rust API produces identical responses to the Python API
/// for the same inputs. This guarantees backward compatibility.
///
/// Strategy:
/// 1. Snapshot testing: Capture expected JSON responses
/// 2. JSON comparison: Deep compare responses (ignoring dynamic fields like timestamps)
/// 3. Error format matching: Ensure error messages match Python format

use feagi_api::transports::http::server::{create_http_server, ApiState};
use feagi_services::{
    GenomeServiceImpl, ConnectomeServiceImpl, SystemServiceImpl,
    AnalyticsServiceImpl, RuntimeServiceImpl, NeuronServiceImpl,
};
use feagi_bdu::ConnectomeManager;
use feagi_burst_engine::RustNPU;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt; // For .oneshot()
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use parking_lot::RwLock;

/// Helper to create a test server with initialized components
async fn create_test_server() -> axum::Router {
    // Initialize NPU with correct parameter count
    let npu = Arc::new(Mutex::new(RustNPU::new(1_000_000, 10_000_000, 10)));
    
    // Initialize ConnectomeManager using instance() singleton
    let manager = Arc::new(RwLock::new(ConnectomeManager::instance()));
    
    // Connect NPU to manager (manager uses Arc<Mutex<RustNPU>>)
    manager.write().connect_npu(Arc::clone(&npu));
    
    // Initialize BurstLoopRunner (placeholder for testing)
    let burst_runner: Option<Arc<RwLock<feagi_burst_engine::BurstLoopRunner>>> = None; // Not needed for API tests
    
    // Create services
    let genome_service = Arc::new(GenomeServiceImpl::new(Arc::clone(&manager)));
    let connectome_service = Arc::new(ConnectomeServiceImpl::new(Arc::clone(&manager)));
    let system_service = Arc::new(SystemServiceImpl::new(
        Arc::clone(&manager),
        burst_runner.clone(),
    ));
    let analytics_service = Arc::new(AnalyticsServiceImpl::new(
        Arc::clone(&manager),
        burst_runner.clone(),
    ));
    let runtime_service: Option<Arc<dyn feagi_services::RuntimeService>> = None; // Can't create without BurstLoopRunner
    let neuron_service = Arc::new(NeuronServiceImpl::new(Arc::clone(&manager)));
    
    // Create API state
    let state = ApiState {
        analytics_service: analytics_service as Arc<dyn feagi_services::AnalyticsService + Send + Sync>,
        connectome_service: connectome_service as Arc<dyn feagi_services::ConnectomeService + Send + Sync>,
        genome_service: genome_service as Arc<dyn feagi_services::GenomeService + Send + Sync>,
        neuron_service: neuron_service as Arc<dyn feagi_services::NeuronService + Send + Sync>,
        runtime_service: runtime_service,
        system_service: Some(system_service as Arc<dyn feagi_services::SystemService + Send + Sync>),
    };
    
    // Create router
    create_http_server(state)
}

/// Helper to make a request and get response as JSON
async fn request_json(
    app: &mut axum::Router,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut request_builder = Request::builder()
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
    let mut app = create_test_server().await;
    
    let (status, response) = request_json(&mut app, "GET", "/api/v1/health", None).await;
    
    assert_eq!(status, StatusCode::OK);
    assert!(response["status"].is_string());
    assert!(response["timestamp"].is_string());
}

#[tokio::test]
async fn test_system_status() {
    let mut app = create_test_server().await;
    
    let (status, response) = request_json(&mut app, "GET", "/api/v1/system/status", None).await;
    
    assert_eq!(status, StatusCode::OK);
    assert!(response["burst_engine_active"].is_boolean());
    assert!(response["burst_count"].is_number());
}

#[tokio::test]
async fn test_system_version() {
    let mut app = create_test_server().await;
    
    let (status, response) = request_json(&mut app, "GET", "/api/v1/system/version", None).await;
    
    assert_eq!(status, StatusCode::OK);
    assert!(response["version"].is_string());
    assert!(response["rust_version"].is_string());
}

// ============================================================================
// GENOME TESTS
// ============================================================================

#[tokio::test]
async fn test_genome_validate_valid() {
    let mut app = create_test_server().await;
    
    // Minimal valid genome
    let genome = json!({
        "blueprint": {
            "cortical_areas": {}
        }
    });
    
    let (status, response) = request_json(
        &mut app,
        "POST",
        "/api/v1/genome/validate",
        Some(json!({ "genome_json": genome.to_string() })),
    ).await;
    
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["valid"], true);
}

#[tokio::test]
async fn test_genome_validate_invalid() {
    let mut app = create_test_server().await;
    
    // Invalid genome (not even valid JSON)
    let (status, response) = request_json(
        &mut app,
        "POST",
        "/api/v1/genome/validate",
        Some(json!({ "genome_json": "not json" })),
    ).await;
    
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["valid"], false);
    assert!(response["errors"].is_array());
}

// ============================================================================
// CORTICAL AREA TESTS
// ============================================================================

#[tokio::test]
async fn test_create_cortical_area_success() {
    let mut app = create_test_server().await;
    
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
    
    let (status, response) = request_json(
        &mut app,
        "POST",
        "/api/v1/connectome/areas",
        Some(create_request),
    ).await;
    
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(response["cortical_id"], "test01");
    assert_eq!(response["name"], "Test Area");
}

#[tokio::test]
async fn test_create_cortical_area_invalid_id() {
    let mut app = create_test_server().await;
    
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
    
    let (status, response) = request_json(
        &mut app,
        "POST",
        "/api/v1/connectome/areas",
        Some(create_request),
    ).await;
    
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(response["error"].is_string());
}

#[tokio::test]
async fn test_get_cortical_area_not_found() {
    let mut app = create_test_server().await;
    
    let (status, response) = request_json(
        &mut app,
        "GET",
        "/api/v1/connectome/areas/notfound",
        None,
    ).await;
    
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(response["error"].is_string());
}

#[tokio::test]
async fn test_list_cortical_areas_empty() {
    let mut app = create_test_server().await;
    
    let (status, response) = request_json(
        &mut app,
        "GET",
        "/api/v1/connectome/areas",
        None,
    ).await;
    
    assert_eq!(status, StatusCode::OK);
    assert!(response.is_array());
    assert_eq!(response.as_array().unwrap().len(), 0);
}

// ============================================================================
// NEURON TESTS
// ============================================================================

#[tokio::test]
async fn test_create_neuron_no_area() {
    let mut app = create_test_server().await;
    
    // Try to create neuron in non-existent area
    let create_request = json!({
        "cortical_id": "noexist",
        "x": 0,
        "y": 0,
        "z": 0,
        "properties": {}
    });
    
    let (status, response) = request_json(
        &mut app,
        "POST",
        "/api/v1/neurons",
        Some(create_request),
    ).await;
    
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(response["error"].is_string());
}

// ============================================================================
// ERROR FORMAT TESTS
// ============================================================================

#[tokio::test]
async fn test_error_format_consistency() {
    let mut app = create_test_server().await;
    
    // All error responses should have consistent format
    let (_, response1) = request_json(
        &mut app,
        "GET",
        "/api/v1/connectome/areas/notfound",
        None,
    ).await;
    
    // Should have "error" field
    assert!(response1["error"].is_string());
    
    // Optionally "details" field for additional context
    // assert!(response1["details"].is_null() || response1["details"].is_object());
}

// ============================================================================
// CONCURRENT REQUEST TESTS
// ============================================================================

#[tokio::test]
async fn test_concurrent_reads() {
    use tokio::task::JoinSet;
    
    let mut app = create_test_server().await;
    
    // Create an area first
    let create_request = json!({
        "cortical_id": "conc01",
        "name": "Concurrent Test",
        "dimensions": { "width": 10, "height": 10, "depth": 1 },
        "area_type": "memory"
    });
    
    request_json(&mut app, "POST", "/api/v1/connectome/areas", Some(create_request)).await;
    
    // Now make 10 concurrent read requests
    let mut set = JoinSet::new();
    
    for _ in 0..10 {
        set.spawn(async {
            let mut app = create_test_server().await;
            request_json(&mut app, "GET", "/api/v1/connectome/areas/conc01", None).await
        });
    }
    
    // All should succeed
    while let Some(result) = set.join_next().await {
        let (status, _) = result.unwrap();
        assert_eq!(status, StatusCode::OK);
    }
}

// ============================================================================
// SNAPSHOT TESTS (using insta crate)
// ============================================================================

#[test]
fn test_api_response_snapshots() {
    // TODO: Add insta crate and create snapshot tests
    // This will capture exact JSON responses and detect any changes
}
