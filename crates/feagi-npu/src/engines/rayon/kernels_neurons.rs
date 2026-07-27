use rayon::prelude::*;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::WrappedQuantizedIndexCount;
use feagi_models::neuron::common_structs::cortical_area_layout::CorticalAreaLayout;
use feagi_models::neuron::model_and_quantization::PackedNeuronModelTypeAndQuantization;
use feagi_models::neuron::model_extensions::neuron_layout_implementations::DimensionalNeuronModel;
use feagi_models::neuron::models::feagi_advanced::FeagiAdvancedModel;
use feagi_models::wrapped_index_collections::{CorticalEngineIndex, NeuronEngineByteIndex, NeuronEngineIndex};
use feagi_models::wrapped_indexes::BurstIndex;
use crate::engines::rayon::data::RayonEngineData;
use crate::engines::rayon::data::sub_structure_data::{CorticalIndexLookupTable, NeuronIndexLookupTable};

fn process_neurons<FIQ: FeagiIndexQuantization>(data: &mut RayonEngineData<FIQ>)
{
    let burst_index = data.burst_index;

    // Rust does NOT like mut par operations, so this entire section is an unsafe block lol
    unsafe {

        // We iterate over the is_firing bytes/bits , thus grouping everything into 8 neuron
        // clusters. We start this with `cortical_engine_indexes` which points us to the
        // cortical area of the 8 grouped neuron

        data.cortical_engine_indexes
            .as_slice()
            .par_iter()
            .enumerate()
            .for_each(|(neuron_group_index, &cortical_engine_index)| {
                let neuron_group_index: NeuronEngineByteIndex<FIQ::NeuronIndexCountQuant> = NeuronEngineByteIndex::quant_from_usize(neuron_group_index);

                let cortical_context = data.cortical_neuron_model_and_quant_and_neuron_properties.get_par(cortical_engine_index);
                let cortical_flags = cortical_context.1;

                if cortical_flags.get_is_cortical_area_frozen_input() {
                    // If cortical area is frozen, don't do anything
                    return;
                }

                let cortical_lookup = data.cortical_index_lookup_table.get_par(cortical_engine_index);
                let neuron_group_lookup = data.cortical_neuron_index_lookup_table.get_par(cortical_engine_index);
                let neuron_count = data.cortical_neuron_count.get_par(cortical_engine_index);

                // Have to go through usize since step can only be implemented on unstable compiler versions
                for neuron_engine_index_u in neuron_group_lookup.get_neuron_engine_index_range_for_group(&neuron_group_index, *neuron_count)
                {
                    let neuron_engine_index: NeuronEngineIndex<FIQ::NeuronIndexCountQuant> = NeuronEngineIndex::quant_from_usize(neuron_engine_index_u);

                    //neuron_dynamics(data, )

                };





            })


    }
    return;

}


// TODO this should be macro generated potentially (maybe from the models crate?)

#[inline]
unsafe fn neuron_dynamics<FIQ: FeagiIndexQuantization>(
    data: &mut RayonEngineData<FIQ>,
    model: PackedNeuronModelTypeAndQuantization,
    burst_index: BurstIndex<FIQ::GlobalBurstIndexQuant>,
    cortical_engine_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
    cortical_lookup_table: &CorticalIndexLookupTable<FIQ>,
    neuron_engine_index: NeuronEngineIndex<FIQ::NeuronIndexCountQuant>,
    neuron_lookup_table: &NeuronIndexLookupTable<FIQ>) -> bool
{

    match model {
        PackedNeuronModelTypeAndQuantization::FeagiAdvanced_Standard => {
            // has per neuron data, full history, dimension layout
            
            let cortical_model_index = cortical_lookup_table.cortical_model_index;
            let cortical_layout_index = cortical_lookup_table.cortical_layout_index;

            let neuron_mp_index = neuron_lookup_table.get_neuron_mp_index(&neuron_engine_index);
            let neuron_model_index = neuron_lookup_table.get_neuron_model_index(&neuron_engine_index);
            let neuron_local_index = neuron_lookup_table.get_neuron_local_index(&neuron_engine_index);
            let neuron_history_index = neuron_lookup_table.get_neuron_history_index(&neuron_engine_index);
            
            
            let cortical_data = data.neuron_model_data.cortical_model_feagi_advanced_quant_standard.get_par(cortical_model_index);
            let cortical_layout_data = data.cortical_layout_dimensional_data.get_par(cortical_layout_index);
            
            let neuron_data = data.neuron_model_data.neuron_model_feagi_advanced_quant_standard.get_mut_par(neuron_model_index);
            let neuron_fcl = data.neuron_membrane_data.fcl_f32.get_mut_par(neuron_mp_index);
            let neuron_mp = data.neuron_membrane_data.mp_f32.get_mut_par(neuron_mp_index);
            let neuron_history = data.neuron_history_data.get_mut_par(neuron_history_index);

            FeagiAdvancedModel::process_incoming_potential_for_dimensional_area(
                neuron_fcl,
                &neuron_local_index,
                &burst_index,
                &cortical_layout_data.dimensions,
                neuron_history,
                cortical_data,
                neuron_data,
                neuron_mp)
        }
    }



}