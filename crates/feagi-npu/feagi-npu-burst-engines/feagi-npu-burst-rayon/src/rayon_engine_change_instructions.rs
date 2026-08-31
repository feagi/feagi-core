use core::marker::PhantomData;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_npu_burst_core::burst_engine_definitions::composable_engine_allocator::ComposableEngineConnectomeChangeInstructions;

pub struct RayonEngineChangeInstructions<FIQ: FeagiIndexQuantization>
{
    _p: PhantomData<FIQ>
}

impl<FIQ: FeagiIndexQuantization> ComposableEngineConnectomeChangeInstructions<FIQ> for RayonEngineChangeInstructions<FIQ> {
    
}