use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::brain_region_identifiers::{BrainRegionAncestryID, BrainRegionID};
use crate::genome::genome_editing::genome_edit_sequence::{
    GenomeEditSequence, GenomeEditSequenceIntensity, GenomeEditSequenceStep, GenomeEditSequenceStepEnum,
};
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};

pub struct GenomeEditStepBrainRegionBuilder<'a, FIQ: FeagiIndexQuantization>(&'a mut GenomeEditSequence<FIQ>);

impl<'a, FIQ: FeagiIndexQuantization> GenomeEditStepBrainRegionBuilder<'a, FIQ> {
    /// Add a new brain region with the current timestamp for the ID and the Ancestry ID
    pub fn add_new_brain_region(
        self,
        parent_region: BrainRegionID,
        metadata: GenomeObjectMetadata,
        positioning: GenomeObjectPositioning,
    ) -> Result<&'a mut GenomeEditSequence<FIQ>, ()> {
        let new_id = BrainRegionID::new_now();
        let step = GenomeEditStepBrainRegionAddRegion {
            parent_region,
            new_region: new_id,
            new_region_ancestor: new_id.into(),
            metadata,
            positioning,
        };
        self.0.inject_step(step.into());
        Ok(self.0)
    }

    /// Add a brain region with an existing Ancestry ID
    pub fn import_brain_region(
        self,
        parent_region: BrainRegionID,
        new_region_ancestor: BrainRegionAncestryID,
        metadata: GenomeObjectMetadata,
        positioning: GenomeObjectPositioning,
    ) -> Result<&'a mut GenomeEditSequence<FIQ>, ()> {
        let step = GenomeEditStepBrainRegionAddRegion {
            parent_region,
            new_region: BrainRegionID::new_now(),
            new_region_ancestor,
            metadata,
            positioning,
        };
        self.0.inject_step(step.into());
        Ok(self.0)
    }
}

// TODO macro

pub(crate) enum GenomeEditStepBrainRegionEnum {
    AddRegion(GenomeEditStepBrainRegionAddRegion),
    /*
    RepositionRegion(),
    RemoveRegion(),
    MoveCorticalAreaToRegion(),
    MoveBrainRegionToRegion(),
    TranslateBrainRegion(),
    EditNameDescriptionBrainRegion(),
    DeleteBrainRegion(),

     */
}

//region Add Region
pub(crate) struct GenomeEditStepBrainRegionAddRegion {
    pub(crate) parent_region: BrainRegionID,
    pub(crate) new_region: BrainRegionID,
    pub(crate) new_region_ancestor: BrainRegionAncestryID,
    pub(crate) metadata: GenomeObjectMetadata,
    pub(crate) positioning: GenomeObjectPositioning,
}

impl<FIQ: FeagiIndexQuantization> Into<GenomeEditSequenceStepEnum<FIQ>> for GenomeEditStepBrainRegionAddRegion {
    fn into(self) -> GenomeEditSequenceStepEnum<FIQ> {
        GenomeEditSequenceStepEnum::<FIQ>::BrainRegion(GenomeEditStepBrainRegionEnum::AddRegion(self))
    }
}

impl<FIQ: FeagiIndexQuantization> GenomeEditSequenceStep<FIQ> for GenomeEditStepBrainRegionAddRegion {
    fn get_intensity(&self) -> GenomeEditSequenceIntensity {
        GenomeEditSequenceIntensity::GenomeOnly
    }
}

//endregion
