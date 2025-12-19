// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// Contract testing utilities
//
// Helper functions for comparing Rust API responses with Python API snapshots.

use serde_json::Value;

/// Compare two JSON values, ignoring dynamic fields like timestamps
pub fn assert_json_structure_matches(actual: &Value, expected: &Value, path: &str) {
    match (actual, expected) {
        (Value::Object(actual_map), Value::Object(expected_map)) => {
            // Check that all expected keys exist in actual
            for (key, expected_value) in expected_map {
                let current_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };

                // Skip dynamic fields
                if is_dynamic_field(key) {
                    continue;
                }

                let actual_value = actual_map.get(key).expect(&format!(
                    "Missing field '{}' in actual response at path '{}'",
                    key, current_path
                ));

                assert_json_structure_matches(actual_value, expected_value, &current_path);
            }
        }
        (Value::Array(actual_arr), Value::Array(expected_arr)) => {
            assert_eq!(
                actual_arr.len(),
                expected_arr.len(),
                "Array length mismatch at path '{}'",
                path
            );

            for (i, (actual_item, expected_item)) in
                actual_arr.iter().zip(expected_arr.iter()).enumerate()
            {
                let current_path = format!("{}[{}]", path, i);
                assert_json_structure_matches(actual_item, expected_item, &current_path);
            }
        }
        (Value::String(_), Value::String(_)) => {
            // Skip dynamic string values (timestamps, paths, etc.)
            if !is_dynamic_field(path) {
                assert_eq!(
                    actual, expected,
                    "String value mismatch at path '{}'",
                    path
                );
            }
        }
        (Value::Number(_), Value::Number(_)) => {
            // Numbers should match (unless dynamic)
            if !is_dynamic_field(path) {
                assert_eq!(
                    actual, expected,
                    "Number value mismatch at path '{}'",
                    path
                );
            }
        }
        (Value::Bool(actual_bool), Value::Bool(expected_bool)) => {
            assert_eq!(
                actual_bool, expected_bool,
                "Boolean value mismatch at path '{}'",
                path
            );
        }
        (Value::Null, Value::Null) => {
            // Both null, OK
        }
        _ => {
            panic!(
                "Type mismatch at path '{}': actual = {:?}, expected = {:?}",
                path, actual, expected
            );
        }
    }
}

/// Check if a field is dynamic (timestamp, path, etc.) and should be ignored in comparison
fn is_dynamic_field(field: &str) -> bool {
    let dynamic_fields = [
        "timestamp",
        "genome_timestamp",
        "connectome_path",
        "neuron_count",      // May vary based on loaded genome
        "synapse_count",     // May vary based on loaded genome
        "cortical_area_count", // May vary based on loaded genome
    ];

    dynamic_fields.iter().any(|f| field.contains(f))
}

/// Assert that the response has the expected success status
pub fn assert_success_response(response: &Value) {
    assert_eq!(
        response.get("success").and_then(|v| v.as_bool()),
        Some(true),
        "Response should have success: true"
    );

    assert!(
        response.get("data").is_some(),
        "Successful response should have 'data' field"
    );

    assert!(
        response.get("timestamp").is_some(),
        "Response should have 'timestamp' field"
    );
}

/// Assert that the response has the expected error status
pub fn assert_error_response(response: &Value, expected_code: u16) {
    assert_eq!(
        response.get("code").and_then(|v| v.as_u64()),
        Some(expected_code as u64),
        "Error response should have correct status code"
    );

    assert!(
        response.get("message").is_some(),
        "Error response should have 'message' field"
    );
}





