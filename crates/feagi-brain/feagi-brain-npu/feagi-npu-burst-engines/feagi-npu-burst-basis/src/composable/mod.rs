
pub use composable_engine_spawner::ComposableBurstEngineSpawner;

/// Public for this crate but must not leave the NPU Crate!
pub mod npu_sealed {
    pub use super::composable_burst_engine::ComposableBurstEngine;
    pub use super::composable_burst_engine_allocator::ComposableBurstEngineAllocator;
    pub use super::connectome_change_messaging::{EngineConnectomeChangeResponse, EngineConnectomeChangeRequest};
    pub use super::composable_phase_notification::ComposablePhaseNotification;
    pub use super::composable_engine_spawner::npu_sealed::ComposableBurstEngineSpawnerNPU;
    pub use super::composable_burst_phase_output::ComposableBurstPhaseOutput;
}

mod composable_burst_engine;
mod composable_burst_engine_allocator;
mod connectome_change_messaging;
mod composable_phase_notification;
mod composable_engine_spawner;
mod composable_burst_phase_output;