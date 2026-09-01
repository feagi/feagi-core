use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::errors::BurstEngineError;

#[cfg(feature = "composable")]
/// Actually applies the changes for connectome edits to the given burst engines connectome data
pub trait ComposableEngineAllocator<FIQ: FeagiIndexQuantization> {
    
    /// The struct that is holding the connectome data of the engine
    type EngineData;
    
    /// The struct that holds all the instructions, step by step, for the connectome changes requested
    type ConnectomeChangeInstructions: ComposableEngineConnectomeChangeInstructions<FIQ>;
    
    /// Apply changes to connectome
    fn process_change_instructions(&mut self, engine_data: &mut Self::EngineData, instructions: Self::ConnectomeChangeInstructions) 
        -> impl core::future::Future<Output = Result<Self::ConnectomeChangeInstructions, BurstEngineError>>;
    
}

#[cfg(feature = "composable")]
/// The actual struct holding the instructions specific for this engine
pub trait ComposableEngineConnectomeChangeInstructions<FIQ: FeagiIndexQuantization> {}