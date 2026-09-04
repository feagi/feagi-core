use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

// NOTE: This should be exported out of the NPU
/// Used to generate a specific burst engine with a specific backend with given parameters
pub trait NonComposableBurstEngineSpawner<FIQ: FeagiIndexQuantization>: npu_sealed::NonComposableBurstEngineSpawnerNPU<FIQ> {}

/// Should NOT be exported out the NPU (but should be out of this crate)
pub mod npu_sealed {
    use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
    use crate::non_composable::non_composable_burst_engine::NonComposableBurstEngine;

    /// Actually contains the logic for generating the burst engine. Should not be exposed out of NPU
    pub trait NonComposableBurstEngineSpawnerNPU<FIQ: FeagiIndexQuantization> {
        type BurstEngine: NonComposableBurstEngine<FIQ>;

        /// Tries to spawn the burst engine given the configuration
        fn spawn_burst_engine(self) -> impl core::future::Future<Output = Result<Self::BurstEngine, ()>>;
    }
}