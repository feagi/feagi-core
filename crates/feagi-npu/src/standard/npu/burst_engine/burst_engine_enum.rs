use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::standard::npu::burst_engine::burst_engine::SyncBurstEngine;
use crate::standard::npu::burst_engine::burst_engine_communication::{EngineConnectomeEditRequest, EngineConnectomeEditResponse, KernelCommand};
use crate::standard::npu::burst_engine::burst_engine_error::FeagiBurstEngineError;

/// Wraps all possible burst engines into a single enum for rapid lookup.
///
/// Construction: use a per-variant constructor (or a `BurstEngineSpec`-style factory) rather
/// than `Default`; wgpu-style backends need a device/adapter/queue at construction and
/// cannot be produced from nothing.
pub enum BurstEngineEnum<FIQ: FeagiIndexQuantization> {
    // TODO variant for Rayon, WGPU, etc
    _Placeholder(core::marker::PhantomData<FIQ>),
}

impl<FIQ: FeagiIndexQuantization + std::marker::Send> SyncBurstEngine<FIQ> for BurstEngineEnum<FIQ> {
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

