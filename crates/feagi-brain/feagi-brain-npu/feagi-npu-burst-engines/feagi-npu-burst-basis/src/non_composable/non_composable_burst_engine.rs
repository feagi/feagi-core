use feagi_basis::prelude::*;
use crate::burst_phases::RunBurstPhase;
use crate::errors::BurstEngineError;
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
