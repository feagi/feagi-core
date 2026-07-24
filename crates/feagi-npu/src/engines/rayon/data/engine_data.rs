use crate::engines::rayon::data::model_quantized_data::NeuronModelData;
use crate::engines::rayon::data::sub_structure_data::CorticalNeuronOffsets;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::WrappedQuantizedIndexCount;
use feagi_models::neuron::common_structs::cortical_area_layout::CorticalAreaLayoutDimensional;
use feagi_models::neuron::common_structs::packed_cortical_neuron_flags::PackedCorticalNeuronPhaseFlags;
use feagi_models::neuron::common_structs::packed_cortical_synapse_flags::PackedCorticalSynapseFlags;
use feagi_models::neuron::model_and_quantization::PackedNeuronModelTypeAndQuantization;
use feagi_models::neuron::model_extensions::neuron_history::NeuronModelFullNeuronHistory;
use feagi_models::wrapped_index_collections::{
    CorticalEngineIndex, CorticalEngineIndexedVector, CorticalLayoutIndexedVector,
    NeuronEngineByteIndexedVector, NeuronHistoryIndexedVector,
};
use feagi_models::wrapped_indexes::BurstIndex;

pub struct RayonEngineData<FIQ: FeagiIndexQuantization> {
    /// The current burst index
    pub burst_index: BurstIndex<FIQ::GlobalBurstIndexQuant>,

    // Cortical / Neuron Level Data
    /// INIT - engine cortical indexes indexed by `NeuronEngineByteIndex`, used to get the
    /// `CorticalEngineIndex` for every 8 neurons
    pub cortical_engine_indexes: NeuronEngineByteIndexedVector<FIQ::NeuronIndexCountQuant, CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>>,

    /// Internally indexed by MPModel indexes, All neuron / cortical data in their various models
    /// and quantizations
    pub neuron_model_data: NeuronModelData<FIQ>,

    /// Indexed by `CorticalEngineIndex`, gets a tuple of `PackedNeuronModelTypeAndQuantization`
    /// and `CorticalNeuronPhaseFlags`
    pub cortical_neuron_model_and_quant_and_neuron_properties:
        CorticalEngineIndexedVector<FIQ::CorticalAreaIndexCountQuant, (PackedNeuronModelTypeAndQuantization, PackedCorticalNeuronPhaseFlags)>,
    /// Indexed by `CorticalEngineIndex`, gets the flag for `PackedCorticalSynapseFlags` which are needed for synapse properties
    pub cortical_synapse_properties: CorticalEngineIndexedVector<FIQ::CorticalAreaIndexCountQuant, PackedCorticalSynapseFlags>,
    /// Indexed by `CorticalEngineIndex`, contains various offsets for neuron index conversion via `CorticalNeuronOffsets`
    pub cortical_index_lookups_and_offsets: CorticalEngineIndexedVector<FIQ::CorticalAreaIndexCountQuant, CorticalNeuronOffsets<FIQ>>,

    /// Indexed by `CorticalLayoutIndex`, contains dimensional layout information
    pub cortical_layout_dimensional_data: CorticalLayoutIndexedVector<FIQ::CorticalAreaIndexCountQuant, CorticalAreaLayoutDimensional<FIQ>>,
    // TODO formless
    pub neuron_history_data: NeuronHistoryIndexedVector<FIQ::NeuronIndexCountQuant, NeuronModelFullNeuronHistory<FIQ>>, // TODO Synapse


    






}

impl<FIQ: FeagiIndexQuantization> RayonEngineData<FIQ> {
    pub fn new_empty() -> Self {
        Self {
            burst_index: BurstIndex::QUANT_MAX / (BurstIndex::quant_from_usize(2)),
            cortical_engine_indexes: CorticalEngineIndexedVector::new_empty(),
            neuron_model_data: NeuronModelData::new(),
            cortical_neuron_model_and_quant_and_neuron_properties: (),
            cortical_synapse_properties: (),
            cortical_index_lookups_and_offsets: (),
            cortical_layout_dimensional_data: (),
            neuron_history_data: (),
        }
    }
}

/*
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
    pub neuron_history: NeuronHistoryIndexedVector<FIQ::NeuronIndexCountQuant, NeuronHistoryFull<FIQ>>,

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


 */
