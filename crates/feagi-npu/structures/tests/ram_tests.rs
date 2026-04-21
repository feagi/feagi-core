
#[cfg(test)]
mod connectome {
    use feagi_npu_structures::connectome::{ConnectomeAllocRam, ConnectomeAllocTrait};
    use feagi_npu_structures::quantizables::NPUQuantization;
    use feagi_structures::descriptors::QuantizablePercentage;
    use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
    use feagi_structures::neurons::descriptors::NumberNeuronsPerVoxel;
    use super::*;

    struct TestQuantization;
    impl NPUQuantization for TestQuantization {
        type NeuronIndex = u32;
        type SynapseIndex = u32;
        type SynapseBundleIndex = u32;
        type CorticalIndex = u16;
        type Coord = u32;
        type BurstDelta = u16;
        type BurstIndex = u32;
        type Value = f32;
        type Percentage = f32;
    }



    #[test]
    fn test_ram_npu<>() {

        let dimensions_a = NeuronVoxelDimensions::<<TestQuantization as NPUQuantization>::Coord>::new(20, 20, 20).unwrap();
        let density_a: NumberNeuronsPerVoxel = 1;


        let dimensions_b =  NeuronVoxelDimensions::<<TestQuantization as NPUQuantization>::Coord>::new(10, 10, 10).unwrap();
        let density_b: NumberNeuronsPerVoxel = 2;


        let mut connectome: ConnectomeAllocRam<TestQuantization> = ConnectomeAllocRam::new();

        connectome.create_interneuron_area_with_default_neurons(dimensions_a, density_a);
        connectome.create_interneuron_area_with_default_neurons(dimensions_b, density_b);



    }
}



