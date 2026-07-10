use feagi_data::collections::linear::bitpacked::BitPackedVector;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::QuantizedIndexCountTrait;
use feagi_npu_common::descriptors::cortical_area_descriptors::{CorticalAreaDescriptors, CorticalAreaLayoutDataDimensional, CorticalAreaLayoutDataMemory};
use feagi_npu_common::wrapped_indexes::{BurstIndex, CorticalEngineIndex, CorticalEngineIndexedVector, CorticalLayoutIndexedVector, NeuronEngineByteIndexedVector, NeuronHistoryIndexedVector, NeuronMPIndexedVector, NeuronPSPUniformIndexedVector};
use crate::engines::blocking::rayon::data::model_data_vecs::{CorticalModelDataVecs, NeuronModelDataVecs};
use crate::engines::blocking::rayon::data::sub_structure_data::{CorticalNeuronOffsets, NeuronHistory};



pub struct RayonEngineData<FIQ: FeagiIndexQuantization>
{
    // Global metadata
    pub burst_index: BurstIndex<FIQ::GlobalBurstIndexQuant>,

    // Neuron
    // Engine Level - across the whole burst engine
    pub neuron_cortical_mapping: NeuronEngineByteIndexedVector<FIQ::NeuronIndexCountQuant, CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>>,
    pub bitpacked_neuron_activity: BitPackedVector<FIQ::NeuronIndexCountQuant>,

    // MP Level (Typed) - By the membrane potential index (different quantization)
    pub neuron_fcl: NeuronMPIndexedPotentials<FIQ::NeuronIndexCountQuant>,
    pub neuron_mp: NeuronMPIndexedPotentials<FIQ::NeuronIndexCountQuant>,

    // Model Level (Typed) - By the neuron model and quantization level index
    pub neuron_model_data_vecs: NeuronModelDataVecs<FIQ>,

    // PSP Uniformity Level (Typed) - only neurons that have psp uniformity enabled by their cortical areas
    pub neuron_psp_uniformity_divisors: NeuronPSPUniformIndexedPotentials<FIQ::NeuronIndexCountQuant>,

    // History Level - only neurons that are using models that need neuron history
    pub neuron_history: NeuronHistoryIndexedVector<FIQ::NeuronIndexCountQuant, NeuronHistory<FIQ>>,


    // Cortical
    // Engine Level
    pub cortical_neuron_offsets: CorticalEngineIndexedVector<FIQ::CorticalAreaIndexCountQuant, CorticalNeuronOffsets<FIQ>>,
    pub cortical_descriptors: CorticalEngineIndexedVector<FIQ::CorticalAreaIndexCountQuant, CorticalAreaDescriptors>,

    // Model Level (Typed) - By the neuron model of the cortical area and the quantization level index
    pub cortical_model_data_vecs: CorticalModelDataVecs<FIQ>,

    // Cortical Layout Level (Typed)
    pub cortical_layouts: CorticalLayoutVecs<FIQ>,
}

// region Grouping

// TODO more quant levels
macro_rules! make_dec_quant_vecs {
    ($name:ident, $quant_index:ident, $wrapped_vector:ident) => {
        pub(crate) struct $name<QI: QuantizedIndexCountTrait> {
            pub float_32: $wrapped_vector<QI, f32>,
        }
    };
}

make_dec_quant_vecs!(NeuronMPIndexedPotentials, NeuronIndexCountQuant, NeuronMPIndexedVector);

make_dec_quant_vecs!(NeuronPSPUniformIndexedPotentials, NeuronIndexCountQuant, NeuronPSPUniformIndexedVector);


/// Stores all layouts of cortical areas
pub struct CorticalLayoutVecs<FIQ: FeagiIndexQuantization> {
    pub dimensional: CorticalLayoutIndexedVector<FIQ::CorticalAreaIndexCountQuant, CorticalAreaLayoutDataDimensional<FIQ>>,
    pub memory: CorticalLayoutIndexedVector<FIQ::CorticalAreaIndexCountQuant, CorticalAreaLayoutDataMemory<FIQ>>,
}






//endregion