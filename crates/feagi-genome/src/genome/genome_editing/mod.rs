pub mod items;
pub mod genome_edit_step;
pub mod genome_edit_steps;

use feagi_connectome::quantization_levels::FeagiIndexQuantization;
use crate::genome::genome_editing::genome_edit_steps::brain_region::GenomeEditStepBrainRegionBuilder;
use crate::genome::genome_editing::genome_edit_steps::GenomeEditStepEnum;

pub struct GenomeEditSequence<FIQ: FeagiIndexQuantization>
{
    requires_connectome_edits: bool,
    sequence: Vec<GenomeEditStepEnum<FIQ>>,
    _p: core::marker::PhantomData<FIQ>,
}


impl<FIQ: FeagiIndexQuantization> GenomeEditSequence<FIQ> {
    pub fn new() -> Self {
        Self {
            requires_connectome_edits: false,
            sequence: vec![],
            _p: core::marker::PhantomData,
        }
    }




    pub fn brain_region(&mut self) -> GenomeEditStepBrainRegionBuilder<'_, FIQ> {
        GenomeEditStepBrainRegionBuilder::new(self)
    }

    pub fn cortical_area(&mut self) -> () {

    }

    pub fn connectivity_rule(&mut self) -> () {}

    pub fn cortical_mapping(&mut self) -> () {}

    pub(crate) fn add_genome_edit_enum(&mut self, genome_edit_step: GenomeEditStepEnum<FIQ>, edits_connectome: bool) {
        self.requires_connectome_edits = self.requires_connectome_edits || edits_connectome;
        self.sequence.push(genome_edit_step)
    }
}














/*
pub(crate) enum GenomePlayWrightItem<FIQ: FeagiIndexQuantization> {
    AddCorticalArea {
        metadata: GenomeObjectMetadata,
        positioning: GenomeObjectPositioning,
        parent_region: BrainRegionID,
        spec: CorticalAreaSpecEnum<FIQ> 
    },
}


 */