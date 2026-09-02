use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::burst_engine_definitions::burst_engine::BurstEngine;
use crate::burst_engine_definitions::composable::composable_engine_allocator::ComposableEngineAllocator;

/// Any burst engine that supports connectome editing AKA composition
pub trait ComposableBurstEngine<FIQ: FeagiIndexQuantization>: BurstEngine<FIQ> {
    /// The engine specific struct that processes engine specific instructions for making edits to the connectome
    type Allocator: ComposableEngineAllocator<FIQ>;
}