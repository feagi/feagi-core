use feagi_structures::feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantization;
use crate::neural_processing_unit_data_structures::burst_engine::burst_engine_just_completed_phase::BurstEngineJustCompletedPhase;
use crate::neural_processing_unit_data_structures::wrappers::NPUWrappedBurstEngineBurstIndex;

/// Represents the data contained of a burst engine. Typically this is not readily available, but
/// we will mandate some basic metadata be kept, as enforced by the shared methods
pub trait BurstEngineData<FGQ: FeagiGlobalQuantization> {
    /// Gets the current burst index of the engine
    fn get_current_burst_index(&self) -> NPUWrappedBurstEngineBurstIndex<FGQ::GlobalBurstIndexQuant>;
    
    /// Gets the current phase of the burst of the engine
    fn get_current_burst_phase(&self) -> BurstEngineJustCompletedPhase;
}

// TODO other metadata? number areas, neurons, synapses?