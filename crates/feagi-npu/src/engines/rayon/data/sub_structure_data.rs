use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_npu_common::wrapped_indexes::{CorticalLayoutIndex, CorticalModelIndex};

/// Contains cortical level neuron offsets to go from a neuron engine index to various other
/// neuron related properties relative to its parent cortical area
#[derive(Clone, Copy)]
pub struct CorticalNeuronOffsets<FIQ: FeagiIndexQuantization>
{
    pub engine_to_local_neuron_index_offset: FIQ::NeuronIndexCountQuant,
    pub engine_to_mp_quant_neuron_index: FIQ::NeuronIndexCountQuant,
    pub engine_to_psp_uniformity_index: FIQ::NeuronIndexCountQuant, // Only valid if psp uniformity is enabled, otherwise may be zero or some invalid value
    pub engine_to_neuron_history_index_offset: FIQ::NeuronIndexCountQuant, // Only valid if the neuron model needs history. Otherwise this will just be 0
}

/// Properties and indexes all cortical areas have
#[derive(Clone, Copy)]
pub struct CorticalContext<FIQ: FeagiIndexQuantization>
{
    pub cortical_model_index: CorticalModelIndex<FIQ::CorticalAreaIndexCountQuant>, // typed
    pub cortical_layout_index: CorticalLayoutIndex<FIQ::CorticalAreaIndexCountQuant>, // typed
}

