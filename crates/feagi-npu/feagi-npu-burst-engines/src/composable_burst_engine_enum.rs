use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_npu_burst_core::burst_engine_definitions::burst_engine::{BurstEngine, ComposableBurstEngine};
use feagi_npu_burst_core::burst_engine_definitions::burst_phase_output::BurstPhaseOutput;
use feagi_npu_burst_core::burst_engine_definitions::burst_phases::RunBurstPhase;
use feagi_npu_burst_core::errors::BurstEngineError;
use core::future::Future;
use feagi_models::wrapped_indexes::BurstIndex;
use feagi_npu_burst_core::burst_engine_definitions::connectome_change_messaging::{EngineConnectomeChangeRequest, EngineConnectomeChangeResponse};
use feagi_npu_burst_rayon::rayon_burst_engine::RayonBurstEngine;

// TODO feature gate composable engines // noncposable engines

pub enum BurstEngineEnum<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    CPURayon(RayonBurstEngine<FIQ>),
}

impl<FIQ> BurstEngineEnum<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    pub fn new_cpu_rayon() -> Self {
        BurstEngineEnum::CPURayon(RayonBurstEngine::new())
    }
}

impl<FIQ> BurstEngine<FIQ> for BurstEngineEnum<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn execute_phase(&mut self, phases: RunBurstPhase, burst_index: BurstIndex<FIQ::BurstIndexQuant>) -> impl Future<Output = Result<BurstPhaseOutput<FIQ>, BurstEngineError>> {
        match self {
            BurstEngineEnum::CPURayon(e) => {e.execute_phase(phases)}
        }
    }
}

impl<FIQ> ComposableBurstEngine<FIQ> for BurstEngineEnum<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn request_connectome_change(&mut self, request: EngineConnectomeChangeRequest<FIQ>) -> impl Future<Output=Result<EngineConnectomeChangeResponse<FIQ>, BurstEngineError>> {
        match self {
            BurstEngineEnum::CPURayon(r) => {r.request_connectome_change(request)}
        }
    }

    

}
