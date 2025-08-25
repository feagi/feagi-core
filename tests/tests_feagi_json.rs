use feagi_core_data_structures_and_processing::io_data::FeagiJSON;
use feagi_core_data_structures_and_processing::io_processing::byte_structures::FeagiByteStructure;
use feagi_core_data_structures_and_processing::io_processing::byte_structures::FeagiByteStructureCompatible;
use serde_json::json;

#[test]
fn test_json_structure_serialize_deserialize_simple() {
    // Create a simple JSON structure from a string
    let json_string = r#"{"name": "test", "value": 42, "active": true}"#;
    let json_structure = FeagiJSON::from_json_string(json_string.to_string()).unwrap();

    // Serialize to bytes
    let sending_byte_structure = json_structure.as_new_feagi_byte_structure().unwrap();
    let bytes = sending_byte_structure.copy_out_as_byte_vector();

    // Deserialize back (pretend bytes were sent over network)
    let received_byte_structure = FeagiByteStructure::create_from_bytes(bytes).unwrap();
    let received_json_structure = FeagiJSON::new_from_feagi_byte_structure(&received_byte_structure).unwrap();

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
    let sending_byte_structure = json_structure.as_new_feagi_byte_structure().unwrap();
    let bytes = sending_byte_structure.copy_out_as_byte_vector();

    let received_byte_structure = FeagiByteStructure::create_from_bytes(bytes).unwrap();
    let received_json_structure = FeagiJSON::new_from_feagi_byte_structure(&received_byte_structure).unwrap();

    // Compare the original JSON value with the received one
    let received_value = received_json_structure.borrow_json_value();
    assert_eq!(&json_value, received_value);
}

#[test]
fn test_json_structure_empty_object() {
    // Test with empty JSON object
    let json_string = "{}";
    let json_structure = FeagiJSON::from_json_string(json_string.to_string()).unwrap();

    let sending_byte_structure = json_structure.as_new_feagi_byte_structure().unwrap();
    let bytes = sending_byte_structure.copy_out_as_byte_vector();

    let received_byte_structure = FeagiByteStructure::create_from_bytes(bytes).unwrap();
    let received_json_structure = FeagiJSON::new_from_feagi_byte_structure(&received_byte_structure).unwrap();

    let original_value: serde_json::Value = json!({});
    let received_value = received_json_structure.borrow_json_value();
    assert_eq!(&original_value, received_value);
}

#[test]
fn test_json_structure_array() {
    // Test with JSON array
    let json_value = json!([1, 2, 3, "hello", true, null, {"nested": "object"}]);
    let json_structure = FeagiJSON::from_json_value(json_value.clone());

    let sending_byte_structure = json_structure.as_new_feagi_byte_structure().unwrap();
    let bytes = sending_byte_structure.copy_out_as_byte_vector();

    let received_byte_structure = FeagiByteStructure::create_from_bytes(bytes).unwrap();
    let received_json_structure = FeagiJSON::new_from_feagi_byte_structure(&received_byte_structure).unwrap();

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

    let sending_byte_structure = json_structure.as_new_feagi_byte_structure().unwrap();
    let bytes = sending_byte_structure.copy_out_as_byte_vector();

    let received_byte_structure = FeagiByteStructure::create_from_bytes(bytes).unwrap();
    let received_json_structure = FeagiJSON::new_from_feagi_byte_structure(&received_byte_structure).unwrap();

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
    let size1 = json_structure.max_number_bytes_needed();
    let size2 = json_structure.max_number_bytes_needed();
    println!("Size check: {} == {}", size1, size2);
    assert_eq!(size1, size2);

    // Create a manual bytes vector and serialize to it
    let mut manual_bytes = vec![0u8; size1];
    println!("Manual bytes before serialization: {:?}", &manual_bytes[0..4.min(manual_bytes.len())]);

    let result = json_structure.overwrite_feagi_byte_structure_slice(&mut manual_bytes);
    println!("Serialization result: {:?}", result);
    println!("Manual bytes after serialization: {:?}", &manual_bytes[0..4.min(manual_bytes.len())]);

    // Verify we can deserialize it back
    let structure = FeagiByteStructure::create_from_bytes(manual_bytes.clone()).unwrap();
    let received_json_structure = FeagiJSON::new_from_feagi_byte_structure(&structure).unwrap();
    
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