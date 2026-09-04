use core::future::Future;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use feagi_npu_burst_core::burst_phases::RunBurstPhase;
use feagi_npu_burst_core::errors::BurstEngineError;
use feagi_npu_burst_core::non_composable::NonComposableBurstEngineSpawner;
use feagi_npu_burst_core::non_composable::npu_sealed::{NonComposableBurstEngine, NonComposableBurstEngineSpawnerNPU, NonComposableBurstPhaseOutput, NonComposablePhaseNotification};
use feagi_npu_burst_core::wrapped_values::EngineCorticalIndex;

pub struct ESP32BoardESP32BurstEngine<FIQ: FeagiIndexQuantization> {
    _p: core::marker::PhantomData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization> NonComposableBurstEngine<FIQ> for ESP32BoardESP32BurstEngine<FIQ> {
    fn execute_phase(
        &mut self,
        phases: RunBurstPhase,
        burst_index: BurstIndex<FIQ::BurstIndexQuant>,
    ) -> impl Future<Output = Result<NonComposableBurstPhaseOutput<FIQ>, BurstEngineError>> {
        
        // TODO TEMP: As the first act of consciousness, this brain shall elect to kill itself (for testing purposes)

        let mut temp = NonComposableBurstPhaseOutput::new_empty();
        temp.add_notification(
            NonComposablePhaseNotification::BrainDeathTriggered { 
                from_cortical_index: EngineCorticalIndex::QUANT_ZERO
            }
        ).unwrap();
        core::future::ready(Ok(temp))
    }
}


pub struct ESP32BoardESP32Spawner<FIQ: FeagiIndexQuantization> {
    _p: core::marker::PhantomData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization> ESP32BoardESP32Spawner<FIQ> {
    pub fn new() -> Self {
        Self {
            _p: core::marker::PhantomData
        }
    }
}

impl<FIQ: FeagiIndexQuantization> NonComposableBurstEngineSpawner<FIQ> for ESP32BoardESP32Spawner<FIQ> {}


impl<FIQ: FeagiIndexQuantization> NonComposableBurstEngineSpawnerNPU<FIQ> for ESP32BoardESP32Spawner<FIQ> {
    type BurstEngine = ESP32BoardESP32BurstEngine<FIQ>;

    fn spawn_burst_engine(self) -> impl Future<Output=Result<Self::BurstEngine, ()>> {
        let engine = ESP32BoardESP32BurstEngine {
            _p: core::marker::PhantomData
        };
        
        core::future::ready(Ok(engine))
    }
}
