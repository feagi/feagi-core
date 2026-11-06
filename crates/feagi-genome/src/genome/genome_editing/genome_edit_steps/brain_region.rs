use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::specifications::brain_regions::BrainRegionID;
use crate::cortical_area::CorticalIdentifier;
use crate::genome::genome_editing::genome_edit_steps::GenomeEditStepType;
use crate::genome::genome_editing::GenomeEditSequence;
use crate::genome::genome_object::GenomeObjectMetadata;
use crate::genome::genome_object::GenomeObjectPositioning;

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
    type SuperEditEnum = Self;
    type ConnectomeEditStep = ();

    fn wrap_to_sub_category_enum(self) -> Self::SuperEditEnum {
        self
    }

    fn genome_validity_check(&self) -> Result<(), ()> {
        match self {
            GenomeEditStepBrainRegionEnum::AddRegion(step) => {
                Ok(())
            }
            /*
            GenomeEditStepBrainRegionEnum::RemoveRegion(step) => {}
            GenomeEditStepBrainRegionEnum::MoveCorticalAreaToRegion(step) => {}
            GenomeEditStepBrainRegionEnum::MoveBrainRegionToRegion(step) => {}
            GenomeEditStepBrainRegionEnum::TranslateBrainRegion(step) => {}
            GenomeEditStepBrainRegionEnum::EditNameDescriptionBrainRegion(step) => {}
            GenomeEditStepBrainRegionEnum::DeleteBrainRegion(step) => {}

             */
        }
    }

    fn construct_connectome_edit_step(&self) -> Self::ConnectomeEditStep {
        todo!()
    }

    fn apply_edit_to_genome(&self, connectome_edit_response: (), genome: ()) -> Result<(), ()> {
        todo!()
    }
}



pub(crate) struct GenomeEditStepBrainRegionAddRegion {
    pub parent_region: BrainRegionID,
    pub new_region: BrainRegionID
}

impl<FIQ: FeagiIndexQuantization> GenomeEditStepType<FIQ> for GenomeEditStepBrainRegionAddRegion {
    type SuperEditEnum = GenomeEditStepBrainRegionEnum;

    type ConnectomeEditStep = ();

    fn wrap_to_sub_category_enum(self) -> Self::SuperEditEnum {
        GenomeEditStepBrainRegionEnum::AddRegion(self)
    }

    fn genome_validity_check(&self) -> Result<(), ()> {
        todo!()
    }

    fn construct_connectome_edit_step(&self) -> Self::ConnectomeEditStep {
        todo!()
    }

    fn apply_edit_to_genome(&self, connectome_edit_response: (), genome: ()) -> Result<(), ()> {
        todo!()
    }
}




pub struct GenomeEditStepBrainRegionBuilder<'a, FIQ: FeagiIndexQuantization>(&'a mut GenomeEditSequence<FIQ>);

impl<'a, FIQ: FeagiIndexQuantization> GenomeEditStepBrainRegionBuilder<'a, FIQ> {
    pub(crate) fn new(field0: &'a mut GenomeEditSequence<FIQ>) -> Self {
        Self(field0)
    }
}

impl<'a, FIQ: FeagiIndexQuantization> GenomeEditStepBrainRegionBuilder<'a, FIQ> {
    pub fn add_now(self, parent_region: BrainRegionID) -> Result<&'a mut GenomeEditSequence<FIQ>, ()> {
        Ok(self.0)
    }

    pub fn add_specific(self, parent_region: BrainRegionID, new_region: BrainRegionID) -> Result<&'a mut GenomeEditSequence<FIQ>, ()> {
        let step = GenomeEditStepBrainRegionAddRegion {
            parent_region,
            new_region,
        };
        GenomeEditStepType::<FIQ>::wrap_to_enum_and_inject(step, self.0);
        Ok(self.0)
    }

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

