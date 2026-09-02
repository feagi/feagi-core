use crate::rayon_data::RayonData;
use crate::rayon_engine_change_instructions::RayonEngineChangeInstructions;
use core::future::Future;
use core::marker::PhantomData;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_npu_burst_core::burst_engine_definitions::composable::composable_engine_allocator::ComposableEngineAllocator;
use feagi_npu_burst_core::errors::BurstEngineError;

pub struct RayonEngineAllocator<FIQ: FeagiIndexQuantization> {
    _p: PhantomData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization> ComposableEngineAllocator<FIQ> for RayonEngineAllocator<FIQ> {
    type EngineData = RayonData<FIQ>;
    type ConnectomeChangeInstructions = RayonEngineChangeInstructions<FIQ>;

    fn process_change_instructions(
        &mut self,
        engine_data: &mut Self::EngineData,
        instructions: Self::ConnectomeChangeInstructions,
    ) -> impl Future<Output = Result<Self::ConnectomeChangeInstructions, BurstEngineError>> {
        todo!()
    }
}

impl<FIQ: FeagiIndexQuantization> Default for RayonEngineAllocator<FIQ> {
    fn default() -> Self {
        Self { _p: PhantomData }
    }
}
