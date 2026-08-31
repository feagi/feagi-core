use feagi_data::generic_collections::generic_par_data::linear::ParDataVector;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use feagi_npu_burst_core::wrapped_values::{EngineCorticalIndex, EngineNeuronIndex};



pub struct RayonData<FIQ>
where
    FIQ: FeagiIndexQuantization,
{

    //region Kernel Start Points

    /// Engine Cortical Indexes indexed by `NeuronEngineIndex`, used to get the cortical index for each neuron
    pub init_engine_neurons_to_cortical_indexes: ParDataVector<EngineNeuronIndex<FIQ::NeuronIndexQuant>, EngineCorticalIndex<FIQ::CorticalAreaIndexCountQuant>>,


    //endregion



    //region Misc Single Data
    /// The current burst index
    pub burst_index: BurstIndex<FIQ::BurstIndexQuant>,

    //endregion
    
}


impl<FIQ: FeagiIndexQuantization> RayonData<FIQ> {
    pub fn new_blank() -> RayonData<FIQ> {
        todo!()
    }
}