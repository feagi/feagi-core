use crate::burst_engine_definitions::burst_phase_output::BurstPhaseOutput;
use crate::burst_engine_definitions::burst_phases::RunBurstPhase;
use crate::errors::BurstEngineError;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::wrapped_indexes::BurstIndex;

/// Base trait for all burst engines. Bursts are executed by phases in an async manner
pub trait BurstEngine<FIQ: FeagiIndexQuantization> {
    fn execute_phase(&mut self, phases: RunBurstPhase, burst_index: BurstIndex<FIQ::BurstIndexQuant>) -> impl core::future::Future<Output = Result<BurstPhaseOutput<FIQ>, BurstEngineError>>;
}

/// A marker trait to denote a burst engine as not being editable
pub trait NonComposableBurstEngine<FIQ: FeagiIndexQuantization>: BurstEngine<FIQ> {}