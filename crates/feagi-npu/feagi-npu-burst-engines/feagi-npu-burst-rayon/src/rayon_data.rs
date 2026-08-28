use feagi_data::generic_collections::generic_par_data::linear::ParDataVector;
use feagi_models::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use feagi_models::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use feagi_npu_burst_core::wrapped_values::{EngineCorticalIndex, EngineNeuronIndex};



pub struct RayonData<NPUIQ, BEIQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization
{

    //region Kernel Start Points

    /// Engine Cortical Indexes indexed by `NeuronEngineIndex`, used to get the cortical index for each neuron
    pub init_engine_neurons_to_cortical_indexes: ParDataVector<EngineNeuronIndex<BEIQ::NeuronIndexQuant>, EngineCorticalIndex<BEIQ::CorticalAreaIndexCountQuant>>,


    //endregion



    //region Misc Single Data
    /// The current burst index
    pub burst_index: BurstIndex<NPUIQ::BurstIndexQuant>,

    //endregion








}
