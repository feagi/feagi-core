
use feagi_data::quantization_levels::feagi_index_quantization::FeagiGlobalQuantization;
use feagi_npu_common::wrapped_indexes::{BurstIndex, CorticalEngineIndexedVector, NeuronCorticalLocalIndex, NeuronMPIndex, NeuronMPIndexedVector, NeuronModelIndexedVector, NeuronWithHistoryIndex, NeuronWithHistoryIndexedVector};
use feagi_npu_common::wrapped_values::NeuronMembranePotential;




pub struct RayonConnectomeData<FIQ: FeagiGlobalQuantization>
{
    pub burst_index: BurstIndex<FIQ::GlobalBurstIndexQuant>,

    /// If the burst index just overflowed, set to true. All other times is false
    pub did_burst_index_overflow: bool,

    pub connectome_neuron_index_offsets_to_mp_quant_neuron_index: ConnectomeNeuronIndexOffsetToMPNeuronIndex<FIQ>,

    pub neuron_history: NeuronWithHistoryIndexedVector<
        FIQ::NeuronIndexCountQuant,
        NeuronHistory<FIQ>
    >,

    pub percent_neurons_firing_this_burst: Vec<u8>, // TODO percentage type!

    pub cortical_context_lookups: CorticalEngineIndexedVector<FIQ::CorticalAreaIndexCountQuant, CorticalContextLookup<FIQ>>,

    // TODO Dimensional cortical layouts vector

    pub neuron_model_neuron_data: NeuronModelNeuronData<FIQ>,

    pub neuron_model_cortical_data: NeuronModelCorticalData<FIQ>,


    pub neuron_potentials: MPQuantMembranePotentials<FIQ>


}


/// Definitions for custom collections that are grouped in non-abritary ways
//region Grouped Collections


decimal_wrapped_vector_quant_group!(
    /// MP Grouped Neuron Potential Values
    MPQuantMembranePotentials,
    NeuronMPIndexedVector,
    NeuronIndexCountQuant
);



//endregion


//region SubStructs

pub struct ConnectomeNeuronIndexOffsetToMPNeuronIndex<FIQ: FeagiGlobalQuantization>
{
    pub float_32_down_offset: FIQ::NeuronIndexCountQuant,
}

impl<FIQ: FeagiGlobalQuantization> ConnectomeNeuronIndexOffsetToMPNeuronIndex<FIQ>
{
    // TODO offset func
}


/// Denotes the last time a specific neuron fired or had an input activity at all. As not all
/// neuron models use this, has its own indexing
#[derive(Clone, Copy)]
pub struct NeuronHistory<FIQ: FeagiGlobalQuantization>
{
    pub burst_index_of_last_input: BurstIndex<FIQ::GlobalBurstIndexQuant>,
    pub burst_index_of_last_firing: BurstIndex<FIQ::GlobalBurstIndexQuant>,
}







// TODO needs proper grouping, macro generation?
pub struct NeuronModelNeuronData<FIQ: FeagiGlobalQuantization> {
    pub float_32_feagi_standard_model: NeuronModelIndexedVector<FIQ::NeuronIndexCountQuant, f32>
}

impl<FIQ: FeagiGlobalQuantization> NeuronModelNeuronData<FIQ> {
    //TODO func to get mut impl of neuron model data
}


// TODO needs proper grouping, macro generation?
pub struct NeuronModelCorticalData<FIQ: FeagiGlobalQuantization> {
    pub float_32_feagi_standard_model: NeuronModelIndexedVector<FIQ::CorticalAreaIndexCountQuant, f32>
}

impl<FIQ: FeagiGlobalQuantization> NeuronModelCorticalData<FIQ> {
    //TODO func to get mut impl of neuron model data
}

//endregion