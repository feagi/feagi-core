use feagi_data::neurons::wrapped_types::CorticalNeuronLocalIndex;
use feagi_models::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use feagi_models::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;
use crate::burst_engine_definitions::burst_phase_output::BurstPhaseOutput;
use crate::burst_engine_definitions::burst_phases::RunBurstPhase;
use crate::burst_engine_definitions::wrapped_values::EngineCorticalIndex;
use crate::errors::BurstEngineError;

pub trait BurstEngine<NPUIQ: NeuronProcessingUnitIndexQuantization, BEIQ: BurstEngineIndexQuantization>
{
    async fn execute_phase(&mut self, phases: RunBurstPhase) -> Result<BurstPhaseOutput, BurstEngineError>;
}

pub trait ComposableBurstEngine<NPUIQ: NeuronProcessingUnitIndexQuantization, BEIQ: BurstEngineIndexQuantization>: BurstEngine<NPUIQ, BEIQ>
{
    async fn add_cortical_area<CA>(&mut self, cortical_area_writer: CA) -> Result<EngineCorticalIndex<BEIQ::CorticalAreaIndexCountQuant>, BurstEngineError>;
    
    async fn remove_cortical_area<CA>(&mut self, cortical_area_index: EngineCorticalIndex<BEIQ::CorticalAreaIndexCountQuant>) -> Result<(), BurstEngineError>;

    async fn inplace_edit_cortical_area<CA>(&mut self, cortical_area_index: EngineCorticalIndex<BEIQ::CorticalAreaIndexCountQuant>) -> Result<(), BurstEngineError>;
    
    async fn add_cortical_mapping<CM>(&mut self, cortical_mapping_writer: CM) -> Result<(), BurstEngineError>;
    
    async fn add_force_fires(&mut self, force_fires_to_add: &[CorticalNeuronLocalIndex<BEIQ::NeuronIndexQuant>]) -> Result<(), BurstEngineError>;
    
    // todo remove force fires
    
    // TODO reimport the rest later
    
}