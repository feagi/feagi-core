use std::collections::HashSet;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::specifications::brain_regions::BrainRegionID;
use crate::cortical_area::CorticalIdentifier;
use crate::genome::containers::brain_region::{BrainRegionContainerInnerRuntime, BrainRegionContainerRuntime};
use crate::genome::genome_editing::genome_edit_steps::{GenomeEditStepType, PreparedGenomeEditStep};
use crate::genome::genome_editing::{GenomeEditError, GenomeEditSequence, GenomeEditStepResult};
use crate::genome::genome_object::GenomeObjectMetadata;
use crate::genome::genome_object::GenomeObjectPositioning;
use crate::genome::genomes::designer_genome::DesignerGenome;

pub(crate) enum GenomeEditStepBrainRegionEnum {
    AddRegion(GenomeEditStepBrainRegionAddRegion),
    /*
    RemoveRegion(),
    MoveCorticalAreaToRegion(),
    MoveBrainRegionToRegion(),
    TranslateBrainRegion(),
    EditNameDescriptionBrainRegion(),
    DeleteBrainRegion(),

     */
}

impl<FIQ: FeagiIndexQuantization> GenomeEditStepType<FIQ> for GenomeEditStepBrainRegionEnum {
    fn genome_validity_check(&self, genome: &DesignerGenome<FIQ>) -> Result<(), GenomeEditError> {
        match self {
            GenomeEditStepBrainRegionEnum::AddRegion(step) => step.genome_validity_check(genome),
        }
    }

    fn construct_connectome_edit_step(self, genome: &DesignerGenome<FIQ>) -> Result<PreparedGenomeEditStep<FIQ>, GenomeEditError> {
        match self {
            GenomeEditStepBrainRegionEnum::AddRegion(step) => step.construct_connectome_edit_step(genome),
        }
    }
}


/// Adds a brain region within another. Only the genome holds brain regions, so the connectome is
/// not edited
pub(crate) struct GenomeEditStepBrainRegionAddRegion {
    pub parent_region: BrainRegionID,
    pub new_region: BrainRegionID,
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
}

impl<FIQ: FeagiIndexQuantization> GenomeEditStepType<FIQ> for GenomeEditStepBrainRegionAddRegion {
    fn genome_validity_check(&self, genome: &DesignerGenome<FIQ>) -> Result<(), GenomeEditError> {
        if genome.brain_region(&self.parent_region).is_none() {
            return Err(GenomeEditError::BrainRegionNotFound(self.parent_region));
        }
        if genome.brain_region(&self.new_region).is_some() {
            return Err(GenomeEditError::BrainRegionAlreadyExists(self.new_region));
        }
        Ok(())
    }

    fn construct_connectome_edit_step(self, _genome: &DesignerGenome<FIQ>) -> Result<PreparedGenomeEditStep<FIQ>, GenomeEditError> {
        let Self { parent_region, new_region, metadata, positioning } = self;
        Ok(PreparedGenomeEditStep::genome_only(Box::new(move |genome, _connectome_edit_response| {
            let parent = genome
                .brain_regions_mut()
                .get_mut(&parent_region)
                .ok_or(GenomeEditError::BrainRegionNotFound(parent_region))?;
            parent.contained_regions_mut().insert(new_region);
            genome.brain_regions_mut().insert(
                new_region,
                BrainRegionContainerRuntime::Inner(BrainRegionContainerInnerRuntime {
                    metadata,
                    positioning,
                    parent_id: parent_region,
                    ancestor_id: new_region.into(),
                    contained_areas: HashSet::new(),
                    contained_regions: HashSet::new(),
                }),
            );
            Ok(GenomeEditStepResult::BrainRegionAdded(new_region))
        })))
    }
}


pub struct GenomeEditStepBrainRegionBuilder<'a, FIQ: FeagiIndexQuantization>(&'a mut GenomeEditSequence<FIQ>);

impl<'a, FIQ: FeagiIndexQuantization> GenomeEditStepBrainRegionBuilder<'a, FIQ> {
    pub(crate) fn new(field0: &'a mut GenomeEditSequence<FIQ>) -> Self {
        Self(field0)
    }
}

impl<'a, FIQ: FeagiIndexQuantization> GenomeEditStepBrainRegionBuilder<'a, FIQ> {
    /// Adds a brain region within `parent_region`, with a new ID. The ID is returned in the result
    /// of this step
    pub fn add_now(self, parent_region: BrainRegionID, metadata: GenomeObjectMetadata, positioning: GenomeObjectPositioning) -> Result<&'a mut GenomeEditSequence<FIQ>, ()> {
        self.add_specific(parent_region, BrainRegionID::new_now(), metadata, positioning)
    }

    /// Adds a brain region within `parent_region`, with the given ID
    pub fn add_specific(
        self,
        parent_region: BrainRegionID,
        new_region: BrainRegionID,
        metadata: GenomeObjectMetadata,
        positioning: GenomeObjectPositioning,
    ) -> Result<&'a mut GenomeEditSequence<FIQ>, ()> {
        let step = GenomeEditStepBrainRegionAddRegion { parent_region, new_region, metadata, positioning };
        self.0.push_step(GenomeEditStepBrainRegionEnum::AddRegion(step).into());
        Ok(self.0)
    }

    // TODO the following do not add a step yet

    pub fn remove(self, region_id: BrainRegionID) -> Result<&'a mut GenomeEditSequence<FIQ>, ()> {
        Ok(self.0)
    }

    pub fn move_cortical_area_to_region(self, area: CorticalIdentifier, to_region: BrainRegionID) -> Result<&'a mut GenomeEditSequence<FIQ>, ()> {
        Ok(self.0)
    }

    pub fn move_brain_region_to_region(self, moving_region: BrainRegionID, to_region: BrainRegionID) -> Result<&'a mut GenomeEditSequence<FIQ>, ()> {
        Ok(self.0)
    }

    pub fn translate_brain_region(self, region: BrainRegionID, new_position: GenomeObjectPositioning) -> Result<&'a mut GenomeEditSequence<FIQ>, ()> {
        Ok(self.0)
    }

    pub fn edit_name_description_brain_region(self, region: BrainRegionID, metadata: GenomeObjectMetadata)-> Result<&'a mut GenomeEditSequence<FIQ>, ()> {
        Ok(self.0)
    }

    pub fn delete_region(self, deleting: BrainRegionID) -> Result<&'a mut GenomeEditSequence<FIQ>, ()> {
        Ok(self.0)
    }
}
