use crate::basis::specifications::brain_regions::BrainRegionID;
use crate::basis::specifications::cortical_area::cortical_id::CorticalID;
use crate::genome::genome_playwright::GenomePlayWright;
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::genome::genome_object_metadata::GenomeObjectMetadata;
use crate::genome::genome_object_positioning::GenomeObjectPositioning;

pub struct GenomePlayWrightBrainRegion<'a, FIQ: FeagiIndexQuantization>(&'a mut GenomePlayWright<FIQ>);

impl<'a, FIQ: FeagiIndexQuantization> GenomePlayWrightBrainRegion<'a, FIQ> {
    pub fn add_new_region_now(self, parent_region: BrainRegionID) -> Result<&'a mut GenomePlayWright<FIQ>, ()> {
        Ok(self.0)
    }

    pub fn add_region(self, new_region: BrainRegionID, parent_region: BrainRegionID) -> Result<&'a mut GenomePlayWright<FIQ>, ()> {
        Ok(self.0)
    }

    pub fn remove(self, region_id: BrainRegionID) -> Result<&'a mut GenomePlayWright<FIQ>, ()> {
        Ok(self.0)
    }

    pub fn move_cortical_area_to_region(self, area: CorticalID, to_region: BrainRegionID) -> Result<&'a mut GenomePlayWright<FIQ>, ()> {
        Ok(self.0)
    }

    pub fn move_brain_region_to_region(self, moving_region: BrainRegionID, to_region: BrainRegionID) -> Result<&'a mut GenomePlayWright<FIQ>, ()> {
        Ok(self.0)
    }

    pub fn translate_brain_region(self, region: BrainRegionID, new_position: GenomeObjectPositioning) -> Result<&'a mut GenomePlayWright<FIQ>, ()> {
        Ok(self.0)
    }

    pub fn edit_meta_brain_region(self, region: BrainRegionID, metadata: GenomeObjectMetadata)-> Result<&'a mut GenomePlayWright<FIQ>, ()> {
        Ok(self.0)
    }

    pub fn delete_region(self, deleting: BrainRegionID) -> Result<&'a mut GenomePlayWright<FIQ>, ()> {
        Ok(self.0)
    }
}
