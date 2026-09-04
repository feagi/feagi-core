use std::future::Future;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use feagi_npu_burst_core::burst_phases::RunBurstPhase;
use feagi_npu_burst_core::errors::BurstEngineError;
use feagi_npu_burst_core::composable::npu_sealed::ComposableBurstPhaseOutput;
use crate::burst_engine_package::EnclosedEngine;

pub enum EnclosedComposableBurstEngine<FIQ: FeagiIndexQuantization> {
    #[cfg(feature = "engines-rayon")]
    CPURayon(RayonBurstEngine<FIQ>),
    /// Only here to get the compiler to shut up as it is blind to multi-feature setups
    Impossible(core::marker::PhantomData<FIQ>),
}

impl<FIQ: FeagiIndexQuantization> EnclosedComposableBurstEngine<FIQ> {
    #[inline(always)]
    pub fn execute_phase(
        &mut self,
        phases: RunBurstPhase,
        burst_index: BurstIndex<FIQ::BurstIndexQuant>,
    ) -> impl Future<Output = Result<ComposableBurstPhaseOutput<FIQ>, BurstEngineError>> + use<'_, FIQ> {
        match self {
            #[cfg(feature = "engines-rayon")]
            EnclosedComposableBurstEngine::CPURayon(e) => e.execute_phase(phases, burst_index),
            _ => core::future::ready(unreachable!("no composable burst engine feature is enabled")),
        }
    }

    // TODO Composition functions
}

impl<FIQ: FeagiIndexQuantization> EnclosedEngine<FIQ> for EnclosedComposableBurstEngine<FIQ> {}

/// Implemented on the burst engines to easily make them a `EnclosedComposableBurstEngine`
pub trait SpawnerToEnclosedComposable<FIQ: FeagiIndexQuantization> {
    fn to_enclosed_burst_engine(self) -> EnclosedComposableBurstEngine<FIQ>;
}

#[cfg(feature = "engines-rayon")]
impl<FIQ: FeagiIndexQuantization> SpawnerToEnclosedComposable<FIQ> for RayonBurstEngine<FIQ> {
    fn to_enclosed_burst_engine(self) -> EnclosedComposableBurstEngine<FIQ> {
        EnclosedComposableBurstEngine::CPURayon(self)
    }
}