use core::future::Future;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use feagi_npu_burst_core::burst_phases::RunBurstPhase;
use feagi_npu_burst_core::errors::BurstEngineError;
use feagi_npu_burst_core::npu_sealed::non_composable::{NonComposableBurstEngine, NonComposableBurstPhaseOutput};
#[cfg(feature = "feagi-npu-burst-esp32")]
use feagi_npu_burst_esp32::esp_32::npu_sealed::ESP32BoardESP32BurstEngine;
use crate::burst_engine_package::EnclosedEngine;

pub enum EnclosedNonComposableBurstEngine<FIQ: FeagiIndexQuantization> {
    #[cfg(feature = "engines-esp32")]
    ESP32BoardESP32(ESP32BoardESP32BurstEngine<FIQ>),

    /// Only here to get the compiler to shut up as it is blind to multi-feature setups
    #[cfg(not(any(feature = "engines-esp32")))]
    Impossible(core::marker::PhantomData<FIQ>),
}

impl<FIQ: FeagiIndexQuantization> EnclosedNonComposableBurstEngine<FIQ> {
    #[inline(always)]
    pub fn execute_phase(
        &mut self,
        phases: RunBurstPhase,
        burst_index: BurstIndex<FIQ::BurstIndexQuant>,
    ) -> impl Future<Output = Result<NonComposableBurstPhaseOutput<FIQ>, BurstEngineError>> + use<'_, FIQ> {
        match self {
            #[cfg(feature = "engines-esp32")]
            EnclosedNonComposableBurstEngine::ESP32BoardESP32(e) => e.execute_phase(phases, burst_index),
            #[cfg(not(any(feature = "engines-esp32")))]
            EnclosedNonComposableBurstEngine::Impossible(_) => {
                async move {
                    unreachable!("Invalid Burst engine for execution")
                }
            }
        }
    }
}

impl<FIQ: FeagiIndexQuantization> EnclosedEngine<FIQ> for EnclosedNonComposableBurstEngine<FIQ> {}

/// Implemented on the burst engines to easily make them a `EnclosedNonComposableBurstEngine`
pub trait SpawnerToEnclosedNonComposable<FIQ: FeagiIndexQuantization> {
    fn to_enclosed_burst_engine(self) -> EnclosedNonComposableBurstEngine<FIQ>;
}

#[cfg(feature = "engines-esp32")]
impl<FIQ: FeagiIndexQuantization> SpawnerToEnclosedNonComposable<FIQ> for ESP32BoardESP32BurstEngine<FIQ> {
    fn to_enclosed_burst_engine(self) -> EnclosedNonComposableBurstEngine<FIQ> {
        EnclosedNonComposableBurstEngine::ESP32BoardESP32(self)
    }
}