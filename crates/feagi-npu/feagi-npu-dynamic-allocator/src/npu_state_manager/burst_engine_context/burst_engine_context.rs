use feagi_data::collections::index_range_managers::index_manager::IndexManager;
use feagi_data::neurons::DimensionalCorticalArea4DDimensions;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// A given index for a burst engine directly managed by this NPU. Is an u8 since if you have
/// more than 256 GPUs attached to one motherboard, you are doing something wrong
pub type BurstEngineIndex = u8;

pub struct BurstEngineContext<FGQ: FeagiIndexQuantization> {
    cortical_index_manager: IndexManager<FGQ::CorticalAreaIndexCountQuant>
}

impl<FGQ: FeagiIndexQuantization> BurstEngineContext<FGQ> {
    
    
    
    
    
    
    fn add_dimensional_cortical_area(
        &mut self, 
        dimensions: DimensionalCorticalArea4DDimensions<FGQ::CorticalAreaIndexCountQuant>,
        /* iterator to spawn neurons*/
        
    )
}