use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

// NOTE: This should be exported out of the NPU
/// Used to generate a specific burst engine with a specific backend with given parameters
pub trait ComposableBurstEngineSpawner<FIQ: FeagiIndexQuantization>: npu_sealed::ComposableBurstEngineSpawnerNPU<FIQ> {}

/// Should NOT be exported out the NPU (but should be out of this crate)
pub mod npu_sealed {
    use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
    use crate::composable::composable_burst_engine::ComposableBurstEngine;

    /// Actually contains the logic for generating the burst engine. Should not be exposed out of NPU
    pub trait ComposableBurstEngineSpawnerNPU<FIQ: FeagiIndexQuantization> {
        type BurstEngine: ComposableBurstEngine<FIQ>;
        // TODO Allocator?

        /// Tries to spawn the burst engine given the configuration
        fn spawn_burst_engine(self) -> impl core::future::Future<Output = Result<Self::BurstEngine, ()>>;
    }
}