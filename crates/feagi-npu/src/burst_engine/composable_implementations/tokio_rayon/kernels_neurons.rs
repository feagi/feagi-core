use crate::burst_engine::composable_implementations::tokio_rayon::data::neuron::neuron_sub_data::{CorticalIndexLookupTable, NeuronIndexLookupTable};
use crate::burst_engine::composable_implementations::tokio_rayon::data::TokioRayonEngineData;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::cortical_area::neuron::layout_specific_implementations::dimensional::DimensionalNeuronModel;
use feagi_models::cortical_area::neuron_model_implementations::feagi_advanced::model::FeagiAdvancedModel;
use feagi_models::cortical_area::neuron_model_implementations::generated_enums::NeuronModelTypeAndQuantizationPacked;
use feagi_models::wrapped_index_collections::NeuronEngineIndex;
use feagi_models::wrapped_indexes::BurstIndex;
use rayon::prelude::*;
use feagi_data::values::quantizable::{QuantizedUnsignedIntegerTrait, WrappedQuantizedUnsignedInteger};

pub(crate) fn process_neurons<FIQ: FeagiIndexQuantization>(data: &TokioRayonEngineData<FIQ>) {
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

                let neuron_engine_index: NeuronEngineIndex<FIQ::NeuronIndexQuant> = NeuronEngineIndex::quant_from_usize_unchecked(neuron_index);
                let neuron_runtime_flags = data.neuron_runtime_flags.get_mut_par(neuron_engine_index);
                let neuron_indexes_lookup = data.cortical_neuron_index_lookup_table.get_par(cortical_engine_index);

                let mut is_neuron_firing: bool = neuron_dynamics(
                    data,
                    cortical_context.0,
                    burst_index,
                    cortical_lookup,
                    neuron_engine_index,
                    neuron_indexes_lookup,
                );

                // Override if neuron is firing, with force off taking priority
                if neuron_runtime_flags.get_debug_force_off() {
                    is_neuron_firing = false;
                } else if neuron_runtime_flags.get_debug_force_fire() {
                    is_neuron_firing = true;
                }

                neuron_runtime_flags.set_firing(is_neuron_firing);
            });
    }
    return;
}
/// Packs each area's per-neuron firing flags into its run of the firing bitmap.
///
/// Runs after [`process_neurons`] has settled every neuron's firing state, and turns the byte per
/// neuron the kernel writes into the bit per neuron that readers consume. Keeping this a separate
/// pass rather than folding it into the dynamics loop is what makes it race-free: the dynamics
/// loop is parallel over neurons, and eight neurons share a bitmap byte, so it cannot write bits
/// without a read-modify-write race. Here the parallelism is over bytes instead, and each worker
/// owns its byte outright.
///
/// Each byte is written whole rather than or-ed in, so last burst's bits are cleared by the same
/// store that sets this burst's.
pub(crate) fn pack_firing_bitmap<FIQ: FeagiIndexQuantization>(data: &TokioRayonEngineData<FIQ>) {
    let neuron_counts = data.cortical_neuron_count.as_slice();
    let neuron_index_lookups = data.cortical_neuron_index_lookup_table.as_slice();
    let neuron_runtime_flags = data.neuron_runtime_flags.as_slice();

    (0..neuron_counts.len()).into_par_iter().for_each(|cortical_area| {
        let neuron_count = neuron_counts[cortical_area].quant_to_usize();
        let first_neuron = neuron_index_lookups[cortical_area]
            .cortical_first_neuron_engine_index
            .deref()
            .quant_to_usize();

        let bitmap_index = FIQ::CorticalAreaIndexCountQuant::quant_from_usize_unchecked(cortical_area);
        let Some((bitmap, _)) = data.neuron_voxel_is_firing.get_slice_by_index(bitmap_index) else {
            return;
        };

        (0..bitmap.number_bytes().quant_to_usize()).into_par_iter().for_each(|byte_index| {
            let first_local_neuron = byte_index * 8;
            // The final byte of an area whose neuron count is not a multiple of eight is
            // only partly populated; its remaining bits stay zero.
            let bits_in_byte = (neuron_count - first_local_neuron).min(8);

            let mut packed: u8 = 0;
            for bit in 0..bits_in_byte {
                if neuron_runtime_flags[first_neuron + first_local_neuron + bit].get_firing() {
                    packed |= 1 << bit;
                }
            }

            // SAFETY: byte indexes are produced by a range, so each is visited by exactly
            // one worker, and areas own disjoint byte ranges of the shared buffer. No
            // other reference to this byte exists for the duration of the write.
            unsafe {
                *bitmap.get_byte_mut_par(FIQ::NeuronIndexQuant::quant_from_usize_unchecked(byte_index)) = packed;
            }
        });
    });
}

// TODO this should be macro generated potentially (maybe from the models crate?)

#[inline(always)]
unsafe fn neuron_dynamics<FIQ: FeagiIndexQuantization>(
    data: &TokioRayonEngineData<FIQ>,
    model: NeuronModelTypeAndQuantizationPacked,
    burst_index: BurstIndex<FIQ::GlobalBurstIndexQuant>,
    cortical_lookup_table: &CorticalIndexLookupTable<FIQ>,
    neuron_engine_index: NeuronEngineIndex<FIQ::NeuronIndexQuant>,
    neuron_index_lookup_table: &NeuronIndexLookupTable<FIQ>,
) -> bool {
    match model {
        NeuronModelTypeAndQuantizationPacked::FeagiAdvanced_Standard => {
            // has per neuron data, full history, dimension layout

            let cortical_model_index = cortical_lookup_table.cortical_model_index;
            let cortical_layout_index = cortical_lookup_table.cortical_layout_index;

            let neuron_mp_index = neuron_index_lookup_table.get_neuron_mp_index(&neuron_engine_index);
            let neuron_model_index = neuron_index_lookup_table.get_neuron_model_index(&neuron_engine_index);
            let neuron_local_index = neuron_index_lookup_table.get_neuron_local_index(&neuron_engine_index);
            let neuron_history_index = neuron_index_lookup_table.get_neuron_history_index(&neuron_engine_index);

            let cortical_data = data
                .neuron_model_data
                .feagi_advanced
                .quantization_standard
                .cortical_data
                .get_par(cortical_model_index);
            let cortical_layout_data = data.cortical_layout_dimensional_data.get_par(cortical_layout_index);

            let neuron_data = data
                .neuron_model_data
                .feagi_advanced
                .quantization_standard
                .neuron_data
                .get_mut_par(neuron_model_index);
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
                neuron_mp,
            );

            neuron_history.burst_last_active = burst_index;
            if is_firing {
                neuron_history.burst_last_fired = burst_index;
            }
            is_firing
        }
    }
}
