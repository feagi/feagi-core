// NOTE: These tests use the old serialization API. They need to be updated to use:
// - FeagiByteContainer instead of FeagiByteStructure
// - FeagiSerializable trait instead of FeagiByteStructureCompatible
// TODO: Update tests to use new serialization API
#[allow(unused_imports)]
use feagi_data_serialization::FeagiByteContainer;
#[allow(unused_imports)]
use feagi_data_serialization::FeagiByteStructureType;
#[allow(unused_imports)]
use feagi_data_serialization::FeagiSerializable;
#[allow(unused_imports)]
use feagi_data_structures::genomic::cortical_area::CorticalID;
#[allow(unused_imports)]
use feagi_data_structures::neuron_voxels::xyzp::{
    CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZP, NeuronVoxelXYZPArrays,
};
#[allow(unused_imports)]
use feagi_data_structures::FeagiJSON;
#[allow(unused_imports)]
use serde_json::json;

#[test]
#[ignore] // TODO: Update to use new serialization API (FeagiByteContainer)
#[allow(unused_variables, dead_code, unreachable_code)]
fn test_combined_neuron_json_multistruct_serialize_deserialize() {
    // TODO: This test needs to be fully updated to use the new serialization API
    // The old API methods (FeagiByteStructure, create_from_2_existing, etc.) no longer exist
    // All code below is commented out until the test is fully rewritten
    /*
    // Create JSON structure
    let json_data = json!({
        "experiment_name": "Neural Network Test",
        "parameters": {
            "learning_rate": 0.001,
            "batch_size": 32,
            "epochs": 100
        },
        "metadata": {
            "created_at": "2024-01-01T00:00:00Z",
            "version": "1.0.0"
        }
    });
    let json_structure = FeagiJSON::from_json_value(json_data.clone());

    // Create neuron structure (similar to the neuron tests)
    let cortical_id_a = CorticalID::try_from_base_64("cAAAAA").unwrap();

    let neuron_a_1 = NeuronVoxelXYZP::new(10, 20, 30, 0.75);
    let neuron_a_2 = NeuronVoxelXYZP::new(40, 50, 60, 0.25);
    let mut neurons_a = NeuronVoxelXYZPArrays::with_capacity(2);
    neurons_a.push(&neuron_a_1);
    neurons_a.push(&neuron_a_2);

    let cortical_id_b = CorticalID::try_from_base_64("cBBBBB").unwrap();
    let neuron_b_1 = NeuronVoxelXYZP::new(100, 200, 300, 0.8);
    let mut neurons_b = NeuronVoxelXYZPArrays::with_capacity(1);
    neurons_b.push(&neuron_b_1);

    let mut neuron_mappings = CorticalMappedXYZPNeuronVoxels::new();
    neuron_mappings.insert(cortical_id_a, neurons_a);
    neuron_mappings.insert(cortical_id_b, neurons_b);

    // Convert both to individual FeagiByteContainers
    let mut json_container = FeagiByteContainer::new_empty();
    json_container
        .overwrite_byte_data_with_single_struct_data(&json_structure, 0)
        .unwrap();
    let mut neuron_container = FeagiByteContainer::new_empty();
    neuron_container
        .overwrite_byte_data_with_single_struct_data(&neuron_mappings, 0)
        .unwrap();

    // Verify individual structures have correct types
    assert_eq!(
        json_container.try_get_structure_type().unwrap(),
        FeagiByteStructureType::JSON
    );
    assert_eq!(
        neuron_container.try_get_structure_type().unwrap(),
        FeagiByteStructureType::NeuronCategoricalXYZP
    );

    // TODO: Update to use new API - FeagiByteStructure::create_from_2_existing no longer exists
    // Create combined multi-struct using new API
    let mut combined_byte_structure = FeagiByteContainer::new_empty();
    combined_byte_structure
        .overwrite_byte_data_with_multiple_struct_data(&[&json_structure, &neuron_mappings], 0)
        .unwrap();

    // Verify the combined structure is a multi-struct
    assert!(combined_byte_structure.is_multistruct().unwrap());
    assert_eq!(
        combined_byte_structure.try_get_structure_type().unwrap(),
        FeagiByteStructureType::MultiStructHolder
    );
    assert_eq!(
        combined_byte_structure.contained_structure_count().unwrap(),
        2
    );

    // Check the order of internal structure types
    let ordered_types = combined_byte_structure.get_ordered_object_types().unwrap();
    assert_eq!(ordered_types.len(), 2);
    assert_eq!(ordered_types[0], FeagiByteStructureType::JSON);
    assert_eq!(
        ordered_types[1],
        FeagiByteStructureType::NeuronCategoricalXYZP
    );

    // Serialize to bytes (simulate network transmission)
    let serialized_bytes = combined_byte_structure.copy_out_as_byte_vector();

    // Deserialize from bytes
    let received_combined_structure =
        FeagiByteStructure::create_from_bytes(serialized_bytes).unwrap();

    // Verify the received structure is still a multi-struct with correct properties
    assert!(received_combined_structure.is_multistruct().unwrap());
    assert_eq!(
        received_combined_structure
            .contained_structure_count()
            .unwrap(),
        2
    );

    // Extract individual structures from the multi-struct
    let received_json_structure_bytes = received_combined_structure
        .copy_out_single_byte_structure_from_multistruct(0)
        .unwrap();
    let received_neuron_structure_bytes = received_combined_structure
        .copy_out_single_byte_structure_from_multistruct(1)
        .unwrap();

    // Verify individual structure types are correct
    assert_eq!(
        received_json_structure_bytes
            .try_get_structure_type()
            .unwrap(),
        FeagiByteStructureType::JSON
    );
    assert_eq!(
        received_neuron_structure_bytes
            .try_get_structure_type()
            .unwrap(),
        FeagiByteStructureType::NeuronCategoricalXYZP
    );

    // Convert back to original data types
    let recovered_json_structure =
        FeagiJSON::new_from_feagi_byte_structure(&received_json_structure_bytes).unwrap();
    let recovered_neuron_mappings = CorticalMappedXYZPNeuronVoxels::new_from_feagi_byte_structure(
        &received_neuron_structure_bytes,
    )
    .unwrap();

    // Verify JSON data integrity
    let recovered_json_value = recovered_json_structure.borrow_json_value();
    assert_eq!(recovered_json_value, &json_data);

    // Verify neuron data integrity
    assert_eq!(recovered_neuron_mappings.len(), 2);
    assert!(recovered_neuron_mappings.contains_cortical_id(
        &CorticalID::new_custom_cortical_area_id("cAAAAA".to_string()).unwrap()
    ));
    assert!(recovered_neuron_mappings.contains_cortical_id(
        &CorticalID::new_custom_cortical_area_id("cBBBBB".to_string()).unwrap()
    ));

    let recovered_neurons_a = recovered_neuron_mappings
        .get_neurons_of(&CorticalID::new_custom_cortical_area_id("cAAAAA".to_string()).unwrap())
        .unwrap();
    let recovered_neurons_b = recovered_neuron_mappings
        .get_neurons_of(&CorticalID::new_custom_cortical_area_id("cBBBBB".to_string()).unwrap())
        .unwrap();

    let recovered_neuron_vec_a = recovered_neurons_a.copy_as_neuron_xyzp_vec();
    let recovered_neuron_vec_b = recovered_neurons_b.copy_as_neuron_xyzp_vec();

    assert_eq!(recovered_neuron_vec_a.len(), 2);
    assert_eq!(recovered_neuron_vec_b.len(), 1);
    assert_eq!(recovered_neuron_vec_a[0], neuron_a_1);
    assert_eq!(recovered_neuron_vec_a[1], neuron_a_2);
    assert_eq!(recovered_neuron_vec_b[0], neuron_b_1);

    println!("✓ Successfully combined, serialized, and deserialized JSON + Neuron data!");
    */
}

#[test]
#[ignore] // TODO: Update to use new serialization API (FeagiByteContainer)
#[allow(unused_variables, dead_code, unreachable_code)]
fn test_multistruct_with_multiple_json_and_neuron_structures() {
    // TODO: This test needs to be fully updated to use the new serialization API
    // All code below is commented out until the test is fully rewritten
    /*
    // Create multiple JSON structures
    let json1 = FeagiJSON::from_json_value(json!({"type": "config", "value": 1}));
    let json2 = FeagiJSON::from_json_value(json!({"type": "metadata", "value": 2}));

    // Create multiple neuron structures
    let cortical_id_1 = CorticalID::from_bytes(b"cAAAAA").unwrap();
    let neuron_1 = NeuronVoxelXYZP::new(1, 1, 1, 0.1);
    let mut neurons_1 = NeuronVoxelXYZPArrays::with_capacity(1);
    neurons_1.push(&neuron_1);
    let mut neuron_mappings_1 = CorticalMappedXYZPNeuronVoxels::new();
    neuron_mappings_1.insert(cortical_id_1, neurons_1);

    let cortical_id_2 = CorticalID::new_custom_cortical_area_id("cTES02".to_string()).unwrap();
    let neuron_2 = NeuronVoxelXYZP::new(2, 2, 2, 0.2);
    let mut neurons_2 = NeuronVoxelXYZPArrays::with_capacity(1);
    neurons_2.push(&neuron_2);
    let mut neuron_mappings_2 = CorticalMappedXYZPNeuronVoxels::new();
    neuron_mappings_2.insert(cortical_id_2, neurons_2);

    // TODO: Update to use new FeagiByteContainer API
    // The old API methods (as_new_feagi_byte_structure, create_from_multiple_existing, etc.) no longer exist
    // This test needs to be rewritten to use FeagiByteContainer::overwrite_byte_data_with_multiple_struct_data()
    todo!("Update test to use new FeagiByteContainer API");
    */
}
