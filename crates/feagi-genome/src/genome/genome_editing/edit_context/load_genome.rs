use feagi_basis::target_frequency::TargetFrequency;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::brain_region::{BrainRegionAncestryID, BrainRegionID, BrainRegionIdAndAncestryConstructor};
use crate::genome::genome_editing::edit_context::GenomeEditSequenceContext;
use crate::genome::genome_editing::GenomeEditSequence;
use crate::genome::genome_editing::steps::brain_region::GenomeEditStepBrainRegionAddingOnlyEnum;
use crate::genome::genome_editing::steps::cortical_area::GenomeEditStepLoadGenomeCorticalAreaEnum;
use crate::genome::genome_object::GenomeObjectMetadata;

/// A Genome Edit Sequence specific for loading a new genome. Will cause a genome reload.
/// Can be sent empty without further steps to load a void genome. Steps can only add data, not
/// edit or remove anything
pub struct GenomeEditSequenceContextLoadGenome {
    genome_metadata: GenomeObjectMetadata,
    target_burst_rate: TargetFrequency,
    new_root_ids: BrainRegionIdAndAncestryConstructor
}

impl GenomeEditSequenceContextLoadGenome {
    pub fn genome_metadata(&self) -> &GenomeObjectMetadata {
        &self.genome_metadata
    }

    pub fn target_burst_rate(&self) -> TargetFrequency {
        self.target_burst_rate
    }

    pub fn new_root_ids(&self) -> &BrainRegionIdAndAncestryConstructor {
        &self.new_root_ids
    }
}

impl GenomeEditSequenceContext for GenomeEditSequenceContextLoadGenome {
    type BrainRegionStepEnum = GenomeEditStepBrainRegionAddingOnlyEnum;
    type CorticalAreaStepEnum<FIQ: FeagiIndexQuantization> = GenomeEditStepLoadGenomeCorticalAreaEnum<FIQ>;
}

// Impls for `GenomeEditSequence` specific to this
impl<FIQ: FeagiIndexQuantization> GenomeEditSequence<FIQ, GenomeEditSequenceContextLoadGenome> {

    /// Builds a new request to reset to a void empty genome
    pub fn reset_to_void_genome(
        genome_metadata: GenomeObjectMetadata,
        target_burst_rate: TargetFrequency,
        new_root_ids: BrainRegionIdAndAncestryConstructor,
    ) -> Self {
        Self {
            context: GenomeEditSequenceContextLoadGenome {
                genome_metadata,
                target_burst_rate,
                new_root_ids,
            },
            steps: vec![],
        }
    }

    // TODO other genomes
}