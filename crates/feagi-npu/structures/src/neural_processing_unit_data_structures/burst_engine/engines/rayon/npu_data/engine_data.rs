use feagi_structures::feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantization;
use crate::neural_processing_unit_data_structures::burst_engine::engines::rayon::npu_data::npu_structured::burst_engine_global::{BurstEngineNeuronIndexWithQuant, CorticalContextLookup, CorticalLayouts, FCLMappingsToFCLC, NeuronHistory};
use crate::neural_processing_unit_data_structures::burst_engine::engines::rayon::npu_data::npu_structured::grouped_by_mp_quant::{EngineNeuronIndexOffsetsToMPQuantNeuronIndex, MPQuantBasePostSynapticPotentials, MPQuantNeuronFCLValues, MPQuantNeuronMembranePotentialValues};
use crate::neural_processing_unit_data_structures::burst_engine::engines::rayon::neuron_models::feagi_standard::data::{FeagiStandardModelCorticalDataCPU, FeagiStandardModelNeuronDataCPU};
use crate::neural_processing_unit_data_structures::burst_engine::engines::rayon::neuron_models::feagi_standard::quantization::FeagiStandardModelStandard32BitQuant;
use crate::neural_processing_unit_data_structures::wrappers::{NPUWrappedBurstEngineBurstIndex, NPUWrappedCorticalAreaBurstEngineIndex};


pub(crate) struct BurstEngineDataRayon<FGQ: FeagiGlobalQuantization>
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
    /// For all neurons whose cortical areas define that this value is needed for their neuron
    /// model, the last burst index that the neuron received an input and the last index that it
    /// fired at
    pub neuron_history: Vec<NeuronHistory<FGQ>>,


    // By cortical area checking neuron activity index (not really an index)
    /// Per cortical area that needs to store the percentage of neurons that fired that burst
    /// (needed for some downstream synapse types)
    pub percent_neurons_firing_this_burst: Vec<u8>, // TODO percentage type!


    // By Consolidated Burst Engine Index (Not really an index)
    /// Filtered neurons that have received a potential input. Contains burst engine level
    /// neuron indexes with the quant flag for rapid mp quant neuron lookups while still
    /// being able to be iterated on the global level
    pub consolidated_neurons_with_fcl:
        Vec<BurstEngineNeuronIndexWithQuant<FGQ>>,
    
    
    // By Engine Cortical Index
    /// For a given cortical area, has index offsets and indexes to other relevant data
    /// for that cortical area
    pub cortical_context_lookups: Vec<CorticalContextLookup<FGQ>>,
    
    
    //endregion

    //endregion


    //region Indexed By Cortical Area


    // By Cortical Layout Indexes
    /// Grouped by type of cortical layout, stores a vector of all the different values for the
    /// different possible cortical layouts, to describe how neurons are layed out
    /// (dimensional or otherwise)
    /// Does NOT contain the base post synaptic potential!
    pub cortical_layouts: CorticalLayouts<FGQ>,

    // By MP Quant Cortical Index

    pub base_postsynaptic_potentials: MPQuantBasePostSynapticPotentials,


    // By Neuron Model Quant Index

    // TODO have proper grouping!
    pub neuron_model_cortical_data: Vec<FeagiStandardModelCorticalDataCPU<FeagiStandardModelStandard32BitQuant>>,




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
    // TODO should we include flags here? or keep it for the cortical?
    // TODO we should have different quant groups?
    ///  for each engine/?mp_quant neuron index, get the engine cortical index
    pub neuron_engine_cortical_indexes: Vec<NPUWrappedCorticalAreaBurstEngineIndex<FGQ::CorticalAreaIndexCountQuant>>,


    // By Neuron Model grouped Index

    // TODO have proper grouping!
    pub neuron_model_neuron_data: Vec<FeagiStandardModelNeuronDataCPU<FeagiStandardModelStandard32BitQuant>>,


    
    // By Burst Engine Index
    //neuron_downstream_synapse_mapping_ranges: X,


    //endregion


















}



