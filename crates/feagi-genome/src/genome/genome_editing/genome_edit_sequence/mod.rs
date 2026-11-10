use crate::basis::cortical_area::FauxCorticalIdentifier;
use crate::genome::genome_editing::genome_edit_sequence::brain_region::GenomeEditStepBrainRegionEnum;
use crate::genome::genome_editing::genome_edit_sequence::cortical_area::GenomeEditStepCorticalAreaEnum;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use std::collections::HashSet;


pub mod brain_region;
pub mod cortical_area;


//region Sequence

pub struct GenomeEditSequence<FIQ: FeagiIndexQuantization> {
    pub(crate) intensity: GenomeEditSequenceIntensity,
    pub(crate) steps: Vec<GenomeEditSequenceStepEnum<FIQ>>,
    pub(crate) contained_faux_cortical_ids: HashSet<FauxCorticalIdentifier>
}


impl<FIQ: FeagiIndexQuantization> GenomeEditSequence<FIQ>
{
    pub fn new() -> Self {
        Self {
            intensity: GenomeEditSequenceIntensity::Empty,
            steps: vec![],
            contained_faux_cortical_ids: Default::default(),
        }
    }

    fn inject_step(&mut self, step: GenomeEditSequenceStepEnum<FIQ>) {
        self.intensity = core::cmp::max(self.intensity, step.get_intensity());
        self.steps.push(step)
    }
}

impl<FIQ: FeagiIndexQuantization> Default for GenomeEditSequence<FIQ>
{
    fn default() -> Self {
        Self::new()
    }
}

//endregion


//region Step

pub(crate) trait GenomeEditSequenceStep<FIQ: FeagiIndexQuantization>: Into<GenomeEditSequenceStepEnum<FIQ>> {

    fn get_intensity(&self) -> GenomeEditSequenceIntensity;

   // type ConnectomeEditStep;

}

// TODO macro

pub(crate) enum GenomeEditSequenceStepEnum<FIQ: FeagiIndexQuantization> {
    BrainRegion(GenomeEditStepBrainRegionEnum),
    CorticalArea(GenomeEditStepCorticalAreaEnum<FIQ>),
}

impl<FIQ: FeagiIndexQuantization> GenomeEditSequenceStepEnum<FIQ> {
    pub fn get_intensity(&self) -> GenomeEditSequenceIntensity {
        match &self {
            GenomeEditSequenceStepEnum::BrainRegion(s) => {
                GenomeEditSequenceIntensity::GenomeOnly
            }
            GenomeEditSequenceStepEnum::CorticalArea(s) => {
                s.get_intensity()
            }
        }

    }
}

//endregion

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
    ConnectomeAllocating = 3
}

