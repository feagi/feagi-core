// NOTE: These tests use the old serialization API. They need to be updated to use:
// - FeagiByteContainer instead of FeagiByteStructure
// - FeagiSerializable trait instead of FeagiByteStructureCompatible
// TODO: Update tests to use new serialization API
use feagi_serialization::{FeagiByteContainer, FeagiByteStructureType, FeagiSerializable};
use feagi_structures::FeagiJSON;
use serde_json::json;

#[test]
fn test_json_structure_serialize_deserialize_simple() {
    // Create a simple JSON structure from a string
    let json_string = r#"{"name": "test", "value": 42, "active": true}"#;
    let json_structure = FeagiJSON::from_json_string(json_string.to_string()).unwrap();

    // Serialize to bytes
    let mut sending_container = FeagiByteContainer::new_empty();
    sending_container
        .overwrite_byte_data_with_single_struct_data(&json_structure, 0)
        .unwrap();
    let bytes = sending_container.get_byte_ref().to_vec();

    // Deserialize back (pretend bytes were sent over network)
    let received_boxed = {
        let mut received_container = FeagiByteContainer::new_empty();
        received_container
            .try_write_data_by_ownership_to_container_and_verify(bytes)
            .unwrap();
        received_container
            .try_create_struct_from_first_found_struct_of_type(FeagiByteStructureType::JSON)
            .unwrap()
            .unwrap()
    };
    let received_json_structure = FeagiJSON::try_from(received_boxed).unwrap();

    // Check that the JSON content is consistent
    let original_json_string = json_structure.to_string();
    let received_json_string = received_json_structure.to_string();

    // Parse both to serde_json::Value for comparison (to handle formatting differences)
    let original_value: serde_json::Value = serde_json::from_str(&original_json_string).unwrap();
    let received_value: serde_json::Value = serde_json::from_str(&received_json_string).unwrap();

    assert_eq!(original_value, received_value);
}

#[test]
fn test_json_structure_serialize_deserialize_complex() {
    // Create a more complex JSON structure using serde_json::json! macro
    let json_value = json!({
        "users": [
            {
                "id": 1,
                "name": "Alice",
                "preferences": {
                    "theme": "dark",
                    "notifications": true
                }
            },
            {
                "id": 2,
                "name": "Bob",
                "preferences": {
                    "theme": "light",
                    "notifications": false
                }
            }
        ],
        "metadata": {
            "version": "1.0.0",
            "timestamp": "2024-01-01T00:00:00Z",
            "features": ["auth", "notifications", "themes"]
        }
    });

    let json_structure = FeagiJSON::from_json_value(json_value.clone());

    // Test serialization/deserialization
    let mut sending_container = FeagiByteContainer::new_empty();
    sending_container
        .overwrite_byte_data_with_single_struct_data(&json_structure, 0)
        .unwrap();
    let bytes = sending_container.get_byte_ref().to_vec();
    let received_boxed = {
        let mut received_container = FeagiByteContainer::new_empty();
        received_container
            .try_write_data_by_ownership_to_container_and_verify(bytes)
            .unwrap();
        received_container
            .try_create_struct_from_first_found_struct_of_type(FeagiByteStructureType::JSON)
            .unwrap()
            .unwrap()
    };
    let received_json_structure = FeagiJSON::try_from(received_boxed).unwrap();

    // Compare the original JSON value with the received one
    let received_value = received_json_structure.borrow_json_value();
    assert_eq!(&json_value, received_value);
}

#[test]
fn test_json_structure_empty_object() {
    // Test with empty JSON object
    let json_string = "{}";
    let json_structure = FeagiJSON::from_json_string(json_string.to_string()).unwrap();

    let mut sending_container = FeagiByteContainer::new_empty();
    sending_container
        .overwrite_byte_data_with_single_struct_data(&json_structure, 0)
        .unwrap();
    let bytes = sending_container.get_byte_ref().to_vec();
    let received_boxed = {
        let mut received_container = FeagiByteContainer::new_empty();
        received_container
            .try_write_data_by_ownership_to_container_and_verify(bytes)
            .unwrap();
        received_container
            .try_create_struct_from_first_found_struct_of_type(FeagiByteStructureType::JSON)
            .unwrap()
            .unwrap()
    };
    let received_json_structure = FeagiJSON::try_from(received_boxed).unwrap();

    let original_value: serde_json::Value = json!({});
    let received_value = received_json_structure.borrow_json_value();
    assert_eq!(&original_value, received_value);
}

#[test]
fn test_json_structure_array() {
    // Test with JSON array
    let json_value = json!([1, 2, 3, "hello", true, null, {"nested": "object"}]);
    let json_structure = FeagiJSON::from_json_value(json_value.clone());

    let mut sending_container = FeagiByteContainer::new_empty();
    sending_container
        .overwrite_byte_data_with_single_struct_data(&json_structure, 0)
        .unwrap();
    let bytes = sending_container.get_byte_ref().to_vec();
    let received_boxed = {
        let mut received_container = FeagiByteContainer::new_empty();
        received_container
            .try_write_data_by_ownership_to_container_and_verify(bytes)
            .unwrap();
        received_container
            .try_create_struct_from_first_found_struct_of_type(FeagiByteStructureType::JSON)
            .unwrap()
            .unwrap()
    };
    let received_json_structure = FeagiJSON::try_from(received_boxed).unwrap();

    let received_value = received_json_structure.borrow_json_value();
    assert_eq!(&json_value, received_value);
}

#[test]
fn test_json_structure_unicode() {
    // Test with Unicode characters
    let json_value = json!({
        "message": "Hello, 世界! 🌍",
        "emoji": "🚀🎉✨",
        "multilang": {
            "english": "Hello",
            "chinese": "你好",
            "japanese": "こんにちは",
            "arabic": "مرحبا"
        }
    });

    let json_structure = FeagiJSON::from_json_value(json_value.clone());

    let mut sending_container = FeagiByteContainer::new_empty();
    sending_container
        .overwrite_byte_data_with_single_struct_data(&json_structure, 0)
        .unwrap();
    let bytes = sending_container.get_byte_ref().to_vec();
    let received_boxed = {
        let mut received_container = FeagiByteContainer::new_empty();
        received_container
            .try_write_data_by_ownership_to_container_and_verify(bytes)
            .unwrap();
        received_container
            .try_create_struct_from_first_found_struct_of_type(FeagiByteStructureType::JSON)
            .unwrap()
            .unwrap()
    };
    let received_json_structure = FeagiJSON::try_from(received_boxed).unwrap();

    let received_value = received_json_structure.borrow_json_value();
    assert_eq!(&json_value, received_value);
}

#[test]
fn test_json_structure_max_bytes_consistency() {
    // Test that max_number_bytes_needed is consistent (similar to the neuron test)
    let json_value = json!({
        "test": "data",
        "numbers": [1, 2, 3, 4, 5],
        "nested": {"key": "value"}
    });

    let json_structure = FeagiJSON::from_json_value(json_value);

    // Check if max_number_bytes_needed is consistent
    let size1 = json_structure.get_number_of_bytes_needed();
    let size2 = json_structure.get_number_of_bytes_needed();
    println!("Size check: {} == {}", size1, size2);
    assert_eq!(size1, size2);

    let mut sending_container = FeagiByteContainer::new_empty();
    sending_container
        .overwrite_byte_data_with_single_struct_data(&json_structure, 0)
        .unwrap();
    let bytes = sending_container.get_byte_ref().to_vec();
    let received_boxed = {
        let mut received_container = FeagiByteContainer::new_empty();
        received_container
            .try_write_data_by_ownership_to_container_and_verify(bytes)
            .unwrap();
        received_container
            .try_create_struct_from_first_found_struct_of_type(FeagiByteStructureType::JSON)
            .unwrap()
            .unwrap()
    };
    let received_json_structure = FeagiJSON::try_from(received_boxed).unwrap();

    // Should be able to get the JSON back
    let json_string = received_json_structure.to_string();
    assert!(!json_string.is_empty());
}

#[test]
fn test_invalid_json_string() {
    // Test error handling for invalid JSON
    let invalid_json = r#"{"invalid": json, missing quotes}"#;
    let result = FeagiJSON::from_json_string(invalid_json.to_string());
    assert!(result.is_err());
}
