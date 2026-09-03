use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_npu_burst_core::burst_engine_definitions::burst_engine_spawning::EngineStartingConnectomeData;

/// Denotes the data the engine starts with (IE the connectome)
pub struct ESP32ConnectomeStartData; // TODO define, expand

impl<FIQ: FeagiIndexQuantization> EngineStartingConnectomeData<FIQ> for ESP32ConnectomeStartData {}


