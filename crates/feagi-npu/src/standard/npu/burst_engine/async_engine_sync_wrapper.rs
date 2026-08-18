use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::standard::npu::burst_engine::burst_engine::{AsyncBurstEngine, SyncBurstEngine};
use crate::standard::npu::burst_engine::burst_engine_communication::{EngineConnectomeEditRequest, EngineConnectomeEditResponse, KernelCommand};
use crate::standard::npu::burst_engine::burst_engine_error::FeagiBurstEngineError;

/// Allows use of async burst engines in sync contexts in a performant manner.
///
/// Owns a per-thread tokio runtime (built by the caller) that drives `AE`'s async methods
/// via `block_on`. Because the runtime is entered per call rather than once at startup,
/// this only supports engines whose ticks are atomic from the pool's perspective.
pub struct SyncWrappedAsyncBurstEngine<FIQ: FeagiIndexQuantization, AE: AsyncBurstEngine<FIQ>>
{
    // TODO tokio::runtime::Runtime (current_thread) built at construction time
    async_engine: AE,
    _p: core::marker::PhantomData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization + std::marker::Send, AE: AsyncBurstEngine<FIQ>> SyncBurstEngine<FIQ> for SyncWrappedAsyncBurstEngine<FIQ, AE>
{
    fn run_kernel(&mut self, kernel: KernelCommand) -> Result<(), FeagiBurstEngineError> {
        todo!()
    }

    fn inject_membrane_potentials(&mut self, injections: ()) -> Result<(), FeagiBurstEngineError> {
        todo!()
    }

    fn inject_force_firings(&mut self, injections: ()) -> Result<(), FeagiBurstEngineError> {
        todo!()
    }

    fn extract_membrane_potentials(&mut self, extractions: ()) -> Result<(), FeagiBurstEngineError> {
        todo!()
    }

    fn extract_visualizations(&mut self, visualizations: ()) -> Result<(), FeagiBurstEngineError> {
        todo!()
    }

    fn edit_agent_registrations(&mut self, agent_registration_context: ()) -> Result<(), FeagiBurstEngineError> {
        todo!()
    }

    fn edit_connectome(&mut self, burst_engine_connectome_request: EngineConnectomeEditRequest<FIQ>) -> Result<EngineConnectomeEditResponse<FIQ>, FeagiBurstEngineError> {
        todo!()
    }
}


