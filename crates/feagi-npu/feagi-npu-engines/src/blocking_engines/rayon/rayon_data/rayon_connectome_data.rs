use feagi_data::quantization_levels::feagi_index_quantization::FeagiGlobalQuantization;
use feagi_npu_common::wrapped_values::{BurstIndex, NeuronMembranePotential};

// TODO add other quantization levels
/// Creates a set of vectors for each decimal quantized type
macro_rules! decimal_quant_group {
    ($(#[$meta:meta])*
    $StructName:ident,
    $QuantizableStruct:ident) => {
        $(#[$meta])*
        struct $StructName <FGQ: FeagiGlobalQuantization>
        {
            pub float_32: Vec<$QuantizableStruct <f32>>,
            _p: core::marker::PhantomData<FGQ>,
        }
    };
}


pub struct RayonConnectomeData<FGQ: FeagiGlobalQuantization>
{
    /// Defines the current burst index
    pub burst_index: BurstIndex<FGQ::GlobalBurstIndexQuant>,

    /// If the burst index just overflowed, set to true. All other times is false
    pub did_burst_index_overflow: bool,






    // /// Per cortical_area area that needs to store the percentage of neurons that fired that burst
    // /// (needed for some downstream synapse types)
    // TODO

}

/// Definitions for custom collections that are grouped in non-abritary ways
//region Sub Collections


decimal_quant_group!(
    /// MP Grouped Neuron Potential Values
    MPQuantMembranePotentials,
    NeuronMembranePotential
);



//endregion


//region SubStructs


/// Denotes the last time a specific neuron fired or had an input activity at all. As not all
/// neuron models use this, has its own indexing
pub struct NeuronHistory<FGQ: FeagiGlobalQuantization>
{
    pub burst_index_of_last_input: BurstIndex<FGQ::GlobalBurstIndexQuant>,
    pub burst_index_of_last_firing: BurstIndex<FGQ::GlobalBurstIndexQuant>,
}



//endregion