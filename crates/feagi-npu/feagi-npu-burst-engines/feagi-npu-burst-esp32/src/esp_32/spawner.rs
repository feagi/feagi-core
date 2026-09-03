use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_npu_burst_core::burst_engine_definitions::burst_engine_spawning::BurstEngineSpawner;
use crate::starting_data::ESP32ConnectomeStartData;

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

impl<FIQ: FeagiIndexQuantization> BurstEngineSpawner<FIQ> for ESP32BoardESP32Spawner<FIQ> {
    type StartingConnectomeData = ESP32ConnectomeStartData;
}
