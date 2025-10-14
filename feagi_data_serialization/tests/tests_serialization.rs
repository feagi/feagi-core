// TODO we need some example byte arrays to write some more tests!

use feagi_data_serialization::FeagiByteContainer;
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
