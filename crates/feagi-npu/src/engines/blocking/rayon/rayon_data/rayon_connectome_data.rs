
use feagi_data::quantization_levels::feagi_index_quantization::FeagiGlobalQuantization;
use feagi_npu_common::wrapped_indexes::{BurstIndex, CorticalEngineIndexedVector, NeuronCorticalLocalIndex, NeuronMPIndex, NeuronMPIndexedVector, NeuronModelIndexedVector, NeuronWithHistoryIndex, NeuronWithHistoryIndexedVector};
use feagi_npu_common::wrapped_values::NeuronMembranePotential;




pub struct RayonConnectomeData<FGQ: FeagiGlobalQuantization>
{
    pub burst_index: BurstIndex<FGQ::GlobalBurstIndexQuant>,

    /// If the burst index just overflowed, set to true. All other times is false
    pub did_burst_index_overflow: bool,

    pub connectome_neuron_index_offsets_to_mp_quant_neuron_index: ConnectomeNeuronIndexOffsetToMPNeuronIndex<FGQ>,

    pub neuron_history: NeuronWithHistoryIndexedVector<
        FGQ::NeuronIndexCountQuant,
        NeuronHistory<FGQ>
    >,

    pub percent_neurons_firing_this_burst: Vec<u8>, // TODO percentage type!

    pub cortical_context_lookups: CorticalEngineIndexedVector<FGQ::CorticalAreaIndexCountQuant, CorticalContextLookup<FGQ>>,

    // TODO Dimensional cortical layouts vector

    pub neuron_model_neuron_data: NeuronModelNeuronData<FGQ>,

    pub neuron_model_cortical_data: NeuronModelCorticalData<FGQ>,


    pub neuron_potentials: MPQuantMembranePotentials<FGQ>


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

pub struct ConnectomeNeuronIndexOffsetToMPNeuronIndex<FGQ: FeagiGlobalQuantization>
{
    pub float_32_down_offset: FGQ::NeuronIndexCountQuant,
}

impl<FGQ: FeagiGlobalQuantization> ConnectomeNeuronIndexOffsetToMPNeuronIndex<FGQ>
{
    // TODO offset func
}


/// Denotes the last time a specific neuron fired or had an input activity at all. As not all
/// neuron models use this, has its own indexing
#[derive(Clone, Copy)]
pub struct NeuronHistory<FGQ: FeagiGlobalQuantization>
{
    pub burst_index_of_last_input: BurstIndex<FGQ::GlobalBurstIndexQuant>,
    pub burst_index_of_last_firing: BurstIndex<FGQ::GlobalBurstIndexQuant>,
}



/// Contains indexes and offsets for various properties of a cortical_area area. Indexed by
/// Engine Cortical Index
#[derive(Clone, Copy)]
pub struct CorticalContextLookup<FGQ: FeagiGlobalQuantization>
{
    /// Subtract the this from a neurons mp quant index to the get the cortical_area area local index
    pub mp_quant_to_local_neuron_index_offset: FGQ::NeuronIndexCountQuant,
    pub mp_quant_to_neuron_history_index_offset: FGQ::NeuronIndexCountQuant, // Only valid if the neuron model needs history. Otherwise this will just be 0

    pub cortical_layout_index: FGQ::CorticalAreaIndexCountQuant, // Neuron Flags will disclose what type of layout
    pub neuron_model_cortical_data_index: FGQ::CorticalAreaIndexCountQuant,
    // NOTE: Base psp potential is a separate array with 1-1 cortical_area engine index lookup, we don't need it here
}

impl<FGQ: FeagiGlobalQuantization> CorticalContextLookup<FGQ> {
    pub fn get_local_neuron_index(&self, index: NeuronMPIndex<FGQ::NeuronIndexCountQuant>) -> NeuronCorticalLocalIndex<FGQ::NeuronIndexCountQuant>
    {
        NeuronCorticalLocalIndex::from(*index.as_ref() - self.mp_quant_to_local_neuron_index_offset)
    }

    pub fn get_neuron_history_index(&self, index: NeuronMPIndex<FGQ::NeuronIndexCountQuant>) -> NeuronWithHistoryIndex<FGQ::NeuronIndexCountQuant>
    {
        NeuronWithHistoryIndex::from(*index.as_ref() - self.mp_quant_to_local_neuron_index_offset)
    }
}



// TODO needs proper grouping, macro generation?
pub struct NeuronModelNeuronData<FGQ: FeagiGlobalQuantization> {
    pub float_32_feagi_standard_model: NeuronModelIndexedVector<FGQ::NeuronIndexCountQuant, f32>
}

impl<FGQ: FeagiGlobalQuantization> NeuronModelNeuronData<FGQ> {
    //TODO func to get mut impl of neuron model data
}


// TODO needs proper grouping, macro generation?
pub struct NeuronModelCorticalData<FGQ: FeagiGlobalQuantization> {
    pub float_32_feagi_standard_model: NeuronModelIndexedVector<FGQ::CorticalAreaIndexCountQuant, f32>
}

impl<FGQ: FeagiGlobalQuantization> NeuronModelCorticalData<FGQ> {
    //TODO func to get mut impl of neuron model data
}

//endregion