



#[cfg(test)]
mod burst_engine_rayon_tests
{
    use feagi_npu_structures::neural_processing_unit_data_structures::burst_engine::model_implementations::neuron_models::feagi_standard::data::{FeagiStandardModelCorticalDataCPU, FeagiStandardModelNeuronDataCPU};
    use feagi_npu_structures::neural_processing_unit_data_structures::burst_engine::model_implementations::neuron_models::feagi_standard::quantization::{FeagiStandardModelQuantization, FeagiStandardModelStandard32BitQuant};
    use feagi_npu_structures::neural_processing_unit_data_structures::burst_engine::engines::rayon::npu_data::BurstEngineDataRayon;
    use feagi_npu_structures::neural_processing_unit_data_structures::burst_engine::engines::rayon::npu_data::npu_structured::burst_engine_global::{CorticalContextLookup, CorticalLayoutDimensionalCPU, NPUWrappedEngineSynapseIndexLength, NPUWrappedNeuronHistoryIndex, NeuronHistory, SynapseRangeMappingFromNeuron};
    use feagi_npu_structures::neural_processing_unit_data_structures::burst_engine::engines::rayon::phase_processors::{BurstEnginePhaseBurstCounterIndexIncrementRayon, BurstEnginePhaseProcessorCPU, NeuronDynamicsNoPreCondenseRayon, SynapseDynamicsNoPreCondenseRayon};
    use feagi_npu_structures::neural_processing_unit_data_structures::wrappers::{NPUWrappedCorticalAreaBurstEngineIndex, NPUWrappedCorticalAreaDimensions, NPUWrappedCorticalLayoutIndex, NPUWrappedDimensionalNeuronAxialPosition, NPUWrappedDimensionalNeuronDensity, NPUWrappedNeuronCorticalLocalIndex, NPUWrappedNeuronIndexBurstEngineIndex, NPUWrappedNeuronMembranePotential, NPUWrappedNeuronNeuronModelMPQuantIndex};
    use feagi_structures::feagi_data::quantizable_linear::base_types::QuantizedIndexCountTrait;
    use feagi_structures::feagi_data::quantizable_linear::wrappers::QuantizedElementWrapperBase;
    use feagi_structures::feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantizationStandard;
    use feagi_structures::feagi_data::SupportsUintOps;
    use std::time::Instant;

    #[test]
    fn rayon_full_test_suite()
    {
        const NUMBER_CORTICAL_AREAS: usize = 2;
        let cube_length: u32 = 100;
        let cube_density: u32 = 3;
        let number_bursts: usize = 10;

        let mut engine_data: BurstEngineDataRayon<FeagiGlobalQuantizationStandard> = BurstEngineDataRayon::new();


        // append cortical_area areas manually

        let mut neuron_index: NPUWrappedNeuronIndexBurstEngineIndex<u32> = NPUWrappedNeuronIndexBurstEngineIndex::QUANT_ZERO;
        let number_neurons_cortical_area: u32 = cube_length * cube_length * cube_length * cube_density;
        let synapse_weight = vec![0];
        // TODO set to 1-1 mapping for now, have a setting not froma  flag

        for c in 0..NUMBER_CORTICAL_AREAS
        {
            engine_data.cortical_context_lookups.push(
                CorticalContextLookup {
                    mp_quant_to_local_neuron_index_offset: NPUWrappedNeuronCorticalLocalIndex::wrap(neuron_index.unwrap()),
                    mp_quant_to_neuron_history_index_offset: NPUWrappedNeuronHistoryIndex::wrap(neuron_index.unwrap()),
                    cortical_layout_index: NPUWrappedCorticalLayoutIndex::wrap(c as u16),
                    neuron_model_cortical_data_index: NPUWrappedNeuronNeuronModelMPQuantIndex::wrap(c as u16),
                }
            );

            // per neuron
            for n in 0..number_neurons_cortical_area
            {
                let neuron_data: FeagiStandardModelNeuronDataCPU<FeagiStandardModelStandard32BitQuant> =
                    FeagiStandardModelNeuronDataCPU::new(
                        1.0,
                        1.0,
                        1,
                        1);

                if c % 3 == 0 {
                    engine_data.neuron_fcls.float_32.push(
                        NPUWrappedNeuronMembranePotential::wrap(100.0)
                    );
                } else {
                    engine_data.neuron_fcls.float_32.push(
                        NPUWrappedNeuronMembranePotential::wrap(0.0)
                    );
                }

                engine_data.neuron_potentials.float_32.push(
                    NPUWrappedNeuronMembranePotential::wrap(0.0)
                );


                engine_data.neuron_history.push(
                    NeuronHistory {
                        burst_index_of_last_input: Default::default(),
                        burst_index_of_last_firing: Default::default()
                    }
                );

                engine_data.neuron_engine_cortical_indexes.push(
                    NPUWrappedCorticalAreaBurstEngineIndex::from_u32(c as u32)
                );

                engine_data.neuron_model_neuron_data.push(
                    neuron_data
                );


                engine_data.synapses_ranges_from_neurons.push(SynapseRangeMappingFromNeuron {
                    synapse_start_index: NPUWrappedEngineSynapseIndexLength::wrap(neuron_index.unwrap()),
                    synapse_start_length: NPUWrappedEngineSynapseIndexLength::QUANT_ONE,
                    source_neuron_index: neuron_index,
                });





                neuron_index += NPUWrappedNeuronIndexBurstEngineIndex::QUANT_ONE;
            }

            let cortical_data: FeagiStandardModelCorticalDataCPU<FeagiStandardModelStandard32BitQuant> =
                FeagiStandardModelCorticalDataCPU::new(
                    1.0,
                    1 as u32,
                    1.0,
                    1 as u32
                );

            let cortical_area_dimensions: NPUWrappedCorticalAreaDimensions<u32>
                = NPUWrappedCorticalAreaDimensions::new_unchecked(
                NPUWrappedDimensionalNeuronAxialPosition::from_u32(cube_length),
                NPUWrappedDimensionalNeuronAxialPosition::from_u32(cube_length),
                NPUWrappedDimensionalNeuronAxialPosition::from_u32(cube_length),
                NPUWrappedDimensionalNeuronDensity::from_u32(cube_density),
            );

            engine_data.cortical_layouts.dimensional.push(
                CorticalLayoutDimensionalCPU::new(
                    cortical_area_dimensions
                )
            );

            engine_data.neuron_model_cortical_data.push(
                cortical_data
            );


        }

        let burst_timer_start = Instant::now();

        // run bursts

        for _i in 0..number_bursts
        {
            // increment burst index
            _ = BurstEnginePhaseBurstCounterIndexIncrementRayon::process_phase(&mut engine_data);

            // neuron dynamics
            _ = NeuronDynamicsNoPreCondenseRayon::process_phase(&mut engine_data);

            // Synapse Dynamics
            _ = SynapseDynamicsNoPreCondenseRayon::process_phase(&mut engine_data);

        }

        let total_burst_millisecondss = burst_timer_start.elapsed().as_millis();
        let average_millisecondss_per_burst = total_burst_millisecondss as f64 / number_bursts as f64;
        println!(
            "Rayon burst engine: {} bursts in {} milliseconds (avg {:.2} millisecondss/burst)",
            number_bursts, total_burst_millisecondss, average_millisecondss_per_burst
        );
    }
}
















/*
    // TODO Actual Injection!

    use feagi_npu_structures::dynamic_burst_engine_interface::npu_requests::npu_request::NPURequest;
    use feagi_structures::genomic::cortical_area::{CoreCorticalType, CorticalAreaType};
    use feagi_structures::neuron_voxels::bit_32::NeuronVoxelDimensions;

    #[test]
    fn npu_dynamics()
    {
        let area_alpha = NPURequest::cortical_area_create_custom(
            NeuronVoxelDimensions::new_unchecked(32, 32, 32),
            0,
            CorticalAreaType::Core(CoreCorticalType::Power).to_cortical_id(),
            Default::default());
    }

     */
