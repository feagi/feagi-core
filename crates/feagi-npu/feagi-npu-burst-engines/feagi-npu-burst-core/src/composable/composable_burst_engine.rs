use crate::burst_phases::RunBurstPhase;
use crate::composable::composable_burst_engine_allocator::ComposableBurstEngineAllocator;
use crate::composable::connectome_change_messaging::{EngineConnectomeChangeRequest, EngineConnectomeChangeResponse};
use crate::errors::BurstEngineError;
use feagi_data::quantization_levels::feagi_index_quantization::{FeagiIndexQuantization};
use feagi_models::wrapped_indexes::BurstIndex;
use crate::composable::composable_burst_phase_output::ComposableBurstPhaseOutput;

/// Defines a Burst Engine that can execute neuron dynamics, and also make changes to its running connectome
pub trait ComposableBurstEngine<FIQ: FeagiIndexQuantization>: Sized {
    /// The engine specific struct that processes engine specific instructions for making edits to the connectome
    type Allocator: ComposableBurstEngineAllocator<FIQ>;

    /// Execute some form of neural computation
    fn execute_phase(
        &mut self,
        phases: RunBurstPhase,
        burst_index: BurstIndex<FIQ::BurstIndexQuant>,
    ) -> impl core::future::Future<
        Output = Result<ComposableBurstPhaseOutput<FIQ>, BurstEngineError>,
    >;

    /// Send several changes to make in the connectome in order
    fn request_changes(
        &mut self,
        previous_burst_index: BurstIndex<FIQ::BurstIndexQuant>,
        engine_connectome_change_requests: Vec<EngineConnectomeChangeRequest<FIQ>>,
    ) -> impl core::future::Future<Output = Result<Vec<EngineConnectomeChangeResponse<FIQ>>, BurstEngineError>>;
}
