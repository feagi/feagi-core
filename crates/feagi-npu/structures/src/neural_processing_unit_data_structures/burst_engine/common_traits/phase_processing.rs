use feagi_structures::feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantization;
use crate::neural_processing_unit_data_structures::burst_engine::burst_engine_just_completed_phase::BurstEngineJustCompletedPhase;
use crate::neural_processing_unit_data_structures::burst_engine::common_traits::burst_engine_data::BurstEngineData;

/// Base trait for all Burst Engine Phase Processors, which is effectively a passable
/// function for manipulating NPU data to go from one phase to another
pub trait BurstEnginePhaseProcessor<FGQ>
where
    FGQ: FeagiGlobalQuantization,
{

}


pub trait BurstEnginePhaseBurstCounterIndexIncrement<FGQ>:
BurstEnginePhaseProcessor<FGQ>
where
    FGQ: FeagiGlobalQuantization,
{}


pub trait BurstEnginePhaseNeuronModelUpdatedForBurstIndexRollover<FGQ>:
BurstEnginePhaseProcessor<FGQ>
where
    FGQ: FeagiGlobalQuantization,
{}


pub trait BurstEnginePhaseSynapseModelUpdatedForBurstIndexRollover<FGQ>:
BurstEnginePhaseProcessor<FGQ>
where
    FGQ: FeagiGlobalQuantization,
{}


pub trait BurstEnginePhaseFCLConsolidation<FGQ>:
BurstEnginePhaseProcessor<FGQ>
where
    FGQ: FeagiGlobalQuantization,
{}


pub trait BurstEnginePhaseNeuronDynamics<FGQ>:
BurstEnginePhaseProcessor<FGQ>
where
    FGQ: FeagiGlobalQuantization,
{}


pub trait BurstEnginePhaseUpdateFiringNeuronBitfield<FGQ>:
BurstEnginePhaseProcessor<FGQ>
where
    FGQ: FeagiGlobalQuantization,
{}


pub trait BurstEnginePhaseCountFiringNeuronsPerCorticalArea<FGQ>:
BurstEnginePhaseProcessor<FGQ>
where
    FGQ: FeagiGlobalQuantization,
{}


pub trait BurstEnginePhasePreSynapseDataExchange<FGQ>:
BurstEnginePhaseProcessor<FGQ>
where
    FGQ: FeagiGlobalQuantization,
{}


pub trait BurstEnginePhaseFiringNeuronConsolidation<FGQ>:
BurstEnginePhaseProcessor<FGQ>
where
    FGQ: FeagiGlobalQuantization,
{}


pub trait BurstEnginePhaseSynapseDynamics<FGQ>:
BurstEnginePhaseProcessor<FGQ>
where
    FGQ: FeagiGlobalQuantization,
{}


pub trait BurstEnginePhasePostSynapseDataExchange<FGQ>:
BurstEnginePhaseProcessor<FGQ>
where
    FGQ: FeagiGlobalQuantization,
{}


pub trait BurstEnginePhaseFCLCConsolidation<FGQ>:
BurstEnginePhaseProcessor<FGQ>
where
    FGQ: FeagiGlobalQuantization,
{}