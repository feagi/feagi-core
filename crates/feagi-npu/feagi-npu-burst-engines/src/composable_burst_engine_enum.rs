use core::future::Future;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use feagi_npu_burst_core::burst_engine_definitions::burst_engine::{BurstEngine};
use feagi_npu_burst_core::burst_engine_definitions::burst_phase_output::BurstPhaseOutput;
use feagi_npu_burst_core::burst_engine_definitions::burst_phases::RunBurstPhase;
use feagi_npu_burst_core::errors::BurstEngineError;
use feagi_npu_burst_rayon::rayon_burst_engine::RayonBurstEngine;


/// Can represent multiple
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
    fn execute_phase(
        &mut self,
        phases: RunBurstPhase,
        burst_index: BurstIndex<FIQ::BurstIndexQuant>,
    ) -> impl Future<Output = Result<BurstPhaseOutput<FIQ>, BurstEngineError>> {
        match self {
            BurstEngineEnum::CPURayon(e) => e.execute_phase(phases, burst_index),
        }
    }
}