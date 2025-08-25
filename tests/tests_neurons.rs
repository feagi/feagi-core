use ndarray::prelude::*;
use feagi_core_data_structures_and_processing::neuron_data::xyzp::{CorticalMappedXYZPNeuronData, NeuronXYZPArrays, NeuronXYZP};
use feagi_core_data_structures_and_processing::genomic_structures::CorticalID;
use feagi_core_data_structures_and_processing::io_processing::byte_structures::FeagiByteStructure;
use feagi_core_data_structures_and_processing::io_processing::byte_structures::FeagiByteStructureCompatible;

#[test]
fn test_minimal_memory_corruption_debug() {
    // Create a simple test case
    let cortical_id = CorticalID::new_custom_cortical_area_id("cAAAAA".to_string()).unwrap();
    let neuron = NeuronXYZP::new(1, 2, 3, 0.5);
    let mut neurons = NeuronXYZPArrays::with_capacity(1);
    neurons.push(&neuron);

    let mut cortical_mappings = CorticalMappedXYZPNeuronData::new();
    cortical_mappings.insert(cortical_id, neurons);

    // Test 1: Check if max_number_bytes_needed is consistent
    let size1 = cortical_mappings.max_number_bytes_needed();
    let size2 = cortical_mappings.max_number_bytes_needed();
    println!("Size check: {} == {}", size1, size2);
    assert_eq!(size1, size2);

    // Test 2: Create a manual bytes vector and serialize to it
    let mut manual_bytes = vec![0u8; size1];
    println!("Manual bytes before serialization: {:?}", &manual_bytes[0..4.min(manual_bytes.len())]);

    let result = cortical_mappings.overwrite_feagi_byte_structure_slice(&mut manual_bytes);
    println!("Serialization result: {:?}", result);
    println!("Manual bytes after serialization: {:?}", &manual_bytes[0..4.min(manual_bytes.len())]);

    // Test 3: Create FeagiByteStructure and immediately check
    let structure = FeagiByteStructure::create_from_bytes(manual_bytes.clone()).unwrap();
    let slice_view = structure.borrow_data_as_slice();
    println!("Slice view: {:?}", &slice_view[0..4.min(slice_view.len())]);

    // Test 4: Clone the slice reference
    let cloned_from_slice = slice_view.to_vec();
    println!("Cloned from slice: {:?}", &cloned_from_slice[0..4.min(cloned_from_slice.len())]);

    // Test 5: Use the new copy method
    let copied_vector = structure.copy_out_as_byte_vector();
    println!("Copied vector: {:?}", &copied_vector[0..4.min(copied_vector.len())]);

    // Check if they're all the same
    assert_eq!(manual_bytes[0..4], cloned_from_slice[0..4]);
    assert_eq!(manual_bytes[0..4], copied_vector[0..4]);
}

#[test]
fn test_serialize_deserialize_neuron_mapped_areas() {

    // cortical area A
    let cortical_id_a = CorticalID::new_custom_cortical_area_id("cAAAAA".to_string()).unwrap();
    let neuron_a_1 = NeuronXYZP::new(1, 2, 3, 0.5);
    let neuron_a_2 = NeuronXYZP::new(4, 5, 7, 0.2);
    let mut neurons_a = NeuronXYZPArrays::with_capacity(2); // lets preallocate
    neurons_a.push(&neuron_a_1);
    neurons_a.push(&neuron_a_2);


    // cortical area b
    let cortical_id_b = CorticalID::new_custom_cortical_area_id("cBBBBB".to_string()).unwrap();
    let neuron_b_1 = NeuronXYZP::new(8, 9, 10, 0.5);
    let neuron_b_2 = NeuronXYZP::new(11, 12, 13, 0.2);
    let mut neurons_b = NeuronXYZPArrays::with_capacity(1); // incorrect preallocation (system should grow)
    neurons_b.push(&neuron_b_1);
    neurons_b.push(&neuron_b_2);

    assert_eq!(
        neurons_a.len(),
        neurons_b.len()
    );

    // lets add cortical are C using arrays
    let cortical_id_c = CorticalID::new_custom_cortical_area_id("cCCCCC".to_string()).unwrap();
    let neurons_c_x = array![1,2,3];
    let neurons_c_y = array![4,5,6];
    let neurons_c_z = array![7,8,9];
    let neurons_c_p: Array::<f32, Ix1>  = array![0.1,0.2,0.3];
    let neurons_c = NeuronXYZPArrays::new_from_ndarrays(neurons_c_x,
                                                            neurons_c_y, neurons_c_z, neurons_c_p).unwrap();


    // cortical mappings
    let mut cortical_mappings = CorticalMappedXYZPNeuronData::new();
    cortical_mappings.insert(cortical_id_a, neurons_a);
    cortical_mappings.insert(cortical_id_b, neurons_b);
    cortical_mappings.insert(cortical_id_c, neurons_c);

    // bytes data serialization
    let sending_byte_structure = cortical_mappings.as_new_feagi_byte_structure().unwrap();
    let bytes = sending_byte_structure.copy_out_as_byte_vector(); // raw bytes
    
    // deserialize (lets pretend 'bytes' was sent over the network)
    let received_byte_structure = FeagiByteStructure::create_from_bytes(bytes).unwrap();
    let received_cortical_mappings = CorticalMappedXYZPNeuronData::new_from_feagi_byte_structure(&received_byte_structure).unwrap();

    assert_eq!(received_cortical_mappings.len(), 3);
    assert!(received_cortical_mappings.contains_cortical_id(&CorticalID::new_custom_cortical_area_id("cAAAAA".to_string()).unwrap()));
    assert!(received_cortical_mappings.contains_cortical_id(&CorticalID::new_custom_cortical_area_id("cBBBBB".to_string()).unwrap()));

    let rec_neurons_a = received_cortical_mappings.get_neurons_of(&CorticalID::new_custom_cortical_area_id("cAAAAA".to_string()).unwrap()).unwrap();
    let rec_neurons_b = received_cortical_mappings.get_neurons_of(&CorticalID::new_custom_cortical_area_id("cBBBBB".to_string()).unwrap()).unwrap();

    let rec_neuron_1_a = rec_neurons_a.copy_as_neuron_xyzp_vec()[0].clone();
    let rec_neuron_2_b = rec_neurons_b.copy_as_neuron_xyzp_vec()[1].clone();

    assert_eq!(rec_neuron_1_a, neuron_a_1);
    assert_eq!(rec_neuron_2_b, neuron_b_2);

}

