use rayon::prelude::*;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::WrappedQuantizedIndexCount;
use feagi_models::cortical_area::neuron_model_implementations::feagi_advanced::model::FeagiAdvancedModel;
use feagi_models::cortical_area::neuron::neuron_model::generated_enums::NeuronModelTypeAndQuantizationPacked;
use feagi_models::cortical_area::neuron::neuron_model::layout_specific_implementations::dimensional::DimensionalNeuronModel;
use feagi_models::wrapped_index_collections::NeuronEngineIndex;
use feagi_models::wrapped_indexes::BurstIndex;
use crate::engines::rayon::data::RayonEngineData;
use crate::engines::rayon::data::neuron::neuron_sub_data::{CorticalIndexLookupTable, NeuronIndexLookupTable};

pub(crate) fn process_neurons<FIQ: FeagiIndexQuantization>(data: &RayonEngineData<FIQ>)
{
    let burst_index = data.burst_index;

    // We access `data` through a shared `&` and mutate disjoint slots via the
    // `get_mut_par` accessors
    unsafe {

        // We iterate over the is_firing bytes/bits , thus grouping everything into 8 neuron
        // clusters. We start this with `cortical_engine_indexes` which points us to the
        // cortical area of the 8 grouped neuron

        data.cortical_engine_indexes
            .as_slice()
            .par_iter()
            .enumerate()
            .for_each(|(neuron_index, &cortical_engine_index)| {

                let cortical_context = data.cortical_neuron_model_and_quant_and_neuron_properties.get_par(cortical_engine_index);
                let cortical_flags = cortical_context.1;

                if cortical_flags.get_cortical_area_frozen_input() {
                    // If cortical area is frozen, don't do anything
                    return;
                }

                let cortical_lookup = data.cortical_index_lookup_table.get_par(cortical_engine_index);

                let neuron_engine_index: NeuronEngineIndex<FIQ::NeuronIndexQuant> = NeuronEngineIndex::quant_from_usize(neuron_index);
                let neuron_runtime_flags = data.neuron_runtime_flags.get_mut_par(neuron_engine_index);
                let neuron_indexes_lookup = data.cortical_neuron_index_lookup_table.get_par(cortical_engine_index);

                let mut is_neuron_firing: bool = neuron_dynamics(
                    data,
                    cortical_context.0,
                    burst_index,
                    cortical_lookup,
                    neuron_engine_index,
                    neuron_indexes_lookup
                );

                // Override if neuron is firing, with force off taking priority
                if neuron_runtime_flags.get_force_off() { is_neuron_firing = false; }
                else if neuron_runtime_flags.get_force_fire() { is_neuron_firing = true; }

                neuron_runtime_flags.set_firing(is_neuron_firing);
            });


    }
    return;

}
// TODO this should be macro generated potentially (maybe from the models crate?)

#[inline(always)]
unsafe fn neuron_dynamics<FIQ: FeagiIndexQuantization>(
    data: &RayonEngineData<FIQ>,
    model: NeuronModelTypeAndQuantizationPacked,
    burst_index: BurstIndex<FIQ::GlobalBurstIndexQuant>,
    cortical_lookup_table: &CorticalIndexLookupTable<FIQ>,
    neuron_engine_index: NeuronEngineIndex<FIQ::NeuronIndexQuant>,
    neuron_index_lookup_table: &NeuronIndexLookupTable<FIQ>) -> bool
{

    match model {
        NeuronModelTypeAndQuantizationPacked::FeagiAdvanced_Standard => {
            // has per neuron data, full history, dimension layout
            
            let cortical_model_index = cortical_lookup_table.cortical_model_index;
            let cortical_layout_index = cortical_lookup_table.cortical_layout_index;

            let neuron_mp_index = neuron_index_lookup_table.get_neuron_mp_index(&neuron_engine_index);
            let neuron_model_index = neuron_index_lookup_table.get_neuron_model_index(&neuron_engine_index);
            let neuron_local_index = neuron_index_lookup_table.get_neuron_local_index(&neuron_engine_index);
            let neuron_history_index = neuron_index_lookup_table.get_neuron_history_index(&neuron_engine_index);
            
            
            let cortical_data = data.neuron_model_data.feagi_advanced.quantization_standard.cortical_data.get_par(cortical_model_index);
            let cortical_layout_data = data.cortical_layout_dimensional_data.get_par(cortical_layout_index);
            
            let neuron_data = data.neuron_model_data.feagi_advanced.quantization_standard.neuron_data.get_mut_par(neuron_model_index);
            let neuron_fcl = data.neuron_membrane_data.fcl_f32.get_mut_par(neuron_mp_index);
            let neuron_mp = data.neuron_membrane_data.mp_f32.get_mut_par(neuron_mp_index);
            let neuron_history = data.neuron_history_data.get_mut_par(neuron_history_index);

            let is_firing = FeagiAdvancedModel::process_incoming_potential_for_dimensional_area(
                neuron_fcl,
                &neuron_local_index,
                &burst_index,
                &cortical_layout_data.dimensions,
                neuron_history,
                cortical_data,
                neuron_data,
                neuron_mp
            );

            neuron_history.burst_last_active = burst_index;
            if is_firing {
                neuron_history.burst_last_fired = burst_index;
            }
            is_firing
        }
    }



}