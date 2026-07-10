use core::marker::PhantomData;
use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialCPUQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_npu_common::wrapped_indexes::{CorticalModelIndexedVector, NeuronModelIndexedVector};
use feagi_npu_models::neuron_models::feagi_standard::data::{FeagiStandardModelCorticalData, FeagiStandardModelNeuronData};
use feagi_npu_models::neuron_models::feagi_standard::quantization::FeagiStandardModelStandard32BitQuant;
use feagi_npu_models::neuron_models::neuron_model_traits::neuron_model_data::{NeuronModelCorticalData, NeuronModelNeuronData};

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
    pub feagi_standard_float_32: ModelQuantTypedNeuronData
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
    CPQ: CorticalPotentialCPUQuantization,
    NMCD: NeuronModelCorticalData<CPQ>,

{
    cortical_areas: CorticalModelIndexedVector<FIQ::CorticalAreaIndexCountQuant, NMCD>,
    _p: PhantomData<CPQ>,
}


/// Holds the data of neurons of a certain model and quant
pub struct ModelQuantTypedNeuronData<FIQ, CPQ, NMND >
where
    FIQ: FeagiIndexQuantization,
    CPQ: CorticalPotentialCPUQuantization,
    NMND: NeuronModelNeuronData<CPQ>,
    
{
    neurons: NeuronModelIndexedVector<FIQ::NeuronIndexCountQuant, NMND>,
    _p: PhantomData<CPQ>,
}

