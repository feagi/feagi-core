use std::future::Future;
use feagi_data::neurons::wrapped_types::CorticalNeuronLocalIndex;
use feagi_models::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use feagi_models::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;
use feagi_npu_burst_core::burst_engine_definitions::burst_engine::{BurstEngine, ComposableBurstEngine};
use feagi_npu_burst_core::burst_engine_definitions::burst_phase_output::BurstPhaseOutput;
use feagi_npu_burst_core::burst_engine_definitions::burst_phases::RunBurstPhase;
use feagi_npu_burst_core::errors::BurstEngineError;
use feagi_npu_burst_core::wrapped_values::EngineCorticalIndex;
use crate::rayon_data::RayonData;


pub struct RayonBurstEngine<NPUIQ: NeuronProcessingUnitIndexQuantization, BEIQ: BurstEngineIndexQuantization> {
    data: RayonData<NPUIQ, BEIQ>,
}

impl<NPUIQ: NeuronProcessingUnitIndexQuantization, BEIQ: BurstEngineIndexQuantization> BurstEngine<NPUIQ, BEIQ> for RayonBurstEngine<NPUIQ, BEIQ> {
    fn execute_phase(&mut self, phases: RunBurstPhase) -> impl Future<Output=Result<BurstPhaseOutput, BurstEngineError>> {
        core::future::ready(Ok(BurstPhaseOutput::NoFurtherActionNeeded))
    }
}

impl<NPUIQ: NeuronProcessingUnitIndexQuantization, BEIQ: BurstEngineIndexQuantization> ComposableBurstEngine<NPUIQ, BEIQ> for RayonBurstEngine<NPUIQ, BEIQ>
{
    /*
    fn add_cortical_area<CA>(&mut self, cortical_area_writer: CA) -> impl Future<Output=Result<EngineCorticalIndex<BEIQ::CorticalAreaIndexCountQuant>, BurstEngineError>> {
        todo!()
    }

    fn remove_cortical_area<CA>(&mut self, cortical_area_index: EngineCorticalIndex<BEIQ::CorticalAreaIndexCountQuant>) -> impl Future<Output=Result<(), BurstEngineError>> {
        todo!()
    }

    fn inplace_edit_cortical_area<CA>(&mut self, cortical_area_index: EngineCorticalIndex<BEIQ::CorticalAreaIndexCountQuant>) -> impl Future<Output=Result<(), BurstEngineError>> {
        todo!()
    }

    fn add_cortical_mapping<CM>(&mut self, cortical_mapping_writer: CM) -> impl Future<Output=Result<(), BurstEngineError>> {
        todo!()
    }

    fn add_force_fires(&mut self, force_fires_to_add: &[CorticalNeuronLocalIndex<BEIQ::NeuronIndexQuant>]) -> impl Future<Output=Result<(), BurstEngineError>> {
        todo!()
    }
    
     */
}
