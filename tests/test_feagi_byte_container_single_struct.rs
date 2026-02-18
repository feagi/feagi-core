use feagi_serialization::{FeagiByteContainer, FeagiSerializable};
use feagi_structures::FeagiJSON;
use serde_json::json;

#[test]
fn single_struct_allocation_includes_session_id() {
    let json_struct = FeagiJSON::from_json_value(json!({"key": "value"}));
    let mut container = FeagiByteContainer::new_empty();

    container
        .overwrite_byte_data_with_single_struct_data(&json_struct, 1)
        .expect("Failed to overwrite byte container with single struct");

    let expected_bytes = FeagiByteContainer::GLOBAL_BYTE_HEADER_BYTE_COUNT
        + FeagiByteContainer::AGENT_ID_BYTE_COUNT
        + FeagiByteContainer::STRUCTURE_LOOKUP_HEADER_BYTE_COUNT_PER_STRUCTURE
        + json_struct.get_number_of_bytes_needed();

    assert_eq!(
        container.get_number_of_bytes_used(),
        expected_bytes,
        "Single-struct containers must reserve space for SessionID"
    );
}
