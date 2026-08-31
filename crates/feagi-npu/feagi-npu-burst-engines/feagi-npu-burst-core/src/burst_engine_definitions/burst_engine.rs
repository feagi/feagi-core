use crate::burst_engine_definitions::burst_phase_output::BurstPhaseOutput;
use crate::burst_engine_definitions::burst_phases::RunBurstPhase;
use crate::errors::BurstEngineError;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use crate::burst_engine_definitions::connectome_change_messaging::{EngineConnectomeChangeRequest, EngineConnectomeChangeResponse};
// TODO Seperate Async and Nonasync?

/// Base trait for all burst engines. Bursts are executed by phases in an async manner
pub trait BurstEngine<FIQ: FeagiIndexQuantization> {
    fn execute_phase(&mut self, phases: RunBurstPhase, burst_index: BurstIndex<FIQ::BurstIndexQuant>) -> impl core::future::Future<Output = Result<BurstPhaseOutput<FIQ>, BurstEngineError>>;
}

/// Any burst engine that supports connectome editing AKA composition
pub trait ComposableBurstEngine<FIQ: FeagiIndexQuantization>: BurstEngine<FIQ> {

    
    fn request_connectome_change(
        &mut self, request: EngineConnectomeChangeRequest<FIQ>
    ) -> impl core::future::Future<Output = Result<EngineConnectomeChangeResponse<FIQ>, BurstEngineError>>;

    
    /*
    fn add_cortical_area<CA>(
        &mut self,
        cortical_area_writer: CA,
    ) -> impl core::future::Future<Output = Result<EngineCorticalIndex<BEIQ::CorticalAreaIndexCountQuant>, BurstEngineError>>;

    fn remove_cortical_area<CA>(
        &mut self,
        cortical_area_index: EngineCorticalIndex<BEIQ::CorticalAreaIndexCountQuant>,
    ) -> impl core::future::Future<Output = Result<(), BurstEngineError>>;

    fn inplace_edit_cortical_area<CA>(
        &mut self,
        cortical_area_index: EngineCorticalIndex<BEIQ::CorticalAreaIndexCountQuant>,
    ) -> impl core::future::Future<Output = Result<(), BurstEngineError>>;

    fn add_cortical_mapping<CM>(&mut self, cortical_mapping_writer: CM) -> impl core::future::Future<Output = Result<(), BurstEngineError>>;

    fn add_force_fires(
        &mut self,
        force_fires_to_add: &[CorticalNeuronLocalIndex<BEIQ::NeuronIndexQuant>],
    ) -> impl core::future::Future<Output = Result<(), BurstEngineError>>;

    // todo remove force fires

    // TODO reimport the rest later

     */
}

/// A marker trait to denote a burst engine as not being editable
pub trait NonComposableBurstEngine<FIQ: FeagiIndexQuantization>: BurstEngine<FIQ>
{

}