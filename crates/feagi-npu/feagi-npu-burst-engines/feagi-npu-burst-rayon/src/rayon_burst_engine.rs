use core::future::Future;
use crate::rayon_data::RayonData;
use crate::rayon_engine_allocator::RayonEngineAllocator;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use feagi_npu_burst_core::burst_engine_definitions::burst_engine::{BurstEngine, ComposableBurstEngine};
use feagi_npu_burst_core::burst_engine_definitions::burst_phase_output::BurstPhaseOutput;
use feagi_npu_burst_core::burst_engine_definitions::burst_phases::RunBurstPhase;
use feagi_npu_burst_core::burst_engine_definitions::connectome_change_messaging::{EngineConnectomeChangeRequest, EngineConnectomeChangeResponse};
use feagi_npu_burst_core::errors::BurstEngineError;


pub struct RayonBurstEngine<FIQ: FeagiIndexQuantization> {
    data: RayonData<FIQ>,
    allocator: RayonEngineAllocator<FIQ>,
}

impl<FIQ: FeagiIndexQuantization> RayonBurstEngine<FIQ> {
    pub fn new() -> Self {
        Self {
            data: RayonData::new_blank(),
            allocator: RayonEngineAllocator::default(),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> BurstEngine<FIQ> for RayonBurstEngine<FIQ> {
    fn execute_phase(
        &mut self,
        phases: RunBurstPhase,
        burst_index: BurstIndex<FIQ::BurstIndexQuant>,
    ) -> impl Future<Output = Result<BurstPhaseOutput<FIQ>, BurstEngineError>> {
        core::future::ready(Ok(BurstPhaseOutput::NoFurtherActionNeeded))
    }
}

impl<FIQ: FeagiIndexQuantization> ComposableBurstEngine<FIQ> for RayonBurstEngine<FIQ> {
    type Allocator = RayonEngineAllocator<FIQ>;
}
