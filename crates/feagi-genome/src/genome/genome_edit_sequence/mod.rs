pub mod cortical_area;
pub mod brain_region;
pub mod connectivity_rule;
pub mod cortical_mapping;
pub mod items;

use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::basis::specifications::brain_regions::BrainRegionID;
use crate::genome::genome_object_metadata::GenomeObjectMetadata;
use crate::genome::genome_object_positioning::GenomeObjectPositioning;
use crate::genome::genome_playwright::cortical_area::GenomePlayWrightCorticalArea;

pub struct GenomePlayWright<FIQ: FeagiIndexQuantization>(Vec<GenomePlayWrightItem<FIQ>>);

impl<FIQ: FeagiIndexQuantization> GenomePlayWright<FIQ> {
    pub fn new() -> Self {
        GenomePlayWright::<FIQ>(Vec::new())
    }

    pub fn brain_region(&mut self) -> () {}

    pub fn cortical_area(&mut self) -> GenomePlayWrightCorticalArea<FIQ> {
        GenomePlayWrightCorticalArea(self)
    }

    pub fn connectivity_rule(&mut self) -> () {}

    pub fn cortical_mapping(&mut self) -> () {}
}




pub(crate) enum GenomePlayWrightItem<FIQ: FeagiIndexQuantization> {
    AddCorticalArea {
        metadata: GenomeObjectMetadata,
        positioning: GenomeObjectPositioning,
        parent_region: BrainRegionID,
        generator: CorticalAreaGeneratorEnum<FIQ> 
    },
}
