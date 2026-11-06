use feagi_connectome::connectome_basis::connectome_editing::{ConnectomeDispatch, ConnectomeEditResultantEnum, ConnectomeEditStepEnum};
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use crate::genome::genome_editing::genome_edit_steps::brain_region::GenomeEditStepBrainRegionEnum;
use crate::genome::genome_editing::genome_edit_steps::cortical_area::GenomeEditStepCorticalAreaEnum;
use crate::genome::genome_editing::{GenomeEditError, GenomeEditStepResult};
use crate::genome::genomes::designer_genome::DesignerGenome;

pub mod cortical_area;
pub mod brain_region;
pub mod connectivity_rule;
pub mod cortical_mapping;


/// A single step of a genome edit sequence
pub(crate) trait GenomeEditStepType<FIQ: FeagiIndexQuantization>: Sized
{
    /// First all genome edit steps are checked in sequence to ensure plausibility, before anything
    /// is changed
    fn genome_validity_check(&self, genome: &DesignerGenome<FIQ>) -> Result<(), GenomeEditError>;

    /// Construct the connectome edit step (`Nothing` for steps that do not affect the connectome),
    /// and how the genome is updated once the connectome has applied it
    fn construct_connectome_edit_step(self, genome: &DesignerGenome<FIQ>) -> Result<PreparedGenomeEditStep<FIQ>, GenomeEditError>;
}

/// How the genome is updated once the connectome applied the edit step, from what the connectome
/// returned (`Nothing` for steps that do not affect the connectome)
pub(crate) type ApplyEditToGenome<FIQ> =
    Box<dyn FnOnce(&mut DesignerGenome<FIQ>, ConnectomeEditResultantEnum<FIQ>) -> Result<GenomeEditStepResult, GenomeEditError>>;

/// A genome edit step, checked and ready to be applied
pub(crate) struct PreparedGenomeEditStep<FIQ: FeagiIndexQuantization> {
    pub(crate) connectome_edit_step: ConnectomeEditStepEnum<FIQ, ConnectomeDispatch>,
    pub(crate) apply_edit_to_genome: ApplyEditToGenome<FIQ>,
}

impl<FIQ: FeagiIndexQuantization> PreparedGenomeEditStep<FIQ> {
    /// A step that only changes the genome
    pub(crate) fn genome_only(apply_edit_to_genome: ApplyEditToGenome<FIQ>) -> Self {
        Self { connectome_edit_step: ConnectomeEditStepEnum::Nothing, apply_edit_to_genome }
    }

    pub(crate) fn edits_connectome(&self) -> bool {
        !matches!(self.connectome_edit_step, ConnectomeEditStepEnum::Nothing)
    }
}


pub(crate) enum GenomeEditStepEnum<FIQ: FeagiIndexQuantization> {
    BrainRegion(GenomeEditStepBrainRegionEnum),
    CorticalArea(GenomeEditStepCorticalAreaEnum<FIQ>),
    // TODO connectivity rules, cortical mappings
}

impl<FIQ: FeagiIndexQuantization> GenomeEditStepType<FIQ> for GenomeEditStepEnum<FIQ> {
    fn genome_validity_check(&self, genome: &DesignerGenome<FIQ>) -> Result<(), GenomeEditError> {
        match self {
            GenomeEditStepEnum::BrainRegion(step) => step.genome_validity_check(genome),
            GenomeEditStepEnum::CorticalArea(step) => step.genome_validity_check(genome),
        }
    }

    fn construct_connectome_edit_step(self, genome: &DesignerGenome<FIQ>) -> Result<PreparedGenomeEditStep<FIQ>, GenomeEditError> {
        match self {
            GenomeEditStepEnum::BrainRegion(step) => step.construct_connectome_edit_step(genome),
            GenomeEditStepEnum::CorticalArea(step) => step.construct_connectome_edit_step(genome),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<GenomeEditStepBrainRegionEnum> for GenomeEditStepEnum<FIQ> {
    fn from(value: GenomeEditStepBrainRegionEnum) -> Self {
        GenomeEditStepEnum::BrainRegion(value)
    }
}

impl<FIQ: FeagiIndexQuantization> From<GenomeEditStepCorticalAreaEnum<FIQ>> for GenomeEditStepEnum<FIQ> {
    fn from(value: GenomeEditStepCorticalAreaEnum<FIQ>) -> Self {
        GenomeEditStepEnum::CorticalArea(value)
    }
}
