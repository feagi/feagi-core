use crate::burst_engine_definitions::burst_phase_output::BurstPhaseOutput;
use crate::burst_engine_definitions::burst_phases::RunBurstPhase;
use crate::errors::BurstEngineError;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
#[cfg(feature = "composable")]
use crate::burst_engine_definitions::composable_engine_allocator::ComposableEngineAllocator;
#[cfg(feature = "composable")]
use crate::burst_engine_definitions::connectome_change_messaging::{EngineConnectomeChangeRequest, EngineConnectomeChangeResponse};

/// Base trait for all burst engines. Bursts are executed by phases in an async manner
pub trait BurstEngine<FIQ: FeagiIndexQuantization> {
    fn execute_phase(&mut self, phases: RunBurstPhase, burst_index: BurstIndex<FIQ::BurstIndexQuant>) -> impl core::future::Future<Output = Result<BurstPhaseOutput<FIQ>, BurstEngineError>>;
}

#[cfg(feature = "composable")]
/// Any burst engine that supports connectome editing AKA composition
pub trait ComposableBurstEngine<FIQ: FeagiIndexQuantization>: BurstEngine<FIQ> {
    /// The engine specific struct that processes engine specific instructions for making edits to the connectome
    type Allocator: ComposableEngineAllocator<FIQ>;
}

/// A marker trait to denote a burst engine as not being editable
pub trait NonComposableBurstEngine<FIQ: FeagiIndexQuantization>: BurstEngine<FIQ> {}