use std::collections::HashSet;
use feagi_basis::target_frequency::TargetFrequency;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use steps::GenomeEditStepSequenceStep;
use crate::basis::brain_region::{BrainRegionAncestryID, BrainRegionID};
use crate::basis::cortical_area::cortical_identifiers::FauxCorticalIdentifier;
use crate::genome::genome_editing::steps::brain_region::{GenomeEditStepBrainRegionAddingOnlyEnum, GenomeEditStepBrainRegionError};
use crate::genome::genome_editing::steps::cortical_area::{GenomeEditStepCorticalAreaError, GenomeEditStepLoadGenomeCorticalAreaEnum};
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};
pub mod steps;
pub mod genome_edit_sequence_verifier;


pub trait GenomeEditSequenceContext: Sized {
    // Cant put FIQ at top due to `GenomeEditStepSequenceStep`
    
    type BrainRegionStepEnum;
    type CorticalAreaStepEnum<FIQ: FeagiIndexQuantization>;
    
}
 // TODO builder

pub struct GenomeEditSequence<FIQ: FeagiIndexQuantization, GESC: GenomeEditSequenceContext> {
    context: GESC,
    steps: Vec<GenomeEditStepSequenceStep<FIQ, GESC>>
}


impl<FIQ: FeagiIndexQuantization> GenomeEditSequence<FIQ, GenomeEditSequenceContextLoadGenome> {
    
    /// Builds a new request to reset to a barebones empty genome
    pub fn reset_to_barebone_genome(
        genome_metadata: GenomeObjectMetadata,
        target_burst_rate: TargetFrequency,
        new_root_genome_id: BrainRegionID,
        new_root_genome_ancestry_id: BrainRegionAncestryID,
    ) -> Self {
        Self {
            context: GenomeEditSequenceContextLoadGenome {
                genome_metadata,
                target_burst_rate,
                new_root_genome_id,
                new_root_genome_ancestry_id,
            },
            steps: vec![],
        }
    }
    
}



//region Load Genome

pub struct GenomeEditSequenceContextLoadGenome {
    genome_metadata: GenomeObjectMetadata,
    target_burst_rate: TargetFrequency,
    new_root_genome_id: BrainRegionID,
    new_root_genome_ancestry_id: BrainRegionAncestryID,
}

impl GenomeEditSequenceContextLoadGenome {
    pub fn genome_metadata(&self) -> &GenomeObjectMetadata {
        &self.genome_metadata
    }

    pub fn target_burst_rate(&self) -> TargetFrequency {
        self.target_burst_rate
    }

    pub fn new_root_genome_id(&self) -> BrainRegionID {
        self.new_root_genome_id
    }

    pub fn new_root_genome_ancestry_id(&self) -> BrainRegionAncestryID {
        self.new_root_genome_ancestry_id
    }
}

impl GenomeEditSequenceContext for GenomeEditSequenceContextLoadGenome {
    type BrainRegionStepEnum = GenomeEditStepBrainRegionAddingOnlyEnum;
    type CorticalAreaStepEnum<FIQ: FeagiIndexQuantization> = GenomeEditStepLoadGenomeCorticalAreaEnum<FIQ>;
}


/*
//endregion

//region Amalgamate Genome

pub struct GenomeEditSequenceContextAmalgamateGenome {
    spawn_location: GenomeObjectPositioning,
}

impl GenomeEditSequenceContext for GenomeEditSequenceContextAmalgamateGenome {
    type BrainRegionStepEnum<FIQ: FeagiIndexQuantization> = ();
    type CorticalAreaStepEnum<FIQ: FeagiIndexQuantization> = ();
}

//endregion


//region Edit Genome

pub struct GenomeEditSequenceContextEditGenome {
    
}

impl GenomeEditSequenceContext for GenomeEditSequenceContextEditGenome {
    type BrainRegionStepEnum<FIQ: FeagiIndexQuantization> = ();
    type CorticalAreaStepEnum<FIQ: FeagiIndexQuantization> = ();
}

//endregion


 */



//endregion

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
