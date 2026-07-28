use feagi_data::collections::linear::bitpacked::BitPackedVector;
use crate::engines::rayon::data::model_quantized_data::NeuronModelData;
use crate::engines::rayon::data::sub_structure_data::{CorticalIndexLookupTable, NeuronIndexLookupTable};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::WrappedQuantizedIndexCount;
use feagi_models::neuron::common_structs::cortical_area_layout::CorticalAreaLayoutDimensional;
use feagi_models::neuron::common_structs::neuron_runtime_flags::NeuronRuntimeFlags;
use feagi_models::neuron::common_structs::packed_cortical_neuron_flags::PackedCorticalNeuronPhaseFlags;
use feagi_models::neuron::common_structs::packed_cortical_synapse_flags::PackedCorticalSynapseFlags;
use feagi_models::neuron::model_and_quantization::PackedNeuronModelTypeAndQuantization;
use feagi_models::neuron::model_extensions::neuron_history::NeuronModelFullNeuronHistory;
use feagi_models::wrapped_index_collections::{CorticalEngineIndex, CorticalEngineIndexedVector, CorticalLayoutIndexedVector, NeuronEngineByteIndexedVector, NeuronEngineIndexedVector, NeuronHistoryIndexedVector};
use feagi_models::wrapped_indexes::BurstIndex;
use crate::engines::rayon::data::quantized_data::NeuronQuantizedData;

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

    /// Contains per neuron data quantized to the membrane potential levels
    pub neuron_membrane_data: NeuronQuantizedData<FIQ>,

    /// Indexed by `CorticalEngineIndex`, gets a tuple of `PackedNeuronModelTypeAndQuantization`
    /// and `CorticalNeuronPhaseFlags`
    pub cortical_neuron_model_and_quant_and_neuron_properties:
        CorticalEngineIndexedVector<FIQ::CorticalAreaIndexCountQuant, (PackedNeuronModelTypeAndQuantization, PackedCorticalNeuronPhaseFlags)>,
    /// Indexed by `CorticalEngineIndex`, gets the flag for `PackedCorticalSynapseFlags` which are needed for synapse properties
    pub cortical_synapse_properties: CorticalEngineIndexedVector<FIQ::CorticalAreaIndexCountQuant, PackedCorticalSynapseFlags>,
    /// Indexed by `CorticalEngineIndex`, contains various indexes for other cortical level properties this cortical area may have
    pub cortical_index_lookup_table: CorticalEngineIndexedVector<FIQ::CorticalAreaIndexCountQuant, CorticalIndexLookupTable<FIQ>>,
    /// Indexed by `CorticalEngineIndex`, contains the number of neurons within that cortical area
    pub cortical_neuron_count: CorticalEngineIndexedVector<FIQ::CorticalAreaIndexCountQuant, FIQ::NeuronIndexCountQuant>,
    /// Indexed by `CorticalEngineIndex`, contains various offsets for neuron index conversion via the `NeuronIndexLookupTable`
    pub cortical_neuron_index_lookup_table: CorticalEngineIndexedVector<FIQ::CorticalAreaIndexCountQuant, NeuronIndexLookupTable<FIQ>>,

    /// Indexed by `CorticalLayoutIndex`, contains dimensional layout information
    pub cortical_layout_dimensional_data: CorticalLayoutIndexedVector<FIQ::CorticalAreaIndexCountQuant, CorticalAreaLayoutDimensional<FIQ>>,
    // TODO formless (NOTE: uniquely coupldnt we just use cortical_neuron_count?)

    /// Indexed by `NeuronEngineByteIndex`, contains the per neuron runtime flags
    pub neuron_runtime_flags: NeuronEngineIndexedVector<FIQ::NeuronIndexCountQuant, NeuronRuntimeFlags>,

    /// Indexed by `NeuronEngineByteIndex` (indirectly) and `NeuronEngineByteIndex`, bitpacked information for if a neuron is firing in this burst
    pub neuron_is_firing: BitPackedVector<FIQ::NeuronIndexCountQuant>,


    /// Indexed by `NeuronHistoryIndex`, for neurons wth it, is the per neuron history of that neuron
    pub neuron_history_data: NeuronHistoryIndexedVector<FIQ::NeuronIndexCountQuant, NeuronModelFullNeuronHistory<FIQ>>, // TODO Synapse









}

impl<FIQ: FeagiIndexQuantization> RayonEngineData<FIQ> {
    pub fn new_empty() -> Self {
        Self {
            burst_index: BurstIndex::QUANT_MAX / (BurstIndex::quant_from_usize(2)),
            cortical_engine_indexes: NeuronEngineByteIndexedVector::new_empty(),
            neuron_model_data: NeuronModelData::new(),
            cortical_neuron_model_and_quant_and_neuron_properties: CorticalEngineIndexedVector::new_empty(),
            cortical_synapse_properties: CorticalEngineIndexedVector::new_empty(),
            cortical_neuron_index_lookup_table: CorticalEngineIndexedVector::new_empty(),
            cortical_layout_dimensional_data: CorticalLayoutIndexedVector::new_empty(),
            neuron_history_data: NeuronHistoryIndexedVector::new_empty(),
        }
    }
}

