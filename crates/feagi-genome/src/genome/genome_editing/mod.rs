use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use steps::GenomeEditStepSequenceStep;
use crate::genome::genome_editing::edit_context::GenomeEditSequenceContext;
use crate::genome::genome_editing::steps::brain_region::{GenomeEditStepBrainRegionError};
use crate::genome::genome_editing::steps::cortical_area::{GenomeEditStepCorticalAreaError};
pub mod steps;
pub mod verified_sequence;
pub mod edit_context;


 // TODO builder instead, do an initial wave of checks, output as a opaque sequence that is 
// serializable

/// A sequence of steps + some initial properties for detailing a change to be made in the genome /
/// connectome
pub struct GenomeEditSequence<FIQ: FeagiIndexQuantization, GESC: GenomeEditSequenceContext> {
    context: GESC,
    steps: Vec<GenomeEditStepSequenceStep<FIQ, GESC>>
}

impl<FIQ: FeagiIndexQuantization, GESC: GenomeEditSequenceContext> GenomeEditSequence<FIQ, GESC> {
    
    /// Adds a step to the end
    pub fn append_step(&mut self, step: GenomeEditStepSequenceStep<FIQ, GESC>) {
        self.steps.push(step)
    }
    
    pub(crate) fn take_context(self) -> GESC {
        self.context
    }
    
    pub(crate) fn get_context(&self) -> &GESC {
        &self.context
    }

    pub(crate) fn get_context_mut(&mut self) -> &mut GESC {
        &mut self.context
    }
    
    pub(crate) fn get_steps(&self) -> &[GenomeEditStepSequenceStep<FIQ, GESC>] {
        &self.steps
    }
}


/// Describes how intense (how long) an edit operation is
#[repr(u8)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum GenomeEditSequenceIntensity {
    /// No edits defined, will be rejected
    Empty = 0,
    /// Edit only touches the genome, no connectome
    GenomeOnly = 1,
    /// Edit involves the connectome but (most likely) does not involve
    /// allocation / deallocation, which is slow
    ConnectomeInPlace = 2,
    /// Requires the connectome to expand / contract, which would be slow
    ConnectomeAllocating = 3,
    /// (Re)laods a genome, guaranteed interruption
    ConnectomeReloading = 4,

}

#[derive(Debug, thiserror::Error)]
pub enum GenomeEditSequenceError {
    #[error(transparent)]
    BrainRegionErr(#[from] GenomeEditStepBrainRegionError),
    #[error(transparent)]
    CorticalAreaErr(#[from] GenomeEditStepCorticalAreaError),

}
