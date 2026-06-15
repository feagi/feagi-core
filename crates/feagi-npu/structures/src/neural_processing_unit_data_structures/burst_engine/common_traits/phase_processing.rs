use feagi_structures::feagi_data::quantization_levels::feagi_global_quantization::FeagiGlobalQuantization;
use crate::neural_processing_unit_data_structures::burst_engine::burst_engine_just_completed_phase::BurstEngineJustCompletedPhase;
use crate::neural_processing_unit_data_structures::burst_engine::common_traits::burst_engine_data::BurstEngineData;

/// Base trait for all Burst Engine Phase Processors, which is effectively a passable
/// function for manipulating NPU data to go from one phase to another
pub trait BurstEnginePhaseProcessor<FGQ, BED>
where
    FGQ: FeagiGlobalQuantization,
    BED: BurstEngineData<FGQ>,
{
    fn process_phase(current_phase: &BurstEngineJustCompletedPhase, data: &mut BED)
        -> BurstEngineJustCompletedPhase;
}


pub trait BurstEnginePhaseBurstCounterIndexIncrement<FGQ, BED>:
BurstEnginePhaseProcessor<FGQ, BED>
where
    FGQ: FeagiGlobalQuantization,
    BED: BurstEngineData<FGQ>,
{}


pub trait BurstEnginePhaseNeuronModelUpdatedForBurstIndexRollover<FGQ, BED>:
BurstEnginePhaseProcessor<FGQ, BED>
where
    FGQ: FeagiGlobalQuantization,
    BED: BurstEngineData<FGQ>,
{}


pub trait BurstEnginePhaseSynapseModelUpdatedForBurstIndexRollover<FGQ, BED>:
BurstEnginePhaseProcessor<FGQ, BED>
where
    FGQ: FeagiGlobalQuantization,
    BED: BurstEngineData<FGQ>,
{}


pub trait BurstEnginePhaseFCLConsolidation<FGQ, BED>:
BurstEnginePhaseProcessor<FGQ, BED>
where
    FGQ: FeagiGlobalQuantization,
    BED: BurstEngineData<FGQ>,
{}


pub trait BurstEnginePhaseNeuronDynamics<FGQ, BED>:
BurstEnginePhaseProcessor<FGQ, BED>
where
    FGQ: FeagiGlobalQuantization,
    BED: BurstEngineData<FGQ>,
{}


pub trait BurstEnginePhaseUpdateFiringNeuronBitfield<FGQ, BED>:
BurstEnginePhaseProcessor<FGQ, BED>
where
    FGQ: FeagiGlobalQuantization,
    BED: BurstEngineData<FGQ>,
{}


pub trait BurstEnginePhaseCountFiringNeuronsPerCorticalArea<FGQ, BED>:
BurstEnginePhaseProcessor<FGQ, BED>
where
    FGQ: FeagiGlobalQuantization,
    BED: BurstEngineData<FGQ>,
{}


pub trait BurstEnginePhasePreSynapseDataExchange<FGQ, BED>:
BurstEnginePhaseProcessor<FGQ, BED>
where
    FGQ: FeagiGlobalQuantization,
    BED: BurstEngineData<FGQ>,
{}


pub trait BurstEnginePhaseFiringNeuronConsolidation<FGQ, BED>:
BurstEnginePhaseProcessor<FGQ, BED>
where
    FGQ: FeagiGlobalQuantization,
    BED: BurstEngineData<FGQ>,
{}


pub trait BurstEnginePhaseSynapseDynamics<FGQ, BED>:
BurstEnginePhaseProcessor<FGQ, BED>
where
    FGQ: FeagiGlobalQuantization,
    BED: BurstEngineData<FGQ>,
{}


pub trait BurstEnginePhasePostSynapseDataExchange<FGQ, BED>:
BurstEnginePhaseProcessor<FGQ, BED>
where
    FGQ: FeagiGlobalQuantization,
    BED: BurstEngineData<FGQ>,
{}


pub trait BurstEnginePhaseFCLCConsolidation<FGQ, BED>:
BurstEnginePhaseProcessor<FGQ, BED>
where
    FGQ: FeagiGlobalQuantization,
    BED: BurstEngineData<FGQ>,
{}