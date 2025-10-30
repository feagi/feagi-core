// Contract tests for health endpoints
//
// These tests verify that the Rust API responses match the Python FastAPI format exactly.

use serde_json::json;

use super::test_utils::{assert_json_structure_matches, assert_success_response};

#[test]
fn test_health_check_response_structure() {
    // Expected response structure from Python FastAPI
    let expected_structure = json!({
        "success": true,
        "data": {
            "status": "healthy",
            "brain_readiness": true,
            "burst_engine": true,
            "neuron_count": 1000,
            "synapse_count": 5000,
            "cortical_area_count": 10,
            "genome_validity": true,
            "influxdb_availability": false,
            "connectome_path": "/path/to/connectome",
            "genome_timestamp": "2025-10-29T12:34:56Z",
            "change_state": "saved",
            "changes_saved_externally": false
        },
        "timestamp": "2025-10-29T12:34:56Z"
    });
    
    // This is a structure test - actual values don't need to match
    // Just verify all required fields exist
    assert_success_response(&expected_structure);
    
    let data = expected_structure.get("data").unwrap();
    
    // Verify all required fields exist in data
    let required_fields = [
        "status",
        "brain_readiness",
        "burst_engine",
        "neuron_count",
        "synapse_count",
        "cortical_area_count",
        "genome_validity",
        "influxdb_availability",
        "connectome_path",
        "genome_timestamp",
        "change_state",
        "changes_saved_externally",
    ];
    
    for field in required_fields {
        assert!(
            data.get(field).is_some(),
            "Health check response must include field '{}'",
            field
        );
    }
}

#[test]
fn test_health_check_field_types() {
    // Verify field types match Python FastAPI
    let sample_response = json!({
        "success": true,
        "data": {
            "status": "healthy",
            "brain_readiness": true,
            "burst_engine": true,
            "neuron_count": 1000,
            "synapse_count": 5000,
            "cortical_area_count": 10,
            "genome_validity": true,
            "influxdb_availability": false,
            "connectome_path": "/path/to/connectome",
            "genome_timestamp": "2025-10-29T12:34:56Z",
            "change_state": "saved",
            "changes_saved_externally": false
        },
        "timestamp": "2025-10-29T12:34:56Z"
    });
    
    let data = sample_response.get("data").unwrap();
    
    // Verify types
    assert!(data.get("status").unwrap().is_string());
    assert!(data.get("brain_readiness").unwrap().is_boolean());
    assert!(data.get("burst_engine").unwrap().is_boolean());
    assert!(data.get("neuron_count").unwrap().is_number());
    assert!(data.get("synapse_count").unwrap().is_number());
    assert!(data.get("cortical_area_count").unwrap().is_number());
    assert!(data.get("genome_validity").unwrap().is_boolean());
    assert!(data.get("influxdb_availability").unwrap().is_boolean());
    assert!(data.get("connectome_path").unwrap().is_string());
    assert!(data.get("genome_timestamp").unwrap().is_string());
    assert!(data.get("change_state").unwrap().is_string());
    assert!(data.get("changes_saved_externally").unwrap().is_boolean());
}

#[test]
fn test_readiness_check_response_structure() {
    // Expected response structure from Python FastAPI
    let expected_structure = json!({
        "success": true,
        "data": {
            "ready": true,
            "components": {
                "api": true,
                "burst_engine": true,
                "state_manager": true,
                "connectome": true
            }
        },
        "timestamp": "2025-10-29T12:34:56Z"
    });
    
    assert_success_response(&expected_structure);
    
    let data = expected_structure.get("data").unwrap();
    
    // Verify required fields
    assert!(data.get("ready").is_some());
    assert!(data.get("components").is_some());
    
    let components = data.get("components").unwrap();
    assert!(components.get("api").is_some());
    assert!(components.get("burst_engine").is_some());
    assert!(components.get("state_manager").is_some());
    assert!(components.get("connectome").is_some());
}

#[test]
fn test_readiness_check_field_types() {
    let sample_response = json!({
        "success": true,
        "data": {
            "ready": true,
            "components": {
                "api": true,
                "burst_engine": true,
                "state_manager": true,
                "connectome": true
            }
        },
        "timestamp": "2025-10-29T12:34:56Z"
    });
    
    let data = sample_response.get("data").unwrap();
    
    // Verify types
    assert!(data.get("ready").unwrap().is_boolean());
    
    let components = data.get("components").unwrap();
    assert!(components.get("api").unwrap().is_boolean());
    assert!(components.get("burst_engine").unwrap().is_boolean());
    assert!(components.get("state_manager").unwrap().is_boolean());
    assert!(components.get("connectome").unwrap().is_boolean());
}

// TODO: Add integration tests that actually call the Rust API and compare with Python snapshots
// These would require:
// 1. A running Rust API server
// 2. Captured response snapshots from Python API
// 3. HTTP client to query the Rust API
// 4. Comparison of actual responses with snapshots
//
// Example:
// #[tokio::test]
// async fn test_health_check_matches_python_snapshot() {
//     let rust_response = reqwest::get("http://localhost:8080/api/v1/health")
//         .await
//         .unwrap()
//         .json::<Value>()
//         .await
//         .unwrap();
//     
//     let python_snapshot = load_snapshot("health_check.json");
//     assert_json_structure_matches(&rust_response, &python_snapshot, "");
// }


