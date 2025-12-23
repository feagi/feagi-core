//! Tests for FEAGI data serialization
//!
//! This module tests the serialization and deserialization of neuron data
//! using the FeagiByteContainer format.

use feagi_serialization::{FeagiByteContainer, FeagiByteStructureType};
use feagi_structures::genomic::cortical_area::descriptors::{
    CorticalAreaDimensions, CorticalUnitIndex,
};
use feagi_structures::genomic::cortical_area::io_cortical_area_data_type::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use feagi_structures::genomic::cortical_area::{CoreCorticalType, CorticalID};
use feagi_structures::genomic::SensoryCorticalUnit;
use feagi_structures::neuron_voxels::xyzp::{
    CorticalMappedXYZPNeuronVoxels, NeuronVoxelXYZPArrays,
};

fn sample_cortical_mapped_neurons(
    dimensions: CorticalAreaDimensions,
    cortical_id: CorticalID,
) -> CorticalMappedXYZPNeuronVoxels {
    let mut neurons = CorticalMappedXYZPNeuronVoxels::new();
    let mut neuron_array = NeuronVoxelXYZPArrays::with_capacity(100);
    for i in 0..dimensions.number_elements() {
        neuron_array.push_raw(
            i % dimensions.width,
            i % dimensions.height,
            i % dimensions.depth,
            (i as f32) / (dimensions.number_elements() as f32),
        );
    }
    neurons.insert(cortical_id, neuron_array);
    neurons
}

#[test]
fn test_byte_container_overwrite_with_struct() {
    let source_neurons = sample_cortical_mapped_neurons(
        CorticalAreaDimensions::new(3, 4, 5).unwrap(),
        CoreCorticalType::Power.to_cortical_id(),
    );
    let mut byte_container = FeagiByteContainer::new_empty();
    byte_container
        .overwrite_byte_data_with_single_struct_data(&source_neurons, 0)
        .unwrap();
    let destination_neurons: CorticalMappedXYZPNeuronVoxels = byte_container
        .try_create_new_struct_from_index(0)
        .unwrap()
        .try_into()
        .unwrap();
    assert_eq!(source_neurons, destination_neurons);
}

#[test]
fn test_byte_container_overwrite_bytes() {
    let source_neurons = sample_cortical_mapped_neurons(
        CorticalAreaDimensions::new(3, 4, 5).unwrap(),
        CoreCorticalType::Death.to_cortical_id(),
    );
    let mut byte_container = FeagiByteContainer::new_empty();
    let empty_bytes = byte_container.get_byte_ref().to_vec();
    let empty_bytes_len = empty_bytes.len();
    assert_eq!(
        empty_bytes_len,
        FeagiByteContainer::GLOBAL_BYTE_HEADER_BYTE_COUNT
    ); // This should be the global header only
    byte_container
        .overwrite_byte_data_with_single_struct_data(&source_neurons, 0)
        .unwrap();
    let neuron_bytes = byte_container.get_byte_ref().to_vec();

    byte_container
        .try_write_data_by_copy_and_verify(&empty_bytes)
        .unwrap(); // reset to empty (but not deallocate)
    assert_eq!(
        byte_container.get_number_of_bytes_allocated(),
        neuron_bytes.len()
    ); // We shouldnt have freed anything
    assert_eq!(&empty_bytes, byte_container.get_byte_ref()); // but these should match

    byte_container
        .try_write_data_by_ownership_to_container_and_verify(empty_bytes)
        .unwrap(); // Now we take ownership, allocation should shrink
    assert_eq!(
        byte_container.get_number_of_bytes_allocated(),
        empty_bytes_len
    );

    byte_container
        .try_write_data_by_copy_and_verify(&neuron_bytes)
        .unwrap(); // This should force the allocation to expand
    assert_eq!(
        byte_container.get_number_of_bytes_allocated(),
        neuron_bytes.len()
    );

    // lets decode back to neurons
    let decoded_neurons: CorticalMappedXYZPNeuronVoxels = byte_container
        .try_create_struct_from_first_found_struct_of_type(
            FeagiByteStructureType::NeuronCategoricalXYZP,
        )
        .unwrap()
        .unwrap()
        .try_into()
        .unwrap();
    assert_eq!(decoded_neurons, source_neurons);
}

#[test]
fn test_byte_container_progressive_memory_allocation() {
    let mut byte_container = FeagiByteContainer::new_empty();
    let initial_allocation = byte_container.get_number_of_bytes_allocated();
    let cortical_id = CoreCorticalType::Power.to_cortical_id();

    let mut previous_allocation = initial_allocation;
    let mut previous_bytes_used = byte_container.get_number_of_bytes_used();

    // Iterate through progressively larger neuron structures
    let iteration_count = 20;
    for iteration in 0..iteration_count {
        // Exponentially increase dimensions each iteration
        let dimension_size = iteration * iteration + 1;
        dbg!(iteration);
        let dimensions =
            CorticalAreaDimensions::new(dimension_size as u32, dimension_size as u32, 1).unwrap();
        let total_neurons = dimensions.number_elements();

        // Create neurons with increasing size
        let neurons = sample_cortical_mapped_neurons(dimensions, cortical_id);

        // Serialize into the byte container
        byte_container
            .overwrite_byte_data_with_single_struct_data(&neurons, iteration as u16)
            .unwrap();

        // Track memory metrics
        let current_allocation = byte_container.get_number_of_bytes_allocated();
        let current_bytes_used = byte_container.get_number_of_bytes_used();

        // Verify the container is valid
        assert!(byte_container.is_valid());
        assert_eq!(
            byte_container
                .try_get_number_contained_structures()
                .unwrap(),
            1
        );
        assert_eq!(
            byte_container.get_increment_counter().unwrap(),
            iteration as u16
        );

        // Verify bytes used increases with more neurons
        assert!(
            current_bytes_used > previous_bytes_used,
            "Iteration {}: bytes used ({}) should be greater than previous ({})",
            iteration,
            current_bytes_used,
            previous_bytes_used
        );

        // Verify allocation grows when needed (may not grow every iteration due to capacity)
        assert!(
            current_allocation >= previous_allocation,
            "Iteration {}: allocation ({}) should not shrink from previous ({})",
            iteration,
            current_allocation,
            previous_allocation
        );

        // Verify we can deserialize correctly
        let decoded_neurons: CorticalMappedXYZPNeuronVoxels = byte_container
            .try_create_new_struct_from_index(0)
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(
            decoded_neurons, neurons,
            "Iteration {}: decoded neurons should match source",
            iteration
        );

        // Verify the decoded structure has the expected number of neurons
        let decoded_neuron_count = decoded_neurons.get_neurons_of(&cortical_id).unwrap().len();
        assert_eq!(
            decoded_neuron_count, total_neurons as usize,
            "Iteration {}: expected {} neurons, got {}",
            iteration, total_neurons, decoded_neuron_count
        );

        println!(
            "Iteration {}: {} neurons, {} bytes used, {} bytes allocated",
            iteration, total_neurons, current_bytes_used, current_allocation
        );

        previous_allocation = current_allocation;
        previous_bytes_used = current_bytes_used;
    }

    // Verify final allocation is significantly larger than initial
    let final_allocation = byte_container.get_number_of_bytes_allocated();
    assert!(
        final_allocation > initial_allocation * 100,
        "Final allocation ({}) should be much larger than initial ({})",
        final_allocation,
        initial_allocation
    );

    // Test freeing unused allocation
    byte_container.free_unused_allocation();
    let freed_allocation = byte_container.get_number_of_bytes_allocated();
    assert_eq!(
        freed_allocation,
        byte_container.get_number_of_bytes_used(),
        "After freeing, allocation should equal bytes used"
    );
}

#[test]
fn test_byte_container_with_sensory_cortical_id() {
    // Test serialization with sensory cortical unit IDs
    let cortical_id = SensoryCorticalUnit::get_cortical_ids_array_for_infrared(
        FrameChangeHandling::Absolute,
        PercentageNeuronPositioning::Linear,
        CorticalUnitIndex::from(0u8),
    )[0];

    let dimensions = CorticalAreaDimensions::new(5, 5, 3).unwrap();
    let source_neurons = sample_cortical_mapped_neurons(dimensions, cortical_id);

    let mut byte_container = FeagiByteContainer::new_empty();
    byte_container
        .overwrite_byte_data_with_single_struct_data(&source_neurons, 0)
        .unwrap();

    // Verify deserialization works correctly
    let decoded_neurons: CorticalMappedXYZPNeuronVoxels = byte_container
        .try_create_new_struct_from_index(0)
        .unwrap()
        .try_into()
        .unwrap();

    assert_eq!(decoded_neurons, source_neurons);
    assert_eq!(
        decoded_neurons.get_neurons_of(&cortical_id).unwrap().len(),
        75
    ); // 5 * 5 * 3
}

#[test]
fn test_byte_container_with_segmented_vision() {
    // Test with a more complex sensor that has multiple cortical areas
    let cortical_ids = SensoryCorticalUnit::get_cortical_ids_array_for_segmented_vision(
        FrameChangeHandling::Incremental,
        CorticalUnitIndex::from(2u8),
    );

    let dimensions = CorticalAreaDimensions::new(4, 4, 2).unwrap();
    let mut neurons = CorticalMappedXYZPNeuronVoxels::new();

    // Add neurons for the center segment (first ID)
    let mut neuron_array = NeuronVoxelXYZPArrays::with_capacity(32);
    for i in 0..dimensions.number_elements() {
        neuron_array.push_raw(
            i % dimensions.width,
            i % dimensions.height,
            i % dimensions.depth,
            (i as f32) / (dimensions.number_elements() as f32),
        );
    }
    neurons.insert(cortical_ids[0], neuron_array);

    let mut byte_container = FeagiByteContainer::new_empty();
    byte_container
        .overwrite_byte_data_with_single_struct_data(&neurons, 0)
        .unwrap();

    let decoded_neurons: CorticalMappedXYZPNeuronVoxels = byte_container
        .try_create_new_struct_from_index(0)
        .unwrap()
        .try_into()
        .unwrap();

    assert_eq!(decoded_neurons, neurons);
}

#[test]
fn test_empty_byte_container() {
    let container = FeagiByteContainer::new_empty();

    assert!(container.is_valid());
    assert_eq!(container.try_get_number_contained_structures().unwrap(), 0);
    assert_eq!(
        container.get_number_of_bytes_used(),
        FeagiByteContainer::GLOBAL_BYTE_HEADER_BYTE_COUNT
    );
    assert!(container.get_contained_struct_types().is_empty());
}

#[test]
fn test_byte_container_multiple_core_types() {
    // Test with multiple different core cortical types
    let power_id = CoreCorticalType::Power.to_cortical_id();
    let death_id = CoreCorticalType::Death.to_cortical_id();

    let _dimensions = CorticalAreaDimensions::new(2, 2, 2).unwrap();

    let mut neurons = CorticalMappedXYZPNeuronVoxels::new();

    // Add neurons for power cortical area
    let mut power_array = NeuronVoxelXYZPArrays::new();
    for i in 0..4 {
        power_array.push_raw(i % 2, i / 2, 0, 0.5);
    }
    neurons.insert(power_id, power_array);

    // Add neurons for death cortical area
    let mut death_array = NeuronVoxelXYZPArrays::new();
    for i in 0..4 {
        death_array.push_raw(i % 2, i / 2, 1, 0.7);
    }
    neurons.insert(death_id, death_array);

    let mut byte_container = FeagiByteContainer::new_empty();
    byte_container
        .overwrite_byte_data_with_single_struct_data(&neurons, 0)
        .unwrap();

    let decoded_neurons: CorticalMappedXYZPNeuronVoxels = byte_container
        .try_create_new_struct_from_index(0)
        .unwrap()
        .try_into()
        .unwrap();

    assert_eq!(decoded_neurons, neurons);
    assert_eq!(decoded_neurons.len(), 2);
    assert!(decoded_neurons.contains_cortical_id(&power_id));
    assert!(decoded_neurons.contains_cortical_id(&death_id));
}

#[test]
fn test_byte_container_structure_types() {
    let cortical_id = CoreCorticalType::Power.to_cortical_id();
    let dimensions = CorticalAreaDimensions::new(2, 2, 1).unwrap();
    let neurons = sample_cortical_mapped_neurons(dimensions, cortical_id);

    let mut byte_container = FeagiByteContainer::new_empty();
    byte_container
        .overwrite_byte_data_with_single_struct_data(&neurons, 0)
        .unwrap();

    // Verify structure types are tracked correctly
    let struct_types = byte_container.get_contained_struct_types();
    assert_eq!(struct_types.len(), 1);
    assert_eq!(
        struct_types[0],
        FeagiByteStructureType::NeuronCategoricalXYZP
    );

    // Verify we can find the structure by type
    let found_struct = byte_container
        .try_create_struct_from_first_found_struct_of_type(
            FeagiByteStructureType::NeuronCategoricalXYZP,
        )
        .unwrap();
    assert!(found_struct.is_some());
}

#[test]
fn test_byte_container_increment_counter() {
    let cortical_id = CoreCorticalType::Death.to_cortical_id();
    let dimensions = CorticalAreaDimensions::new(1, 1, 1).unwrap();

    let mut byte_container = FeagiByteContainer::new_empty();

    // Test different increment counters
    for counter in 0..10 {
        let neurons = sample_cortical_mapped_neurons(dimensions, cortical_id);
        byte_container
            .overwrite_byte_data_with_single_struct_data(&neurons, counter)
            .unwrap();

        assert_eq!(byte_container.get_increment_counter().unwrap(), counter);
    }
}

#[test]
fn test_byte_container_large_neuron_set() {
    // Test with a larger set of neurons to ensure proper handling
    let cortical_id = CoreCorticalType::Power.to_cortical_id();
    let dimensions = CorticalAreaDimensions::new(50, 50, 10).unwrap();
    let total_neurons = dimensions.number_elements();

    let neurons = sample_cortical_mapped_neurons(dimensions, cortical_id);

    let mut byte_container = FeagiByteContainer::new_empty();
    byte_container
        .overwrite_byte_data_with_single_struct_data(&neurons, 0)
        .unwrap();

    // Verify the container is valid and contains the correct data
    assert!(byte_container.is_valid());
    assert_eq!(
        byte_container
            .try_get_number_contained_structures()
            .unwrap(),
        1
    );

    // Verify deserialization preserves all neurons
    let decoded_neurons: CorticalMappedXYZPNeuronVoxels = byte_container
        .try_create_new_struct_from_index(0)
        .unwrap()
        .try_into()
        .unwrap();

    let decoded_count = decoded_neurons.get_neurons_of(&cortical_id).unwrap().len();
    assert_eq!(decoded_count, total_neurons as usize);
    assert_eq!(decoded_neurons, neurons);
}

#[test]
fn test_byte_container_validation() {
    let mut byte_container = FeagiByteContainer::new_empty();

    // Initially valid
    assert!(byte_container.is_valid());

    // Add some data
    let cortical_id = CoreCorticalType::Power.to_cortical_id();
    let dimensions = CorticalAreaDimensions::new(3, 3, 3).unwrap();
    let neurons = sample_cortical_mapped_neurons(dimensions, cortical_id);

    byte_container
        .overwrite_byte_data_with_single_struct_data(&neurons, 0)
        .unwrap();

    // Still valid after adding data
    assert!(byte_container.is_valid());

    // Verify we can get the byte reference
    let byte_ref = byte_container.get_byte_ref();
    assert!(byte_ref.len() > FeagiByteContainer::GLOBAL_BYTE_HEADER_BYTE_COUNT);
}

#[test]
fn test_byte_container_memory_efficiency() {
    // Test that the container doesn't waste excessive memory
    let cortical_id = CoreCorticalType::Death.to_cortical_id();
    let dimensions = CorticalAreaDimensions::new(10, 10, 10).unwrap();
    let neurons = sample_cortical_mapped_neurons(dimensions, cortical_id);

    let mut byte_container = FeagiByteContainer::new_empty();
    byte_container
        .overwrite_byte_data_with_single_struct_data(&neurons, 0)
        .unwrap();

    let bytes_used = byte_container.get_number_of_bytes_used();
    let bytes_allocated = byte_container.get_number_of_bytes_allocated();

    // Allocated should be >= used
    assert!(bytes_allocated >= bytes_used);

    // After shrinking, they should be equal
    byte_container.free_unused_allocation();
    assert_eq!(
        byte_container.get_number_of_bytes_allocated(),
        byte_container.get_number_of_bytes_used()
    );
}
