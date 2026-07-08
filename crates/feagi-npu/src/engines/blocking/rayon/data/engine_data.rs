use feagi_data::quantization_levels::feagi_index_quantization::FeagiGlobalQuantization;
use feagi_npu_common::wrapped_indexes::BurstIndex;

pub struct RayonEngineData<FGQ: FeagiGlobalQuantization>
{
    // Global metadata
    
    pub burst_index: BurstIndex<FGQ::GlobalBurstIndexQuant>,
    
    
    
    // Neuron / Cortical Area
    
    pub neuron_fcl: Vec<f32>,
    pub bitpacked_neuron_activity: Vec<u8>, // TODO proper bitpacked vector
    pub neuron_mp: Vec<f32>,
    
    
    
    
}