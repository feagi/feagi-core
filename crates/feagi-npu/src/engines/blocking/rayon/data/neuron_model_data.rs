use core::marker::PhantomData;
use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::neuron::models::feagi_advanced::data::{FeagiStandardModelCorticalData, FeagiStandardModelNeuronData};
use feagi_models::neuron::models::feagi_advanced::quantization::FeagiStandardModelStandard32BitQuant;
use feagi_models::neuron::shared::data::{NeuronModelCorticalData, NeuronModelNeuronData};
use feagi_npu_common::wrapped_indexes::{CorticalModelIndexedVector, NeuronModelIndexedVector};

// TODO this should be macroized
// NOTE: this only stores the quantization relevant to the CPU!
/// Container to make holding neuron model specific data less annoying
pub struct NeuronModelDataContainer<FIQ: FeagiIndexQuantization> {
    pub cortical_feagi_standard_float_32: ModelQuantTypedCorticalData
    <
        FIQ, 
        FeagiStandardModelStandard32BitQuant,
        FeagiStandardModelCorticalData<FeagiStandardModelStandard32BitQuant>,
    >,
    pub neuron_feagi_standard_float_32: ModelQuantTypedNeuronData
    <
        FIQ, 
        FeagiStandardModelStandard32BitQuant,
        FeagiStandardModelNeuronData<FeagiStandardModelStandard32BitQuant>
    >,
    
    _p: PhantomData<FIQ>,
}

/// Holds the data of cortical areas of a certain model and quant
pub struct ModelQuantTypedCorticalData<FIQ, CPQ, NMCD >
where
    FIQ: FeagiIndexQuantization,
    CPQ: CorticalPotentialQuantization,
    NMCD: NeuronModelCorticalData<CPQ>,

{
    pub cortical_areas: CorticalModelIndexedVector<FIQ::CorticalAreaIndexCountQuant, NMCD>,
    _p: PhantomData<CPQ>,
}


/// Holds the data of neurons of a certain model and quant
pub struct ModelQuantTypedNeuronData<FIQ, CPQ, NMND >
where
    FIQ: FeagiIndexQuantization,
    CPQ: CorticalPotentialQuantization,
    NMND: NeuronModelNeuronData<CPQ>,
    
{
    pub neurons: NeuronModelIndexedVector<FIQ::NeuronIndexCountQuant, NMND>,
    _p: PhantomData<CPQ>,
}

