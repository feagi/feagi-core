use feagi_data::neurons::wrapped_types::CorticalNeuronLocalIndex;
use feagi_models::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use feagi_models::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;
use feagi_npu_burst_core::burst_engine_definitions::burst_engine::{BurstEngine, ComposableBurstEngine};
use feagi_npu_burst_core::burst_engine_definitions::burst_phase_output::BurstPhaseOutput;
use feagi_npu_burst_core::burst_engine_definitions::burst_phases::RunBurstPhase;
use feagi_npu_burst_core::wrapped_values::EngineCorticalIndex;
use feagi_npu_burst_core::errors::BurstEngineError;
use core::future::Future;
use feagi_npu_burst_rayon::rayon_burst_engine::RayonBurstEngine;

pub enum ComposableBurstEngineEnum<NPUIQ, BEIQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
{
    CPURayon(RayonBurstEngine<NPUIQ, BEIQ>),
}

impl<NPUIQ, BEIQ> BurstEngine<NPUIQ, BEIQ> for ComposableBurstEngineEnum<NPUIQ, BEIQ>
where
    BEIQ: BurstEngineIndexQuantization,
    NPUIQ: NeuronProcessingUnitIndexQuantization,
{
    fn execute_phase(&mut self, phases: RunBurstPhase) -> impl Future<Output = Result<BurstPhaseOutput, BurstEngineError>> {
        match self {
            ComposableBurstEngineEnum::CPURayon(e) => {e.execute_phase(phases)}
        }
    }
}

impl<NPUIQ, BEIQ> ComposableBurstEngine<NPUIQ, BEIQ> for ComposableBurstEngineEnum<NPUIQ, BEIQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
{
    /*
    fn add_cortical_area<CA>(
        &mut self,
        cortical_area_writer: CA,
    ) -> impl Future<Output = Result<EngineCorticalIndex<BEIQ::CorticalAreaIndexCountQuant>, BurstEngineError>> {
        todo!()
    }

    fn remove_cortical_area<CA>(
        &mut self,
        cortical_area_index: EngineCorticalIndex<BEIQ::CorticalAreaIndexCountQuant>,
    ) -> impl Future<Output = Result<(), BurstEngineError>> {
        todo!()
    }

    fn inplace_edit_cortical_area<CA>(
        &mut self,
        cortical_area_index: EngineCorticalIndex<BEIQ::CorticalAreaIndexCountQuant>,
    ) -> impl Future<Output = Result<(), BurstEngineError>> {
        todo!()
    }

    fn add_cortical_mapping<CM>(&mut self, cortical_mapping_writer: CM) -> impl Future<Output = Result<(), BurstEngineError>> {
        todo!()
    }

    fn add_force_fires(
        &mut self,
        force_fires_to_add: &[CorticalNeuronLocalIndex<BEIQ::NeuronIndexQuant>],
    ) -> impl Future<Output = Result<(), BurstEngineError>> {
        todo!()
    }
    
     */
}
