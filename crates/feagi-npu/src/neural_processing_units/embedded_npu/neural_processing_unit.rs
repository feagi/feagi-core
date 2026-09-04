use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use feagi_npu_burst_engines::burst_phases::RunBurstPhase;
use feagi_npu_burst_engines::feagi_npu_burst_esp32::esp_32::ESP32BoardESP32Spawner;
use feagi_npu_burst_engines::npu_sealed::non_composable::{NonComposableBurstEngineSpawnerNPU, NonComposableBurstPhaseOutput};
use feagi_npu_burst_engines::npu_sealed::{EnclosedNonComposableBurstEngine, NonComposableEngineToEnclosedNonComposable};

// TODO spawner should be taking ina  trait (over a generic)

pub struct EmbeddedNPU<FIQ: FeagiIndexQuantization> {
    enclosed_engine: EnclosedNonComposableBurstEngine<FIQ>
}

impl<FIQ: FeagiIndexQuantization> EmbeddedNPU<FIQ> {

    pub async fn new(spawner: ESP32BoardESP32Spawner<FIQ>) -> Self {
        let engine = spawner.spawn_burst_engine().await.unwrap();
        let enclosed_engine = engine.to_enclosed_burst_engine();
        Self {
            enclosed_engine
        }
    }
    
    pub async fn test_burst(&mut self) -> NonComposableBurstPhaseOutput<FIQ> {
        let phase_result = self.enclosed_engine.execute_phase(
            RunBurstPhase::Full,
            BurstIndex::QUANT_ZERO
        ).await.unwrap();
        phase_result
    }



}