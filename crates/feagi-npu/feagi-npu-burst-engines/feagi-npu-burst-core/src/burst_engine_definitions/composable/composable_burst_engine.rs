use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use crate::burst_engine_definitions::burst_engine::BurstEngine;
use crate::burst_engine_definitions::composable::composable_engine_allocator::ComposableEngineAllocator;
use crate::burst_engine_definitions::composable::connectome_change_messaging::{EngineConnectomeChangeRequest, EngineConnectomeChangeResponse};
use crate::errors::BurstEngineError;

/// Any burst engine that supports connectome editing AKA composition
pub trait ComposableBurstEngine<FIQ: FeagiIndexQuantization>: BurstEngine<FIQ> {
    /// The engine specific struct that processes engine specific instructions for making edits to the connectome
    type Allocator: ComposableEngineAllocator<FIQ>;

    /// Send several changes to make in order
    fn request_changes(&mut self, previous_burst_index: BurstIndex<FIQ::BurstIndexQuant>, engine_connectome_change_requests: Vec<EngineConnectomeChangeRequest<FIQ>>) -> impl core::future::Future<Output = Result<Vec<EngineConnectomeChangeResponse<FIQ>>, BurstEngineError>>;
}