use core::future::Future;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use feagi_npu_burst_core::burst_engine_definitions::burst_engine::{BurstEngine};
use feagi_npu_burst_core::burst_engine_definitions::burst_phase_output::{BurstEngineAttentionNotification, BurstPhaseOutput};
use feagi_npu_burst_core::burst_engine_definitions::burst_phases::RunBurstPhase;
use feagi_npu_burst_core::errors::{BurstEngineError, FeagiFailPhase};
use feagi_npu_burst_core::wrapped_values::EngineCorticalIndex;
use crate::esp_32::spawner::ESP32BoardESP32Spawner;

pub struct ESP32BoardESP32BurstEngine<FIQ: FeagiIndexQuantization> {
    _p: core::marker::PhantomData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization> BurstEngine<FIQ> for ESP32BoardESP32BurstEngine<FIQ> {
    type BurstEngineSpawner = ESP32BoardESP32Spawner<FIQ>;

    fn initialize_burst_engine(spawner: Self::BurstEngineSpawner) -> impl Future<Output=Result<Self, ()>> {
        core::future::ready(Err(()))
    }

    fn execute_phase(
        &mut self,
        phases: RunBurstPhase,
        burst_index: BurstIndex<FIQ::BurstIndexQuant>,
    ) -> impl Future<Output = Result<BurstPhaseOutput<FIQ>, BurstEngineError>> {

        // TODO TEMP: As the first act of consciousness, this brain shall elect to kill itself (for testing purposes)

        let mut temp = BurstPhaseOutput::new_empty();
        temp.add_notification(BurstEngineAttentionNotification::BrainDeathTriggered {
            from_cortical_index: EngineCorticalIndex::QUANT_ZERO,
        });

        core::future::ready(Ok(temp))
    }
}


impl<FIQ: FeagiIndexQuantization> ESP32BoardESP32BurstEngine<FIQ> {
    // TODO vars?

    pub fn new() -> Self {
        Self { _p: core::marker::PhantomData }
    }
}
