use feagi_models::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;
use feagi_npu_burst_engines::feagi_npu_burst_core::burst_engine_definitions::burst_phases::RunBurstPhase;

pub enum ComposableBurstEngineWorkerCommand<NPUIQ: NeuronProcessingUnitIndexQuantization> {
    /// Run the burst engine, either the default full burst or a specific phase.
    RunPhases {
        burst_index: BurstIndex<FIQ::GlobalBurstIndexQuant>,
        phase: RunBurstPhase,
    },
    BurstIndexRollback {
        burst_index: BurstIndex<FIQ::GlobalBurstIndexQuant>,
    },
    // TODO connectome edit
    Stop,
}