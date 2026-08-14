use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::npu::burst_engine::composable_implementations::tokio_rayon::tokio_rayon_burst_engine::TokioRayonBurstEngine;
use crate::npu::burst_engine::{BurstEngine, ComposableBurstEngine};

/// All possible composable burst engines in a fast access enum, using the same common trait access
pub enum ComposableBurstEngineEnum<FIQ: FeagiIndexQuantization> {
    TokioRayonBurstEngine(TokioRayonBurstEngine<FIQ>)
}

impl<FIQ: FeagiIndexQuantization> BurstEngine<FIQ> for ComposableBurstEngineEnum<FIQ> {
    async fn run_kernel(&mut self) {
        todo!()
    }
}

impl<FIQ: FeagiIndexQuantization> ComposableBurstEngine<FIQ> for ComposableBurstEngineEnum<FIQ> {
    async fn edit_connectome(&mut self) {
        match self {
            ComposableBurstEngineEnum::TokioRayonBurstEngine(engine) => {
                engine.edit_connectome().await
            }
        }
    }
}

