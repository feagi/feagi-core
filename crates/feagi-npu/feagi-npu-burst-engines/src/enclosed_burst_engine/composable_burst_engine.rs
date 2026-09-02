//! This whole module is feature gated by composable. This is an interface for composable burst engines

use core::future::Future;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use feagi_npu_burst_core::burst_engine_definitions::burst_phase_output::BurstPhaseOutput;
use feagi_npu_burst_core::burst_engine_definitions::burst_phases::RunBurstPhase;
use feagi_npu_burst_core::errors::BurstEngineError;

#[cfg(feature = "composable")]
pub struct ComposableBurstEngine<FIQ: FeagiIndexQuantization> {
    engine: ComposableBurstEngineEnum<FIQ>,
}

#[cfg(feature = "composable")]
impl<FIQ: FeagiIndexQuantization> ComposableBurstEngine<FIQ> {

    pub fn execute_phase(
        &mut self,
        phases: RunBurstPhase,
        burst_index: BurstIndex<FIQ::BurstIndexQuant>,
    ) -> impl Future<Output = Result<BurstPhaseOutput<FIQ>, BurstEngineError>> {
        self.engine.execute_phase(phases, burst_index)
    }

    pub fn request_connectome_change(&mut self) {
        todo!()
    }
}

#[cfg(feature = "composable")]
enum ComposableBurstEngineEnum<FIQ: FeagiIndexQuantization> {
    #[cfg(feature = "engine-composable-rayon")]
    CPURayon(RayonBurstEngine<FIQ>),
    // Keeps `FIQ` live when every engine variant is cfg'd out.
    #[cfg(not(feature = "engine-composable-rayon"))]
    Impossible(core::marker::PhantomData<FIQ>), // Only here to get the compiler to shut up as it is blind to multi-feature setups
}

#[cfg(feature = "composable")]
impl<FIQ: FeagiIndexQuantization> ComposableBurstEngineEnum<FIQ> {
    #[inline(always)]
    pub(super) fn execute_phase(
        &mut self,
        phases: RunBurstPhase,
        burst_index: BurstIndex<FIQ::BurstIndexQuant>,
    ) -> impl Future<Output = Result<BurstPhaseOutput<FIQ>, BurstEngineError>> {
        match self {
            #[cfg(feature = "engine-composable-rayon")]
            ComposableBurstEngineEnum::CPURayon(e) => e.execute_phase(phases, burst_index),
            #[cfg(not(feature = "engine-composable-rayon"))]
            ComposableBurstEngineEnum::Impossible(_) => async move {
                unreachable!("no composable burst engine feature is enabled")
            },
        }
    }
}