use crate::engines::rayon::data::neuron::model_quantized_data::NeuronModelData;
use crate::engines::rayon::data::neuron::neuron_sub_data::{CorticalIndexLookupTable, NeuronIndexLookupTable};
use crate::engines::rayon::data::neuron::potential_quantized_data::NeuronQuantizedData;
use crate::flags::cortical_runtime_flags::CorticalRuntimeFlags;
use crate::flags::neuron_runtime_flags::NeuronRuntimeFlags;
use feagi_data::collections::linear::bitpacked::BitPackedVector;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::{QuantizedElementBase, QuantizedIndexCountTrait};
use feagi_models::neuron::cortical_area_layout::CorticalAreaLayoutDimensional;
use feagi_models::neuron::model_capabilities::neuron_history::NeuronModelFullNeuronHistory;
use feagi_models::neuron::model_generated::model_type_and_quantization::NeuronModelTypeAndQuantizationPacked;
use feagi_models::wrapped_index_collections::{CorticalEngineIndex, CorticalEngineIndexedVector, CorticalLayoutIndexedVector, MappingEntryEngineIndex, MappingEntryEngineIndexedVector, NeuronEngineIndexedVector, NeuronHistoryIndexedVector, NeuronMPIndex, SynapseEngineIndexedVector};
use feagi_models::wrapped_indexes::BurstIndex;
use crate::engines::rayon::data::synapse::model_quantized_data::SynapseModelData;
use crate::engines::rayon::data::synapse::synapse_sub_data::{CorticalMappingEntryIndexLookupTable, CorticalMappingEntryProperties};

pub struct RayonEngineData<FIQ: FeagiIndexQuantization> {
    /// The current burst index
    pub burst_index: BurstIndex<FIQ::GlobalBurstIndexQuant>,

    //region Cortical / Neuron Level Data
    /// INIT - Engine Cortical Indexes indexed by `NeuronEngineIndex`, used to get the cortical index for each neuron
    pub cortical_engine_indexes: NeuronEngineIndexedVector<FIQ::NeuronIndexQuant, CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>>,

    /// Internally indexed by MPModel indexes, All neuron / cortical data in their various models
    /// and quantizations
    pub neuron_model_data: NeuronModelData<FIQ>,

    /// Contains per neuron data quantized to the membrane potential levels
    pub neuron_membrane_data: NeuronQuantizedData<FIQ>,

    /// Indexed by `CorticalEngineIndex`, gets a tuple of `NeuronModelTypeAndQuantizationPacked`
    /// and `CorticalRuntimeFlags`
    pub cortical_neuron_model_and_quant_and_neuron_properties:
        CorticalEngineIndexedVector<FIQ::CorticalAreaIndexCountQuant, (NeuronModelTypeAndQuantizationPacked, CorticalRuntimeFlags)>,
    /// Indexed by `CorticalEngineIndex`, contains various indexes for other cortical level properties this cortical area may have
    pub cortical_index_lookup_table: CorticalEngineIndexedVector<FIQ::CorticalAreaIndexCountQuant, CorticalIndexLookupTable<FIQ>>,
    /// Indexed by `CorticalEngineIndex`, contains the number of neurons within that cortical area
    pub cortical_neuron_count: CorticalEngineIndexedVector<FIQ::CorticalAreaIndexCountQuant, FIQ::NeuronIndexQuant>,
    /// Indexed by `CorticalEngineIndex`, contains various offsets for neuron index conversion via the `NeuronIndexLookupTable`
    pub cortical_neuron_index_lookup_table: CorticalEngineIndexedVector<FIQ::CorticalAreaIndexCountQuant, NeuronIndexLookupTable<FIQ>>,

    /// Indexed by `CorticalLayoutIndex`, contains dimensional layout information
    pub cortical_layout_dimensional_data: CorticalLayoutIndexedVector<FIQ::CorticalAreaIndexCountQuant, CorticalAreaLayoutDimensional<FIQ>>,
    // TODO formless (NOTE: uniquely couldnt we just use cortical_neuron_count?)
    /// Indexed by `NeuronEngineIndex`, contains the per neuron runtime flags
    pub neuron_runtime_flags: NeuronEngineIndexedVector<FIQ::NeuronIndexQuant, NeuronRuntimeFlags>,

    /// Indexed by `NeuronEngineByteIndex` (indirectly) and `NeuronEngineByteIndex`, bitpacked information for if a VOXEL is firing in this burst
    pub neuron_voxel_is_firing: BitPackedVector<FIQ::NeuronIndexQuant>,

    /// Indexed by `NeuronHistoryIndex`, for neurons wth it, is the per neuron history of that neuron
    pub neuron_history_data: NeuronHistoryIndexedVector<FIQ::NeuronIndexQuant, NeuronModelFullNeuronHistory<FIQ>>,
    //endregion

    //region Synapse data
    /// INIT - Cortical Mapping Entries indexed by `SynapseEngineIndex`
    pub cortical_mapping_entry_indexes: SynapseEngineIndexedVector<FIQ::SynapseIndexCountQuant, MappingEntryEngineIndex<FIQ::CorticalMappingEntryIndexCountQuant>>,
    
    /// Quantized and per model data for all cortical mapping entries and their synapses
    pub synapse_model_data: SynapseModelData<FIQ>,

    /// Retains various properties of a cortical mapping entry, such as start/end mp quants, flags, and its own model / quant, indexed by `MappingEntryEngineIndex`
    pub cortical_mapping_entry_properties: MappingEntryEngineIndexedVector<FIQ::CorticalMappingEntryIndexCountQuant, CorticalMappingEntryProperties>,

    pub cortical_mapping_index_lookup_table: MappingEntryEngineIndexedVector<FIQ::CorticalMappingEntryIndexCountQuant, CorticalMappingEntryIndexLookupTable<FIQ>>,

    
    


    /// the MP indexes of the source and destination (in that order) neurons of a given synapse. Destination may be from the FCL or FCLC but source is always from the MP
    pub synapse_source_destination_mp_neuron_indexes: SynapseEngineIndexedVector<FIQ::SynapseIndexCountQuant, (NeuronMPIndex<FIQ::NeuronIndexQuant>, NeuronMPIndex<FIQ::NeuronIndexQuant>)>,




    //endregion
}

// Not `#[derive(Default)]`: that would require `FIQ: Default`, but `FIQ` is only ever used
// through its associated types here (it's a zero-sized quantization-level marker). Every field
// below already has its own `Default` impl that doesn't require `FIQ: Default`.
impl<FIQ: FeagiIndexQuantization> Default for RayonEngineData<FIQ> {
    fn default() -> Self {
        Self {
            burst_index: Default::default(),
            cortical_engine_indexes: Default::default(),
            neuron_model_data: Default::default(),
            neuron_membrane_data: Default::default(),
            cortical_neuron_model_and_quant_and_neuron_properties: Default::default(),
            cortical_index_lookup_table: Default::default(),
            cortical_neuron_count: Default::default(),
            cortical_neuron_index_lookup_table: Default::default(),
            cortical_layout_dimensional_data: Default::default(),
            neuron_runtime_flags: Default::default(),
            neuron_voxel_is_firing: BitPackedVector::new_uniform(FIQ::NeuronIndexQuant::QUANT_ZERO, false),
            neuron_history_data: Default::default(),
            cortical_mapping_entry_indexes: Default::default(),
            synapse_model_data: Default::default(),
            cortical_mapping_entry_properties: Default::default(),
            cortical_mapping_index_lookup_table: Default::default(),
            synapse_source_destination_mp_neuron_indexes: Default::default(),
        }
    }
}

