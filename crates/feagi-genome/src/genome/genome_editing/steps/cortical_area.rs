use serde::{Deserialize, Serialize};
use crate::basis::brain_region::BrainRegionID;
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::cortical_area::cortical_identifiers::{CorticalIdentifier, FauxCorticalIdentifier};
use crate::basis::cortical_area::{CorticalTypedSpawningParameters};

/// The enum edit step for cortical areas for the load genome sequence type. Can only add cortical aras
pub enum GenomeEditStepLoadGenomeCorticalAreaEnum<FIQ: FeagiIndexQuantization> {
    AddArea {
        metadata: GenomeObjectMetadata,
        positioning: GenomeObjectPositioning,
        faux_cortical_identifier: FauxCorticalIdentifier,
        neurogenesis: CorticalTypedSpawningParameters<FIQ>, // TODO swap
        parent_region: Option<BrainRegionID> // TODO verify should ensure that type matches target
    }
}


pub enum GenomeEditStepCorticalAreaEditingEnum<FIQ: FeagiIndexQuantization> {
    AddArea {
        metadata: GenomeObjectMetadata,
        positioning: GenomeObjectPositioning,
        faux_cortical_identifier: FauxCorticalIdentifier,
        neurogenesis: CorticalTypedSpawningParameters<FIQ>, // TODO swap
        parent_region: Option<BrainRegionID> // TODO verify should ensure that type matches target
    },
    DeleteArea {
        deleting: CorticalIdentifier
    }
}



#[derive(Debug, thiserror::Error, Clone, Copy, Serialize, Deserialize)]
pub enum GenomeEditStepCorticalAreaError {
    #[error("The given cortical type cannot be added to a root brain region directly!")]
    CorticalTypeCannotBeAddedToRootRegion,
    #[error("The given cortical type cannot be added to an inner brain region directly!")]
    CorticalTypeCannotBeAddedToInnerRegion,
    #[error("The given faux cortical identifier was already used in this sequence!")]
    FauxCorticalIDAlreadyUsed,
}