use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::specifications::brain_regions::BrainRegionID;
use crate::genome::genome_editing::GenomeEditSequence;
use crate::genome::genome_object::GenomeObjectMetadata;
use crate::genome::genome_object::GenomeObjectPositioning;
use feagi_connectome::connectome_basis::neuron_model::CorticalAreaInitEnum;

pub struct GenomePlayWrightCorticalArea<'a, FIQ: FeagiIndexQuantization>(&'a mut GenomeEditSequence<FIQ>);


impl<'a, FIQ: FeagiIndexQuantization> GenomePlayWrightCorticalArea<'a, FIQ> {
    /// Add a cortical area
    pub fn add(
        self,
        metadata: GenomeObjectMetadata,
        positioning: GenomeObjectPositioning,
        parent_region: BrainRegionID,
        init: CorticalAreaInitEnum<FIQ>,
    ) -> Result<&'a mut GenomeEditSequence<FIQ>, ()> {

        // TODO error checking



        //self.0.0.push();
        Ok(self.0)
    }



    // TODO Remove

    // TODO Edit Layout

    // TODO Edit properties

    // TODO Edit variance
}