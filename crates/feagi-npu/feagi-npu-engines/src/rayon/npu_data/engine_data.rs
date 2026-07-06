use feagi_structures::feagi_data::quantizable_linear::wrappers::QuantizedElementWrapperBase;
use feagi_structures::feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantization;
use feagi_structures::feagi_data::SupportsUintOps;
use crate::neural_processing_unit_data_structures::burst_engine::engines::rayon::npu_data::npu_structured::burst_engine_global::{BurstEngineNeuronIndexWithQuant, CorticalContextLookup, CorticalLayouts, FCLMappingsToFCLC, NeuronHistory, SynapseDef, SynapseRangeMappingFromNeuron};
use crate::neural_processing_unit_data_structures::burst_engine::engines::rayon::npu_data::npu_structured::grouped_by_mp_quant::{EngineNeuronIndexOffsetsToMPQuantNeuronIndex, MPQuantBasePostSynapticPotentials, MPQuantNeuronFCLValues, MPQuantNeuronMembranePotentialValues};
use crate::neural_processing_unit_data_structures::burst_engine::model_implementations::neuron_models::feagi_standard::data::{FeagiStandardModelCorticalDataCPU, FeagiStandardModelNeuronDataCPU};
use crate::neural_processing_unit_data_structures::burst_engine::model_implementations::neuron_models::feagi_standard::quantization::FeagiStandardModelStandard32BitQuant;
use crate::neural_processing_unit_data_structures::wrappers::{NPUWrappedBurstEngineBurstIndex, NPUWrappedCorticalAreaBurstEngineIndex};


pub struct BurstEngineDataRayon<FGQ: FeagiGlobalQuantization>
{
    //region Global Metadata, Converters and Flags

    /// Defines the current burst index
    pub burst_index: NPUWrappedBurstEngineBurstIndex<FGQ::GlobalBurstIndexQuant>,


    /// If the burst index just overflowed, set to true. All other times is false
    pub did_burst_index_overflow: bool,


    /// Stores offsets needed to subtract from Engine Neuron indexes to MP Quant neuron indexes.
    /// You will need to have the quant / mp quant flag for this O(0) lookup.
    pub engine_neuron_index_offsets_to_mp_quant_neuron_index:
        EngineNeuronIndexOffsetsToMPQuantNeuronIndex<FGQ>,


    // region Global Indexed Neuron / Cortical Data

    // By Neuron model with activity history (not really an index)
    /// For all neurons whose cortical_area areas define that this value is needed for their neuron
    /// model, the last burst index that the neuron received an input and the last index that it
    /// fired at
    pub neuron_history: Vec<NeuronHistory<FGQ>>,


    // By cortical_area area checking neuron activity index (not really an index)
    /// Per cortical_area area that needs to store the percentage of neurons that fired that burst
    /// (needed for some downstream synapse types)
    pub percent_neurons_firing_this_burst: Vec<u8>, // TODO percentage type!


    // By Consolidated Burst Engine Index (Not really an index)
    /// Filtered neurons that have received a potential input. Contains burst engine level
    /// neuron indexes with the quant flag for rapid mp quant neuron lookups while still
    /// being able to be iterated on the global level
    pub consolidated_neurons_with_fcl:
        Vec<BurstEngineNeuronIndexWithQuant<FGQ>>,
    
    
    

    
    
    //endregion

    //endregion


    //region Indexed By Cortical Area

    // By Engine Cortical Index
    /// For a given cortical_area area, has index offsets and indexes to other relevant data
    /// for that cortical_area area
    pub cortical_context_lookups: Vec<CorticalContextLookup<FGQ>>,

    // By Cortical Layout Indexes
    /// Grouped by type of cortical_area layout, stores a vector of all the different values for the
    /// different possible cortical_area layouts, to describe how neurons are layed out
    /// (dimensional or otherwise)
    /// Does NOT contain the base post synaptic potential!
    pub cortical_layouts: CorticalLayouts<FGQ>,

    // By MP Quant Cortical Index

    pub base_postsynaptic_potentials: MPQuantBasePostSynapticPotentials,


    // By Neuron Model Quant Index

    // TODO have proper grouping!
    pub neuron_model_cortical_data: Vec<FeagiStandardModelCorticalDataCPU<FeagiStandardModelStandard32BitQuant>>,

    pub synapses_ranges_from_neurons: Vec<SynapseRangeMappingFromNeuron<FGQ>>,

    pub synapse_def: Vec<SynapseDef>,

    //endregion


    //region Indexed By Neuron



    // By Neuron FCL FCLC Consolidation Index (Not really an index)

    pub fcl_mappings_to_FCLC: Vec<FCLMappingsToFCLC<FGQ>>,

    // By Neurons with Synapse Range Index (Not really an index)

    // By MP Quant grouped Index
    /// Grouped by the quantization of the membrane potential, the FCL input of all neurons
    pub neuron_fcls: MPQuantNeuronFCLValues,
    /// Grouped by the quantization of the membrane potential, the neuron potential of all neurons
    pub neuron_potentials: MPQuantNeuronMembranePotentialValues,
    // TODO should we include flags here? or keep it for the cortical_area?
    // TODO we should have different quant groups?
    ///  for each engine/?mp_quant neuron index, get the engine cortical_area index
    pub neuron_engine_cortical_indexes: Vec<NPUWrappedCorticalAreaBurstEngineIndex<FGQ::CorticalAreaIndexCountQuant>>,


    // By Neuron Model grouped Index

    // TODO have proper grouping!
    pub neuron_model_neuron_data: Vec<FeagiStandardModelNeuronDataCPU<FeagiStandardModelStandard32BitQuant>>,
    
    // By Burst Engine Index
    //neuron_downstream_synapse_mapping_ranges: X,


    //endregion

    //region Indexed by Synapse
    
    
    
    //endregion
    
}


impl<FGQ: FeagiGlobalQuantization> BurstEngineDataRayon<FGQ>
{
    pub fn new() -> Self {
        Self {
            burst_index: NPUWrappedBurstEngineBurstIndex::QUANT_MAX / NPUWrappedBurstEngineBurstIndex::wrap(  FGQ::GlobalBurstIndexQuant::from_usize_unchecked(2)),
            did_burst_index_overflow: false,
            engine_neuron_index_offsets_to_mp_quant_neuron_index: EngineNeuronIndexOffsetsToMPQuantNeuronIndex { float_32: Default::default() },
            neuron_history: vec![],
            percent_neurons_firing_this_burst: vec![],
            consolidated_neurons_with_fcl: vec![],
            cortical_context_lookups: vec![],
            cortical_layouts: CorticalLayouts { dimensional: vec![] },
            base_postsynaptic_potentials: MPQuantBasePostSynapticPotentials { float_32: vec![] },
            neuron_model_cortical_data: vec![],
            synapses_ranges_from_neurons: vec![],
            synapse_def: vec![],
            fcl_mappings_to_FCLC: vec![],
            neuron_fcls: MPQuantNeuronFCLValues { float_32: vec![] },
            neuron_potentials: MPQuantNeuronMembranePotentialValues { float_32: vec![] },
            neuron_engine_cortical_indexes: vec![],
            neuron_model_neuron_data: vec![],
        }
    }
}




