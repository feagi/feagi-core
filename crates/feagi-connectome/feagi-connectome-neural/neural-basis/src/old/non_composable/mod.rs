
pub use non_composable_engine_spawner::NonComposableBurstEngineSpawner;

/// Public for this crate but must not leave the NPU Crate!
pub mod npu_sealed {
    pub use super::non_composable_burst_engine::NonComposableBurstEngine;
    pub use super::non_composable_phase_notification::{NonComposablePhaseNotification};
    pub use super::non_composable_burst_phase_output::NonComposableBurstPhaseOutput;
    pub use super::non_composable_engine_spawner::npu_sealed::NonComposableBurstEngineSpawnerNPU;
}



mod non_composable_burst_engine;
mod non_composable_phase_notification;
mod non_composable_engine_spawner;
mod non_composable_burst_phase_output;