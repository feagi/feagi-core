use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::basis::specifications::brain_regions::BrainRegionID;
use crate::genome::genome_object_metadata::GenomeObjectMetadata;
use crate::genome::genome_object_positioning::GenomeObjectPositioning;


pub struct GenomePlayWright<FIQ: FeagiIndexQuantization>(Vec<GenomePlayWrightItem<FIQ>>);

impl<FIQ: FeagiIndexQuantization> GenomePlayWright<FIQ> {
    pub fn new() -> Self {
        GenomePlayWright(Vec::new())
    }

    // TODO connectivity rules

    pub fn cortical_area(&mut self) -> GenomePlayWrightCorticalArea<FIQ> {
        GenomePlayWrightCorticalArea(self)
    }

    // TODO Mappings
}


//region Cortical Area
pub struct GenomePlayWrightCorticalArea<'a, FIQ: FeagiIndexQuantization>(&'a mut GenomePlayWright<FIQ>);


impl<'a, FIQ: FeagiIndexQuantization> GenomePlayWrightCorticalArea<'a, FIQ> {
    /// Add a cortical area
    pub fn add(
        self,
        metadata: GenomeObjectMetadata,
        positioning: GenomeObjectPositioning,
        parent_region: BrainRegionID,
        generator: CorticalAreaGeneratorEnum<FIQ>,
    ) -> Result<&'a mut GenomePlayWright<FIQ>, ()> {

        // TODO error checking

        let item = GenomePlayWrightItem::AddCorticalArea {
            metadata,
            positioning,
            parent_region,
            generator,
        };

        self.0.0.push(item);
        Ok(self.0)
    }

    // TODO Remove

    // TODO Edit Layout

    // TODO Edit properties

    // TODO Edit variance
}

//endregion




pub(crate) enum GenomePlayWrightItem<FIQ: FeagiIndexQuantization> {
    AddCorticalArea {
        metadata: GenomeObjectMetadata,
        positioning: GenomeObjectPositioning,
        parent_region: BrainRegionID,
        generator: CorticalAreaGeneratorEnum<FIQ> 
    },
}
