use crate::engines::blocking::rayon::data::sub_structure_data::{CorticalContext, CorticalNeuronOffsets};
use feagi_data::collections::linear::bitpacked::BitPackedVector;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::QuantizedIndexCountTrait;
use feagi_models::cortical_area_layout::{CorticalAreaLayoutDataDimensional, CorticalAreaLayoutDataMemory};
use feagi_models::neuron_history::NeuronHistory;
use feagi_npu_common::npu_encoded_data::cortical_area_runtime_flags::CorticalAreaRuntimeFlags;
use feagi_npu_common::npu_encoded_data::neuron_runtime_flags::NeuronRuntimeFlags;
use feagi_models::burst_index::{BurstIndex, CorticalEngineIndex, CorticalEngineIndexedVector, CorticalLayoutIndexedVector, NeuronEngineByteIndexedVector, NeuronEngineIndexedVector, NeuronHistoryIndexedVector, NeuronMPIndexedVector, NeuronPSPUniformIndexedVector};
use crate::engines::blocking::rayon::data::neuron_model_data::NeuronModelDataContainer;

pub struct RayonEngineData<FIQ: FeagiIndexQuantization> {
    // Global metadata
    pub burst_index: BurstIndex<FIQ::GlobalBurstIndexQuant>,
    /// Contains all the actual model data (IE trained data)
    pub neuron_model_data_container: NeuronModelDataContainer<FIQ>,


    // Neuron
    // Engine Level - across the whole burst engine but 
    pub neuron_cortical_mapping: NeuronEngineByteIndexedVector<FIQ::NeuronIndexCountQuant, CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>>,
    pub neuron_runtime_flags: NeuronEngineIndexedVector<FIQ::NeuronIndexCountQuant, NeuronRuntimeFlags>,
    pub bitpacked_neuron_activity: BitPackedVector<FIQ::NeuronIndexCountQuant>,

    // MP Level (Typed) - By the membrane potential index (different quantization)
    pub neuron_fcl: NeuronMPIndexedPotentials<FIQ::NeuronIndexCountQuant>,
    pub neuron_mp: NeuronMPIndexedPotentials<FIQ::NeuronIndexCountQuant>,

    // PSP Uniformity Level (Typed) - only neurons that have psp uniformity enabled by their cortical areas
    pub neuron_psp_uniformity_divisors: NeuronPSPUniformIndexedPotentials<FIQ::NeuronIndexCountQuant>,

    // History Level - only neurons that are using models that need neuron history
    pub neuron_history: NeuronHistoryIndexedVector<FIQ::NeuronIndexCountQuant, NeuronHistory<FIQ>>,

    // Cortical
    // Engine Level - Cortical areas relative to the burst engine
    pub cortical_neuron_offsets: CorticalEngineIndexedVector<FIQ::CorticalAreaIndexCountQuant, CorticalNeuronOffsets<FIQ>>,
    pub cortical_contexts: CorticalEngineIndexedVector<FIQ::CorticalAreaIndexCountQuant, CorticalContext<FIQ>>,
    pub cortical_runtime_flags: CorticalEngineIndexedVector<FIQ::CorticalAreaIndexCountQuant, CorticalAreaRuntimeFlags>,
    

    // Cortical Layout Level (Typed)
    pub cortical_layouts: CorticalLayoutVecs<FIQ>,
}

// region Grouping

// TODO more quant levels (CPU)
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
