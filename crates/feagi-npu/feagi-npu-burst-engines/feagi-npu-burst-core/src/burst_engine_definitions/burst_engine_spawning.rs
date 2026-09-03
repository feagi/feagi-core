use crate::burst_engine_definitions::burst_engine::BurstEngine;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

// NOTE: This should be exported out of the NPU
/// Used to generate a specific burst engine with a specific backend with given parameters
pub trait BurstEngineSpawner<FIQ: FeagiIndexQuantization> {
    type StartingConnectomeData: EngineStartingConnectomeData<FIQ>;
}


/// Denotes any connectome data the burst engine may start with
pub trait EngineStartingConnectomeData<FIQ: FeagiIndexQuantization> {}

/// Default case of `EngineStartingConnectomeData`, where no data is included at all.
pub struct NoStartingConnectomeData;

impl<FIQ: FeagiIndexQuantization> EngineStartingConnectomeData<FIQ> for NoStartingConnectomeData {}