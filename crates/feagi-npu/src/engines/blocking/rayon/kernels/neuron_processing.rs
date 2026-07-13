use crate::engines::blocking::rayon::data::engine_data::RayonEngineData;
use crate::engines::blocking::rayon::data::sub_structure_data::CorticalNeuronOffsets;
use feagi_data::collections::linear::bitpacked::{BitPackedMutTrait, BitPackedTrait};
use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronCorticalLocalIndex, NeuronVoxelDensityIndex};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::QuantizedIndexCountTrait;
use feagi_models::burst_index::{CorticalEngineIndex, CorticalLayoutIndex, NeuronEngineByteIndex, NeuronEngineIndex, NeuronHistoryIndex, NeuronMPIndex, NeuronPSPUniformIndex};
use rayon::prelude::*;
use feagi_data::neuron_voxels::wrapped_values::NeuronVoxelCoordinateAxis;
use feagi_models::cortical_area_layout::CorticalAreaLayoutDataDimensional;
use feagi_npu_models::neuron_models::feagi_standard::processor::FeagiStandardModelProcessor;
use feagi_npu_models::neuron_models::NeuronModelCPUDescriptor;

/// Contains several methods of processing the neurons in the rayon burst engine
pub enum RayonNeuronProcessing {
    /// Updates visualizer data always as each loop processes neurons in batches of 8 and directly
    /// writes to the bitpacked u8, without need a separate loop to check for visualization
    VisualizerInline,
}

impl RayonNeuronProcessing {
    ///
    pub fn process_neurons<FIQ: FeagiIndexQuantization>(&self, data: &mut RayonEngineData<FIQ>) {

        let number_engine_neurons =
            NeuronEngineIndex::from(data.bitpacked_neuron_activity.number_addressable_bits());
        let burst_index = data.burst_index;
        
        
        match self {
            RayonNeuronProcessing::VisualizerInline => {
                // We iterate over the bytes directly and write the results there
                
                data.bitpacked_neuron_activity
                    .par_iter_bytes_mut()
                    .enumerate()
                    .for_each(|(index_u, neuron_bits)| {


                        let neuron_byte_i: NeuronEngineByteIndex<FIQ::NeuronIndexCountQuant> =
                            NeuronEngineByteIndex::from_usize_unchecked(index_u);
                        let mut neuron_engine_i: NeuronEngineIndex<FIQ::NeuronIndexCountQuant> =
                            NeuronEngineIndex::from_usize_unchecked(neuron_byte_i.to_usize() << 3);
                        // At the tail end of cortical areas, there will likely be some spare bits
                        // and thus not a full 8 bits allocated to neurons
                        let num_neurons_in_byte =
                            (number_engine_neurons.to_usize() - neuron_engine_i.to_usize()).min(8);
                        
                        unsafe {
                            // TODO should we reset the activity first?

                            let cortical_engine_i: CorticalEngineIndex<
                                FIQ::CorticalAreaIndexCountQuant,
                            > = *data.neuron_cortical_mapping.get_par(neuron_byte_i);
                            
                            let cortical_runtime_flags =
                                *data.cortical_runtime_flags.get_par(cortical_engine_i);

                            if cortical_runtime_flags.get_cortical_area_paused()
                            {
                                // If paused, return without resetting or changing anything
                                return;
                            }
                            
                            *neuron_bits = 0; // Reset neuron firing bits
                            
                            let cortical_neuron_offsets: &CorticalNeuronOffsets<FIQ> =
                                data.cortical_neuron_offsets.get_par(cortical_engine_i);
                            
                            // TODO check what cortical layout we are using!
                            
                            let cortical_context = data.cortical_contexts.get_par(cortical_engine_i);
                            
                            // TODO we need per neuron type quantization reading. for now... just dont
                            // TODO obv this should be read per cortical area
                            let neuron_model_quant = NeuronModelCPUDescriptor::FEAGI_STANDARD_FLOAT_32;


                            for _ in 0..num_neurons_in_byte {
                                
                                // neuron_engine_i is the neuron engine level index
                                // First checking to get the neuron indexes, and seeing if the
                                // neuron is actually allowed to fire
                                
                                let neuron_runtime_flags = data.neuron_runtime_flags.get_par(neuron_engine_i);
                                
                                if !neuron_runtime_flags.get_force_off() {
                                    // If a neuron is disabled, do not
                                    // run the neuron, just skip it
                                    neuron_engine_i += NeuronEngineIndex::QUANT_ONE;
                                    continue;
                                }
                                
                                // At this point we know the neuron can fire. Check if the neuron
                                // has received any input that would need processing to see if we
                                // are firing
                                
                                let neuron_mp_i: NeuronMPIndex<FIQ::NeuronIndexCountQuant> =
                                    NeuronMPIndex::from(
                                        *neuron_engine_i.as_ref()
                                            - cortical_neuron_offsets
                                                .engine_to_mp_quant_neuron_index,
                                    );
                                
                                
                                let fcl = data.neuron_fcl.float_32.get_mut_par(neuron_mp_i);
                                if *fcl == 0.0 {
                                    neuron_engine_i += NeuronEngineIndex::QUANT_ONE;
                                    continue; // no need to check if the neuron isnt active
                                }
                                
                                // We have input. Collect info and context needed for this neuron's
                                // firing function

                                let mp = data.neuron_mp.float_32.get_mut_par(neuron_mp_i);

                                let cortical_data = data.neuron_model_data_container.cortical_feagi_standard_float_32.cortical_areas.get_par(cortical_context.cortical_model_index);
                                let neuron_data = data.neuron_model_data_container.neuron_feagi_standard_float_32.neurons.get_mut_par()
                                
                                let neuron_local_i: NeuronCorticalLocalIndex<
                                    FIQ::NeuronIndexCountQuant,
                                > = NeuronCorticalLocalIndex::from(
                                    *neuron_engine_i.as_ref()
                                        - cortical_neuron_offsets
                                        .engine_to_local_neuron_index_offset,
                                );


                                // TODO another match case of layout types (dont merge this with the other jump table, too big)
                                
                                let dimensional_layout = data.cortical_layouts.dimensional.get_par(cortical_context.cortical_layout_index);
                                
                                
                                match neuron_model_quant {
                                    NeuronModelCPUDescriptor::FEAGI_STANDARD_FLOAT_32 => {
                                        
                                        
                                        
                                        
                                    }
                                }
                                
                                

                                
                                
                                
                                



                                let neuron_history_i: NeuronHistoryIndex<
                                    FIQ::NeuronIndexCountQuant,
                                > = NeuronHistoryIndex::from(
                                    *neuron_engine_i.as_ref()
                                        - cortical_neuron_offsets
                                        .engine_to_neuron_history_index_offset,
                                );



                                
                                
                                
                                
                                


                                let neuron_psp_uni_i: NeuronPSPUniformIndex<
                                    FIQ::NeuronIndexCountQuant,
                                > = NeuronPSPUniformIndex::from(
                                    *neuron_engine_i.as_ref()
                                        - cortical_neuron_offsets.engine_to_psp_uniformity_index,
                                );
                                



                                // TODO get actual flag for neuron model and quantization and run appropriate match

                                // TODO why does the neuron processor trait only assume dimensional?

                                // TODO instead of multiple processor traits, constrain the where clause on the function


                                neuron_engine_i += NeuronEngineIndex::QUANT_ONE;
                            }
                        }
                    });

                // Rayon doesn't need to consolidate FCL, work stealing is sufficient

                // for each bit pack u8, start a new mut u8
                // loop 0 - 8 get neuron engine index (start then increment)

                // // get mut neuron fcl mp
                // // if fcl is zero, continue loop
                // // get cortical area index
                // // get cortical context
                // // get local neuron index
                // // get mut neuron history
                // // get dimensional layout TODO not always dimensional!
                // // get cortical model data
                // // get mut neuron model data
                // // get mut neuron potential
                // // is_firing = feagi standard model firing

                // // neuron fcl mp = 0
                // // update neuron history
                // // update bit 0-8 for if fired

                //

                increment_burst_counter(data);
            }
        }
    }
}


fn increment_burst_counter<FIQ: FeagiIndexQuantization>(data: &mut RayonEngineData<FIQ>) {
    if *data.burst_index.as_ref() == FIQ::GlobalBurstIndexQuant::QUANT_MAX {
        // OVERFLOW!

        *data.burst_index.as_mut() =
            FIQ::GlobalBurstIndexQuant::QUANT_MAX / FIQ::GlobalBurstIndexQuant::from_usize(2)

        // TODO call the right functions to handle overflow
    } else {
        *data.burst_index.as_mut() += FIQ::GlobalBurstIndexQuant::QUANT_MAX;
    }
}
func(model: NeuronModelCPUDescriptor) -> &impl 