use core::future::Future;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
#[cfg(feature = "engine-noncomposable-esp32-esp32")]
use feagi_npu_burst_core::burst_engine_definitions::burst_engine::BurstEngine;
use feagi_npu_burst_core::burst_engine_definitions::burst_phase_output::BurstPhaseOutput;
use feagi_npu_burst_core::burst_engine_definitions::burst_phases::RunBurstPhase;
use feagi_npu_burst_core::errors::BurstEngineError;
#[cfg(feature = "engine-noncomposable-esp32-esp32")]
use feagi_npu_burst_esp32::esp_32::burst_engine::ESP32BoardESP32BurstEngine;

pub struct NonComposableBurstEngine<FIQ: FeagiIndexQuantization> {
    engine: NonComposableBurstEngineEnum<FIQ>,
}

impl<FIQ: FeagiIndexQuantization> NonComposableBurstEngine<FIQ> {
    /// Executes a given phase
    pub fn execute_phase(
        &mut self,
        phases: RunBurstPhase,
        burst_index: BurstIndex<FIQ::BurstIndexQuant>,
    ) -> impl Future<Output = Result<BurstPhaseOutput<FIQ>, BurstEngineError>> + use<'_, FIQ> {
        self.engine.execute_phase(phases, burst_index)
    }
}


enum NonComposableBurstEngineEnum<FIQ: FeagiIndexQuantization> {
    #[cfg(feature = "engine-noncomposable-esp32-esp32")]
    ESP32BoardESP32(ESP32BoardESP32BurstEngine<FIQ>),
    // Keeps `FIQ` live when every engine variant is cfg'd out.
    #[cfg(not(feature = "engine-noncomposable-esp32-esp32"))]
    Impossible(core::marker::PhantomData<FIQ>), // Only here to get the compiler to shut up as it is blind to multi-feature setups
}

impl<FIQ: FeagiIndexQuantization> NonComposableBurstEngineEnum<FIQ> {
    #[inline(always)]
    pub(super) fn execute_phase(
        &mut self,
        phases: RunBurstPhase,
        burst_index: BurstIndex<FIQ::BurstIndexQuant>,
    ) -> impl Future<Output = Result<BurstPhaseOutput<FIQ>, BurstEngineError>> + use<'_, FIQ> {
        match self {
            #[cfg(feature = "engine-noncomposable-esp32-esp32")]
            NonComposableBurstEngineEnum::ESP32BoardESP32(e) => e.execute_phase(phases, burst_index),
            #[cfg(not(feature = "engine-noncomposable-esp32-esp32"))]
            NonComposableBurstEngineEnum::Impossible(_) => async move {
                unreachable!("no non-composable burst engine feature is enabled")
            },
        }
    }
}