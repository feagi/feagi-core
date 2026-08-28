use crate::burst_engine_definitions::burst_phase_output::BurstPhaseOutput;
use crate::burst_engine_definitions::burst_phases::RunBurstPhase;
use crate::wrapped_values::EngineCorticalIndex;
use crate::errors::BurstEngineError;
use feagi_data::neurons::wrapped_types::CorticalNeuronLocalIndex;
use feagi_models::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use feagi_models::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;

// TODO Seperate Async and Nonasync?

/// Base trait for all burst engines. Bursts are executed by phases in an async manner
pub trait BurstEngine<NPUIQ: NeuronProcessingUnitIndexQuantization, BEIQ: BurstEngineIndexQuantization> {
    fn execute_phase(&mut self, phases: RunBurstPhase) -> impl core::future::Future<Output = Result<BurstPhaseOutput, BurstEngineError>>;
}

/// Any burst engine that supports connectome editing AKA composition
pub trait ComposableBurstEngine<NPUIQ: NeuronProcessingUnitIndexQuantization, BEIQ: BurstEngineIndexQuantization>: BurstEngine<NPUIQ, BEIQ> {

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
pub trait NonComposableBurstEngine<NPUIQ: NeuronProcessingUnitIndexQuantization, BEIQ: BurstEngineIndexQuantization>: BurstEngine<NPUIQ, BEIQ>
{

}