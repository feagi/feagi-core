use feagi_data::nested_channels::channels_flume::OuterFlumeChannelPair;
use feagi_models::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantizationQuantizationNormal;
use feagi_models::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;
use feagi_npu_burst_engines::ComposableBurstEngineEnum;


pub type BurstEngineWorkerCommandTx = OuterFlumeChannelPair<>



pub fn composable_burst_engine_worker<NPUIQ: NeuronProcessingUnitIndexQuantization>(
    mut burst_engine: ComposableBurstEngineEnum<NPUIQ, BurstEngineIndexQuantizationQuantizationNormal>, // TODO BEIQ quant enum!
    inner_command_channels: ()
    
)
{
    
    loop {
        
        match command {
            
            // Arm Break
            
            
            // Arm Run Phases
            
            
            // Arm Run Edits
            
            
            
            
        }
        
    }
    
}
