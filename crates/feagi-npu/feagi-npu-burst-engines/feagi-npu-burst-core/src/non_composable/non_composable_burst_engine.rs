use crate::burst_phases::RunBurstPhase;
use crate::errors::BurstEngineError;
use feagi_data::quantization_levels::feagi_index_quantization::{FeagiIndexQuantization};
use feagi_models::wrapped_indexes::BurstIndex;
use crate::non_composable::non_composable_burst_phase_output::NonComposableBurstPhaseOutput;

/// Defines a Burst Engine that can execute neuron dynamics
pub trait NonComposableBurstEngine<FIQ: FeagiIndexQuantization>: Sized {
    /// Execute some form of neural computation
    fn execute_phase(
        &mut self,
        phases: RunBurstPhase,
        burst_index: BurstIndex<FIQ::BurstIndexQuant>,
    ) -> impl core::future::Future<
        Output = Result<
            NonComposableBurstPhaseOutput<FIQ>,
            BurstEngineError,
        >,
    >;
}
