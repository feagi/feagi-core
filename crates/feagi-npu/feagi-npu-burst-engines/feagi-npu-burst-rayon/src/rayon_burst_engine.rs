use std::future::Future;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_npu_burst_core::burst_engine_definitions::burst_engine::{BurstEngine, ComposableBurstEngine};
use feagi_npu_burst_core::burst_engine_definitions::burst_phase_output::BurstPhaseOutput;
use feagi_npu_burst_core::burst_engine_definitions::burst_phases::RunBurstPhase;
use feagi_npu_burst_core::burst_engine_definitions::connectome_change_messaging::{EngineConnectomeChangeRequest, EngineConnectomeChangeResponse};
use feagi_npu_burst_core::errors::BurstEngineError;
use feagi_npu_burst_core::wrapped_values::EngineCorticalIndex;
use crate::rayon_data::RayonData;


pub struct RayonBurstEngine<FIQ: FeagiIndexQuantization> {
    data: RayonData<FIQ>,
    connectome_composer: ()
}

impl<FIQ: FeagiIndexQuantization> RayonBurstEngine<FIQ> {
    pub fn new() -> Self {
        Self {
            data: RayonData::new_blank(),
            connectome_composer: ()
        }
    }
}


impl<FIQ: FeagiIndexQuantization> BurstEngine<FIQ> for RayonBurstEngine<FIQ> {
    fn execute_phase(&mut self, phases: RunBurstPhase) -> impl Future<Output=Result<BurstPhaseOutput<FIQ>, BurstEngineError>> {
        core::future::ready(Ok(BurstPhaseOutput::NoFurtherActionNeeded))
    }
}

impl<FIQ: FeagiIndexQuantization> ComposableBurstEngine<FIQ> for RayonBurstEngine<FIQ>
{
    fn request_connectome_change(
        &mut self, request: EngineConnectomeChangeRequest<FIQ>
    ) -> impl Future<Output=Result<EngineConnectomeChangeResponse<FIQ>, BurstEngineError>> {

        // TODO
        core::future::ready(Ok(EngineConnectomeChangeResponse::CorticalAreaRemoved{}))
        
    }
}
