use feagi_data::neurons::wrapped_types::CorticalNeuronLocalIndex;
use feagi_models::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use feagi_models::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;
use crate::burst_engine_definitions::burst_engine::{BurstEngine, ComposableBurstEngine};
use crate::burst_engine_definitions::burst_phase_output::BurstPhaseOutput;
use crate::burst_engine_definitions::burst_phases::RunBurstPhase;
use crate::burst_engine_definitions::wrapped_values::EngineCorticalIndex;
use crate::errors::BurstEngineError;

pub enum ComposableBurstEngineEnum<NPUIQ, BEIQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
{
    CPURayon((NPUIQ, BEIQ)),
}

impl<NPUIQ, BEIQ> BurstEngine<NPUIQ, BEIQ> for ComposableBurstEngineEnum<NPUIQ, BEIQ>
where
    BEIQ: BurstEngineIndexQuantization,
    NPUIQ: NeuronProcessingUnitIndexQuantization,
{
    async fn execute_phase(&mut self, phases: RunBurstPhase) -> Result<BurstPhaseOutput, BurstEngineError> {
        todo!()
    }
}

impl<NPUIQ, BEIQ> ComposableBurstEngine<NPUIQ, BEIQ> for ComposableBurstEngineEnum<NPUIQ, BEIQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
{
    async fn add_cortical_area<CA>(&mut self, cortical_area_writer: CA) -> Result<EngineCorticalIndex<BEIQ::CorticalAreaIndexCountQuant>, BurstEngineError> {
        todo!()
    }

    async fn remove_cortical_area<CA>(&mut self, cortical_area_index: EngineCorticalIndex<BEIQ::CorticalAreaIndexCountQuant>) -> Result<(), BurstEngineError> {
        todo!()
    }

    async fn inplace_edit_cortical_area<CA>(&mut self, cortical_area_index: EngineCorticalIndex<BEIQ::CorticalAreaIndexCountQuant>) -> Result<(), BurstEngineError> {
        todo!()
    }

    async fn add_cortical_mapping<CM>(&mut self, cortical_mapping_writer: CM) -> Result<(), BurstEngineError> {
        todo!()
    }

    async fn add_force_fires(&mut self, force_fires_to_add: &[CorticalNeuronLocalIndex<BEIQ::NeuronIndexQuant>]) -> Result<(), BurstEngineError> {
        todo!()
    }
}