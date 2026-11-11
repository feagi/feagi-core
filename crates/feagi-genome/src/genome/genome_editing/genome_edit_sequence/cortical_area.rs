use crate::basis::brain_region::BrainRegionID;
use crate::basis::cortical_area::CTStorageOrGeneratorTyped;
use crate::genome::genome_editing::genome_edit_sequence::{GenomeEditSequence, GenomeEditSequenceIntensity, GenomeEditSequenceStep, GenomeEditSequenceStepEnum};
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::cortical_area::cortical_identifiers::FauxCorticalIdentifier;

pub struct GenomeEditStepCorticalAreaBuilder<'a, FIQ: FeagiIndexQuantization>(pub(super) &'a mut GenomeEditSequence<FIQ>);

impl<'a, FIQ: FeagiIndexQuantization> GenomeEditStepCorticalAreaBuilder<'a, FIQ> {

    pub fn add_new_area(
        self,
        parent_region: BrainRegionID,
        metadata: GenomeObjectMetadata,
        positioning: GenomeObjectPositioning,
        faux_cortical_identifier: FauxCorticalIdentifier,
        generator: CTStorageOrGeneratorTyped<FIQ>
    ) -> Result<&'a mut GenomeEditSequence<FIQ>, ()> {
        // true if not contained, false if did
        if self.0.contained_faux_cortical_ids.insert(faux_cortical_identifier) {
            let step = GenomeEditStepCorticalAreaAddArea {
                parent_region,
                metadata,
                positioning,
                faux_cortical_identifier: faux_cortical_identifier.clone(),
                generator,
            };
            
            self.0.inject_step(step.into());
            return Ok(self.0)
        }
        Err(())
    }

}


impl<'a, FIQ: FeagiIndexQuantization> GenomeEditStepCorticalAreaBuilder<'a, FIQ> {

}

// TODO macro

pub(crate) enum GenomeEditStepCorticalAreaEnum<FIQ: FeagiIndexQuantization> {
    AddCorticalArea(GenomeEditStepCorticalAreaAddArea<FIQ>),
}

impl<FIQ: FeagiIndexQuantization> GenomeEditStepCorticalAreaEnum<FIQ> {
    pub(crate) fn get_intensity(&self) -> GenomeEditSequenceIntensity {
        match &self {
            GenomeEditStepCorticalAreaEnum::AddCorticalArea(s) => s.get_intensity(),
        }
    }
}

//region Add Cortical Area




//endregion

pub(crate) struct GenomeEditStepCorticalAreaAddArea<FIQ: FeagiIndexQuantization> {
    pub(crate) parent_region: BrainRegionID,
    pub(crate) metadata: GenomeObjectMetadata,
    pub(crate) positioning: GenomeObjectPositioning,
    pub(crate) faux_cortical_identifier: FauxCorticalIdentifier,
    pub(crate) generator: CTStorageOrGeneratorTyped<FIQ>
}

impl<FIQ: FeagiIndexQuantization> Into<GenomeEditSequenceStepEnum<FIQ>> for GenomeEditStepCorticalAreaAddArea<FIQ> {
    fn into(self) -> GenomeEditSequenceStepEnum<FIQ> {
        GenomeEditSequenceStepEnum::<FIQ>::CorticalArea(GenomeEditStepCorticalAreaEnum::<FIQ>::AddCorticalArea(self))
    }
}

impl<FIQ: FeagiIndexQuantization> GenomeEditSequenceStep<FIQ> for GenomeEditStepCorticalAreaAddArea<FIQ> {
    fn get_intensity(&self) -> GenomeEditSequenceIntensity {
        GenomeEditSequenceIntensity::ConnectomeAllocating
    }
}