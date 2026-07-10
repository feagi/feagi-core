use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::wrapped_indexes::BurstIndex;

/// Contains the burst indexes from when the neuron was last active / fired. Only neuron models
/// that require this data will get this data indexed
#[derive(Clone, Copy)]
pub struct NeuronHistory<FIQ: FeagiIndexQuantization>
{
    pub burst_last_active: BurstIndex<FIQ::GlobalBurstIndexQuant>,
    pub burst_last_fired: BurstIndex<FIQ::GlobalBurstIndexQuant>,
}