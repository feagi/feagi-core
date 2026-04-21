
#[cfg(test)]
mod connectome {
    use feagi_npu_structures::connectome::{ConnectomeAllocRam, ConnectomeAllocTrait, ConnectomeBaseTrait};
    use feagi_npu_structures::executors::cortical_mapping_definition_executors::NonPlasticCorticalMappingDefinitionExecutor;
    use feagi_npu_structures::neuron::dimensional_neurons::shared_structs::DimensionalNeuronCorticalData;
    use feagi_npu_structures::neuron::dimensional_neurons::shared_structs::DimensionalTypedNeuronIndex;
    use feagi_npu_structures::neuron::flags::NeuronFlag;
    use feagi_npu_structures::quantizables::{BurstDelta, NPUQuantization, NPUNeuronIndex, PSPMultiplier, SynapseCount, SynapticWeight, BurstGlobalIndex};
    use feagi_npu_structures::synapse::non_plastic_dimensional::{NonPlasticSynapseFull, NonplasticSynapseProperties};
    use feagi_npu_structures::synapse::SynapseFlag;
    use feagi_structures::base_quantizable::{QuantizableUIntType, QuantizableValueType};
    use feagi_structures::FeagiStructuresError;
    use feagi_structures::genomic::cortical_area::DimensionCorticalAreaType;
    use feagi_structures::neuron_voxels::descriptors::NeuronVoxelDimensions;
    use feagi_structures::neurons::descriptors::NumberNeuronsPerVoxel;

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
    
    struct TestSynapseMapper {
        
    }
    impl<Q: NPUQuantization> NonPlasticCorticalMappingDefinitionExecutor<Q> for TestSynapseMapper {
        fn non_plastic_synapse_iterator(&self, 
                                        source_area_type: DimensionCorticalAreaType, 
                                        _source_cortical_data: &DimensionalNeuronCorticalData<Q>, 
                                        _source_neuron_flags: &[NeuronFlag], 
                                        destination_area_type: DimensionCorticalAreaType, 
                                        _destination_cortical_data: &DimensionalNeuronCorticalData<Q>, 
                                        _destination_neuron_flags: &[NeuronFlag]) 
            -> Result<(impl Iterator<Item=NonPlasticSynapseFull<Q::NeuronIndex, Q::BurstDelta, Q::Value>>, SynapseCount<Q::SynapseIndex>), FeagiStructuresError> {
            let iter = (0usize..=5).map(move |i| {
                let mut synapse_flag = SynapseFlag::ALL_ZEROS;
                synapse_flag.set_valid(true);

                NonPlasticSynapseFull {
                    source_neuron_index: DimensionalTypedNeuronIndex {
                        index: NPUNeuronIndex(Q::NeuronIndex::ZERO),
                        dimensional_type: source_area_type,
                    },
                    destination_neuron_index: DimensionalTypedNeuronIndex {
                        index: NPUNeuronIndex(Q::NeuronIndex::from_usize(i)),
                        dimensional_type: destination_area_type,
                    },
                    synapse_properties: NonplasticSynapseProperties {
                        synapse_flag,
                        synapse_weight: SynapticWeight(Q::Value::ZERO),
                        postsynaptic_potential_multiplier: PSPMultiplier(Q::Value::ZERO),
                        synaptic_delay: BurstDelta(Q::BurstDelta::ZERO),
                    },
                }
            });

            Ok((iter, SynapseCount(Q::SynapseIndex::from_usize(6))))
        }
    }



    #[test]
    fn test_ram_npu<>() {

        let dimensions_a = NeuronVoxelDimensions::<<TestQuantization as NPUQuantization>::Coord>::new(20, 20, 20).unwrap();
        let density_a: NumberNeuronsPerVoxel = 1;
        
        let dimensions_b =  NeuronVoxelDimensions::<<TestQuantization as NPUQuantization>::Coord>::new(10, 10, 10).unwrap();
        let density_b: NumberNeuronsPerVoxel = 2;

        let mut connectome: ConnectomeAllocRam<TestQuantization> = ConnectomeAllocRam::new();
        let mut burst_index: BurstGlobalIndex<u32> = BurstGlobalIndex::ZERO;

        
        // Create 2 cortical areas
        let cortical_index_a = connectome.create_interneuron_area_with_default_neurons(dimensions_a, density_a).unwrap();
        let cortical_index_b = connectome.create_interneuron_area_with_default_neurons(dimensions_b, density_b).unwrap();

        let mapper = TestSynapseMapper {};

        let synapse_bundle_one = connectome.add_nonplastic_connection_from_dimensional_area_to_dimensional_area(
            cortical_index_a,
            DimensionCorticalAreaType::Custom,
            cortical_index_b,
            DimensionCorticalAreaType::Custom,
            &mapper
        ).unwrap();

        connectome.process_burst(&burst_index).unwrap();

        
    }
}



