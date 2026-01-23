// NOTE: These tests use the old serialization API. They need to be updated to use:
// - FeagiByteContainer instead of FeagiByteStructure
// - FeagiSerializable trait instead of FeagiByteStructureCompatible
// TODO: Update tests to use new serialization API
use feagi_serialization::{FeagiByteContainer, FeagiByteStructureType};
use feagi_structures::genomic::cortical_area::CoreCorticalType;
use feagi_structures::neuron_voxels::xyzp::{
    CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZP, NeuronVoxelXYZPArrays,
};
use feagi_structures::FeagiJSON;
use serde_json::json;

#[test]
fn test_combined_neuron_json_multistruct_serialize_deserialize() {
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
    let cortical_id_a = CoreCorticalType::Power.to_cortical_id();

    let neuron_a_1 = NeuronVoxelXYZP::new(10, 20, 30, 0.75);
    let neuron_a_2 = NeuronVoxelXYZP::new(40, 50, 60, 0.25);
    let mut neurons_a = NeuronVoxelXYZPArrays::with_capacity(2);
    neurons_a.push(&neuron_a_1);
    neurons_a.push(&neuron_a_2);

    let cortical_id_b = CoreCorticalType::Death.to_cortical_id();
    let neuron_b_1 = NeuronVoxelXYZP::new(100, 200, 300, 0.8);
    let mut neurons_b = NeuronVoxelXYZPArrays::with_capacity(1);
    neurons_b.push(&neuron_b_1);

    let mut neuron_mappings = CorticalMappedXYZPNeuronVoxels::new();
    neuron_mappings.insert(cortical_id_a, neurons_a);
    neuron_mappings.insert(cortical_id_b, neurons_b);

    // Create combined multi-struct using new API
    let mut combined_byte_structure = FeagiByteContainer::new_empty();
    combined_byte_structure
        .overwrite_byte_data_with_multiple_struct_data(vec![&json_structure, &neuron_mappings], 0)
        .unwrap();

    assert_eq!(
        combined_byte_structure
            .try_get_number_contained_structures()
            .unwrap(),
        2
    );

    // Check the order of internal structure types
    let ordered_types = combined_byte_structure.get_contained_struct_types();
    assert_eq!(ordered_types.len(), 2);
    assert_eq!(ordered_types[0], FeagiByteStructureType::JSON);
    assert_eq!(
        ordered_types[1],
        FeagiByteStructureType::NeuronCategoricalXYZP
    );

    // Serialize to bytes (simulate network transmission)
    let serialized_bytes = combined_byte_structure.get_byte_ref().to_vec();

    // Deserialize from bytes
    let mut received_combined_structure = FeagiByteContainer::new_empty();
    received_combined_structure
        .try_write_data_by_ownership_to_container_and_verify(serialized_bytes)
        .unwrap();

    // Verify the received structure is still a multi-struct with correct properties
    assert_eq!(
        received_combined_structure
            .try_get_number_contained_structures()
            .unwrap(),
        2
    );

    // Extract individual structures from the multi-struct
    let received_json_structure_bytes = received_combined_structure
        .try_create_new_struct_from_index(0)
        .unwrap();
    let received_neuron_structure_bytes = received_combined_structure
        .try_create_new_struct_from_index(1)
        .unwrap();

    // Convert back to original data types
    let recovered_json_structure = FeagiJSON::try_from(received_json_structure_bytes).unwrap();
    let recovered_neuron_mappings =
        CorticalMappedXYZPNeuronVoxels::try_from(received_neuron_structure_bytes).unwrap();

    // Verify JSON data integrity
    let recovered_json_value = recovered_json_structure.borrow_json_value();
    assert_eq!(recovered_json_value, &json_data);

    // Verify neuron data integrity
    assert_eq!(recovered_neuron_mappings.len(), 2);
    assert!(recovered_neuron_mappings.contains_cortical_id(&cortical_id_a));
    assert!(recovered_neuron_mappings.contains_cortical_id(&cortical_id_b));

    let recovered_neurons_a = recovered_neuron_mappings
        .get_neurons_of(&cortical_id_a)
        .unwrap();
    let recovered_neurons_b = recovered_neuron_mappings
        .get_neurons_of(&cortical_id_b)
        .unwrap();

    let recovered_neuron_vec_a = recovered_neurons_a.copy_as_neuron_xyzp_vec();
    let recovered_neuron_vec_b = recovered_neurons_b.copy_as_neuron_xyzp_vec();

    assert_eq!(recovered_neuron_vec_a.len(), 2);
    assert_eq!(recovered_neuron_vec_b.len(), 1);
    assert_eq!(recovered_neuron_vec_a[0], neuron_a_1);
    assert_eq!(recovered_neuron_vec_a[1], neuron_a_2);
    assert_eq!(recovered_neuron_vec_b[0], neuron_b_1);

    println!("Successfully combined, serialized, and deserialized JSON and neuron data.");
}

#[test]
fn test_multistruct_with_multiple_json_and_neuron_structures() {
    // Create multiple JSON structures
    let json1 = FeagiJSON::from_json_value(json!({"type": "config", "value": 1}));
    let json2 = FeagiJSON::from_json_value(json!({"type": "metadata", "value": 2}));

    // Create multiple neuron structures
    let cortical_id_1 = CoreCorticalType::Power.to_cortical_id();
    let neuron_1 = NeuronVoxelXYZP::new(1, 1, 1, 0.1);
    let mut neurons_1 = NeuronVoxelXYZPArrays::with_capacity(1);
    neurons_1.push(&neuron_1);
    let mut neuron_mappings_1 = CorticalMappedXYZPNeuronVoxels::new();
    neuron_mappings_1.insert(cortical_id_1, neurons_1);

    let cortical_id_2 = CoreCorticalType::Death.to_cortical_id();
    let neuron_2 = NeuronVoxelXYZP::new(2, 2, 2, 0.2);
    let mut neurons_2 = NeuronVoxelXYZPArrays::with_capacity(1);
    neurons_2.push(&neuron_2);
    let mut neuron_mappings_2 = CorticalMappedXYZPNeuronVoxels::new();
    neuron_mappings_2.insert(cortical_id_2, neurons_2);

    let mut combined_container = FeagiByteContainer::new_empty();
    combined_container
        .overwrite_byte_data_with_multiple_struct_data(
            vec![&json1, &json2, &neuron_mappings_1, &neuron_mappings_2],
            0,
        )
        .unwrap();

    assert_eq!(
        combined_container
            .try_get_number_contained_structures()
            .unwrap(),
        4
    );
    let contained_types = combined_container.get_contained_struct_types();
    assert_eq!(
        contained_types,
        vec![
            FeagiByteStructureType::JSON,
            FeagiByteStructureType::JSON,
            FeagiByteStructureType::NeuronCategoricalXYZP,
            FeagiByteStructureType::NeuronCategoricalXYZP
        ]
    );

    let bytes = combined_container.get_byte_ref().to_vec();
    let mut received_container = FeagiByteContainer::new_empty();
    received_container
        .try_write_data_by_ownership_to_container_and_verify(bytes)
        .unwrap();

    let json_boxed = received_container
        .try_create_new_struct_from_index(0)
        .unwrap();
    let recovered_json = FeagiJSON::try_from(json_boxed).unwrap();
    assert_eq!(
        recovered_json.borrow_json_value(),
        json1.borrow_json_value()
    );

    let neuron_boxed = received_container
        .try_create_new_struct_from_index(2)
        .unwrap();
    let recovered_neurons = CorticalMappedXYZPNeuronVoxels::try_from(neuron_boxed).unwrap();
    assert!(recovered_neurons.contains_cortical_id(&cortical_id_1));
}
