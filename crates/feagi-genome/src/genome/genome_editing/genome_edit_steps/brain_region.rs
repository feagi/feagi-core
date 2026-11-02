use crate::basis::specifications::brain_regions::BrainRegionID;
use crate::basis::specifications::cortical_area::cortical_id::CorticalID;
use crate::genome::genome_editing::GenomeEditSequence;
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::genome::genome_editing::genome_edit_steps::{GenomeEditStepEnum, GenomeEditStepType};
use crate::genome::genome_object_metadata::GenomeObjectMetadata;
use crate::genome::genome_object_positioning::GenomeObjectPositioning;

pub(crate) enum GenomeEditStepBrainRegionEnum {
    AddRegion(GenomeEditStepBrainRegionAddRegion),
    RemoveRegion(),
    MoveCorticalAreaToRegion(),
    MoveBrainRegionToRegion(),
    TranslateBrainRegion(),
    EditNameDescriptionBrainRegion(),
    DeleteBrainRegion(),
}

impl<FIQ: FeagiIndexQuantization> GenomeEditStepType<FIQ> for GenomeEditStepBrainRegionEnum {
    type SuperEditEnum = Self;
    type ConnectomeEditStep = ();

    fn wrap_to_sub_category_enum(self) -> Self::SuperEditEnum {
        self
    }

    fn genome_validity_check(&self) -> Result<(), ()> {
        match self {
            GenomeEditStepBrainRegionEnum::AddRegion(step) => {}
            GenomeEditStepBrainRegionEnum::RemoveRegion(step) => {}
            GenomeEditStepBrainRegionEnum::MoveCorticalAreaToRegion(step) => {}
            GenomeEditStepBrainRegionEnum::MoveBrainRegionToRegion(step) => {}
            GenomeEditStepBrainRegionEnum::TranslateBrainRegion(step) => {}
            GenomeEditStepBrainRegionEnum::EditNameDescriptionBrainRegion(step) => {}
            GenomeEditStepBrainRegionEnum::DeleteBrainRegion(step) => {}
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
    type SuperEditEnum = ();

    type ConnectomeEditStep = ();

    fn wrap_to_sub_category_enum(self) -> Self::SuperEditEnum {
        todo!()
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

    fn wrap_to_enum_and_inject(self) -> GenomeEditStepEnum<FIQ> {
        let wrapper = GenomeEditStepBrainRegionEnum::AddRegion(self);
        let wrapper = GenomeEditStepEnum::BrainRegion(wrapper);
        
    }
}




pub struct GenomeEditStepBrainRegionBuilder<'a, FIQ: FeagiIndexQuantization>(&'a mut GenomeEditSequence<FIQ>);

impl<'a, FIQ: FeagiIndexQuantization> GenomeEditStepBrainRegionBuilder<'a, FIQ> {
    pub fn add_now(self, parent_region: BrainRegionID) -> Result<&'a mut GenomeEditSequence<FIQ>, ()> {
        Ok(self.0)
    }

    pub fn add_specific(self, parent_region: BrainRegionID, new_region: BrainRegionID) -> Result<&'a mut GenomeEditSequence<FIQ>, ()> {
        let step = GenomeEditStepBrainRegionAddRegion {
            parent_region,
            new_region,
        };
        
        
        Ok(self.0)
    }

    pub fn remove(self, region_id: BrainRegionID) -> Result<&'a mut GenomeEditSequence<FIQ>, ()> {
        Ok(self.0)
    }

    pub fn move_cortical_area_to_region(self, area: CorticalID, to_region: BrainRegionID) -> Result<&'a mut GenomeEditSequence<FIQ>, ()> {
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

