use crate::burst_engine_definitions::burst_phase_output::BurstPhaseOutput;
use crate::burst_engine_definitions::burst_phases::RunBurstPhase;
use crate::errors::BurstEngineError;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use crate::burst_engine_definitions::burst_engine_spawning::BurstEngineSpawner;
#[cfg(feature = "alloc")]
use crate::burst_engine_definitions::composable_burst_engine_allocator::ComposableBurstEngineAllocator;
#[cfg(feature = "alloc")]
use crate::burst_engine_definitions::connectome_change_messaging::{EngineConnectomeChangeRequest, EngineConnectomeChangeResponse};

/// Base trait for all burst engines. Bursts are executed by phases in an async manner
pub trait BurstEngine<FIQ: FeagiIndexQuantization>: Sized {
    /// The struct that is publically used to define the parameters to spawn this burst engine
    type BurstEngineSpawner: BurstEngineSpawner<FIQ>;

    /// Initializes the burst engine, activating whatever hardware it is and prepares it for use
    fn initialize_burst_engine(spawner: Self::BurstEngineSpawner) -> impl core::future::Future<Output = Result<Self, ()>>; // TODO error

    /// execute some form of neural computation
    fn execute_phase(&mut self, phases: RunBurstPhase, burst_index: BurstIndex<FIQ::BurstIndexQuant>) -> impl core::future::Future<Output = Result<BurstPhaseOutput<FIQ>, BurstEngineError>>;
}

#[cfg(feature = "alloc")]
/// Any burst engine that supports connectome editing AKA composition
pub trait ComposableBurstEngine<FIQ: FeagiIndexQuantization>: BurstEngine<FIQ> {
    /// The engine specific struct that processes engine specific instructions for making edits to the connectome
    type Allocator: ComposableBurstEngineAllocator<FIQ>;

    /// Send several changes to make in order
    fn request_changes(&mut self, previous_burst_index: BurstIndex<FIQ::BurstIndexQuant>, engine_connectome_change_requests: Vec<EngineConnectomeChangeRequest<FIQ>>) -> impl core::future::Future<Output = Result<Vec<EngineConnectomeChangeResponse<FIQ>>, BurstEngineError>>;
}