use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

use feagi_npu_burst_engines::feagi_npu_burst_esp32::esp_32::burst_engine::ESP32BoardESP32BurstEngine;


pub struct EmbeddedNPU<FIQ: FeagiIndexQuantization> {
    temp: ESP32BoardESP32BurstEngine<FIQ>
}

impl<FIQ: FeagiIndexQuantization> EmbeddedNPU<FIQ> {

    pub fn new() -> Self {
        Self {temp: ESP32BoardESP32BurstEngine::new()}
    }

    

}