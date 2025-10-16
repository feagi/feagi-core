// TODO we need some example byte arrays to write some more tests!

use feagi_data_serialization::{FeagiByteContainer, FeagiByteStructureType};
use feagi_data_structures::genomic::CorticalID;
use feagi_data_structures::genomic::descriptors::CorticalDimensions;
use feagi_data_structures::neuron_voxels::xyzp::{CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZPArrays};

fn sample_cortical_mapped_neurons(dimensions: CorticalDimensions, cortical_id: CorticalID) -> CorticalMappedXYZPNeuronVoxels {
    let mut neurons = CorticalMappedXYZPNeuronVoxels::new();
    let mut neuron_array = NeuronVoxelXYZPArrays::with_capacity(100);
    for i in 0..dimensions.number_elements() {
        neuron_array.push_raw(
            i % dimensions.width,
            i % dimensions.height,
            i % dimensions.depth,
            (i as f32) / (dimensions.number_elements() as f32),
        );
    };
    neurons.insert(cortical_id, neuron_array);
    neurons
}


#[test]
fn test_byte_container_overwrite_with_struct() {
    let source_neurons = sample_cortical_mapped_neurons(
        CorticalDimensions::new(3, 4, 5).unwrap(),
        CorticalID::new_custom_cortical_area_id("c_lmao".into()).unwrap()
    );
    let mut byte_container = FeagiByteContainer::new_empty();
    byte_container.overwrite_byte_data_with_single_struct_data(&source_neurons, 0).unwrap();
    let destination_neurons: CorticalMappedXYZPNeuronVoxels = byte_container.try_create_new_struct_from_index(0).unwrap().try_into().unwrap();
    assert_eq!(source_neurons, destination_neurons);
}

#[test]
fn test_byte_container_overwrite_bytes() {
    let source_neurons = sample_cortical_mapped_neurons(
        CorticalDimensions::new(3, 4, 5).unwrap(),
        CorticalID::new_custom_cortical_area_id("c_lmao".into()).unwrap()
    );
    let mut byte_container = FeagiByteContainer::new_empty();
    let empty_bytes = byte_container.get_byte_ref().to_vec();
    let empty_bytes_len = empty_bytes.len();
    assert_eq!(empty_bytes_len, FeagiByteContainer::GLOBAL_BYTE_HEADER_BYTE_COUNT); // This should be the global header only
    byte_container.overwrite_byte_data_with_single_struct_data(&source_neurons, 0).unwrap();
    let neuron_bytes = byte_container.get_byte_ref().to_vec();

    byte_container.try_write_data_by_copy_and_verify(&empty_bytes).unwrap(); // reset to empty (but not deallocate)
    assert_eq!(byte_container.get_number_of_bytes_allocated(), neuron_bytes.len()); // We shouldnt have freed anything
    assert_eq!(&empty_bytes, byte_container.get_byte_ref()); // but these should match


    byte_container.try_write_data_by_ownership_to_container_and_verify(empty_bytes).unwrap(); // Now we take ownership, allocation should shrink
    assert_eq!(byte_container.get_number_of_bytes_allocated(), empty_bytes_len);

    byte_container.try_write_data_by_copy_and_verify(&neuron_bytes).unwrap(); // This should force the allocation to expand
    assert_eq!(byte_container.get_number_of_bytes_allocated(), neuron_bytes.len());

    // lets decode back to neurons
    let decoded_neurons: CorticalMappedXYZPNeuronVoxels = byte_container.try_create_struct_from_first_found_struct_of_type(FeagiByteStructureType::NeuronCategoricalXYZP).unwrap().unwrap().try_into().unwrap();
    assert_eq!(decoded_neurons, source_neurons);
}

#[test]
fn test_byte_container_progressive_memory_allocation() {
    let mut byte_container = FeagiByteContainer::new_empty();
    let initial_allocation = byte_container.get_number_of_bytes_allocated();
    let cortical_id = CorticalID::new_custom_cortical_area_id("c_test".into()).unwrap();
    
    let mut previous_allocation = initial_allocation;
    let mut previous_bytes_used = byte_container.get_number_of_bytes_used();
    
    // Iterate through progressively larger neuron structures
    let iteration_count = 100;
    for iteration in 0..iteration_count {
        // Exponentially increase dimensions each iteration
        let dimension_size = iteration * iteration + 1;
        let dimensions = CorticalDimensions::new(dimension_size as u32, dimension_size as u32, 1).unwrap();
        let total_neurons = dimensions.number_elements();
        
        // Create neurons with increasing size
        let neurons = sample_cortical_mapped_neurons(dimensions, cortical_id.clone());
        
        // Serialize into the byte container
        byte_container.overwrite_byte_data_with_single_struct_data(&neurons, iteration as u16).unwrap();
        
        // Track memory metrics
        let current_allocation = byte_container.get_number_of_bytes_allocated();
        let current_bytes_used = byte_container.get_number_of_bytes_used();
        
        // Verify the container is valid
        assert!(byte_container.is_valid());
        assert_eq!(byte_container.try_get_number_contained_structures().unwrap(), 1);
        assert_eq!(byte_container.get_increment_counter().unwrap(), iteration as u16);
        
        // Verify bytes used increases with more neurons
        assert!(current_bytes_used > previous_bytes_used, 
            "Iteration {}: bytes used ({}) should be greater than previous ({})", 
            iteration, current_bytes_used, previous_bytes_used);
        
        // Verify allocation grows when needed (may not grow every iteration due to capacity)
        assert!(current_allocation >= previous_allocation,
            "Iteration {}: allocation ({}) should not shrink from previous ({})",
            iteration, current_allocation, previous_allocation);
        
        // Verify we can deserialize correctly
        let decoded_neurons: CorticalMappedXYZPNeuronVoxels = byte_container
            .try_create_new_struct_from_index(0)
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(decoded_neurons, neurons, "Iteration {}: decoded neurons should match source", iteration);
        
        // Verify the decoded structure has the expected number of neurons
        let decoded_neuron_count = decoded_neurons.get_neurons_of(&cortical_id).unwrap().len();
        assert_eq!(decoded_neuron_count, total_neurons as usize,
            "Iteration {}: expected {} neurons, got {}", 
            iteration, total_neurons, decoded_neuron_count);
        
        println!("Iteration {}: {} neurons, {} bytes used, {} bytes allocated", 
                 iteration, total_neurons, current_bytes_used, current_allocation);
        
        previous_allocation = current_allocation;
        previous_bytes_used = current_bytes_used;
    }
    
    // Verify final allocation is significantly larger than initial
    let final_allocation = byte_container.get_number_of_bytes_allocated();
    assert!(final_allocation > initial_allocation * 100,
        "Final allocation ({}) should be much larger than initial ({})",
        final_allocation, initial_allocation);
    
    // Test freeing unused allocation
    byte_container.free_unused_allocation();
    let freed_allocation = byte_container.get_number_of_bytes_allocated();
    assert_eq!(freed_allocation, byte_container.get_number_of_bytes_used(),
        "After freeing, allocation should equal bytes used");
}
