use serde::{Deserialize, Serialize};
use feagi_connectome::feagi_connectome_neural::neural_basis::connectome_editing::ConnectomeEditStepType;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::basis::brain_region::{BrainRegionID, BrainRegionIdAndAncestryConstructor};
use crate::basis::cortical_area::cortical_identifiers::{CorticalIdentifier, FauxCorticalIdentifier};
use crate::basis::cortical_area::CorticalTypedSpawningParameters;
use crate::genome::genome_editing::{GenomeEditSequenceContext, GenomeEditSequenceIntensity};
use crate::genome::genome_object::{GenomeObjectMetadata, GenomeObjectPositioning};



/// A edit request submitted without being verified first. Verification first checks if the request
/// is plausible, and it used to build the `GenomeValidatedEditStep` with any additional context
/// required
pub(crate) trait GenomeUnvalidatedEditStep<FIQ: FeagiIndexQuantization> {
    type ValidatedStep: GenomeValidatedEditStep<FIQ>;


    const INTENSITY: GenomeEditSequenceIntensity;
}



/// A edit step validated by genome to be valid, and that may have additional context added as a
/// result. May need to wait on connectome step to be done to implement.
pub(crate) trait GenomeValidatedEditStep<FIQ: FeagiIndexQuantization> {
    type GenomeResultant: GenomeEditResultant<FIQ, ConnectomeResultant=<Self::ConnectomeStep as ConnectomeEditStepType<FIQ>>::ConnectomeEditResultant>;
    type ConnectomeStep: ConnectomeEditStepType<FIQ>;

}

pub(crate) trait GenomeEditResultant<FIQ: FeagiIndexQuantization> {
    type ConnectomeResultant;


}



pub(crate) enum GenomeEditSequenceStepLoadingGenome<FIQ: FeagiIndexQuantization> {

}


pub(crate) enum GenomeEditSequenceStepEditingGenome<FIQ: FeagiIndexQuantization> {

}


pub(crate) enum GenomeEditSequenceStepAmalgamation<FIQ: FeagiIndexQuantization> {

}



//region Cortical Area







//endregion








pub(crate) enum GenomeEditStepSequenceStep<FIQ: FeagiIndexQuantization, GESC: GenomeEditSequenceContext> {
    BrainRegion(GESC::BrainRegionStepEnum),
    CorticalArea(GESC::CorticalAreaStepEnum<FIQ>),
    //ConnectivityRule
    //CorticalMapping
}

//region brain region


/// The enum edit step for brain regions for the load genome sequence type. Can only add brain regions
pub enum GenomeEditStepBrainRegionAddingOnlyEnum {
    /// Adds a brain region with the given details. if optional_parent_region is None, the region is
    /// added to the root brain region
    AddRegion {
        metadata: GenomeObjectMetadata,
        positioning: GenomeObjectPositioning,
        this_ids: BrainRegionIdAndAncestryConstructor,
        optional_parent_region: Option<BrainRegionID>,
    }
}


/// The enum edit step for brain regions for editing brain regions in any way
pub enum GenomeEditStepBrainRegionEditingEnum {
    /// Adds a brain region with the given details. if optional_parent_region is None, the region is
    /// added to the root brain region
    AddRegion {
        metadata: GenomeObjectMetadata,
        positioning: GenomeObjectPositioning,
        this_ids: BrainRegionIdAndAncestryConstructor,
        optional_parent_region: Option<BrainRegionID>,
    },
    /// Deletes a given inner region. Anything inside gets moved up first
    DeleteRegion {
        deleting: BrainRegionID
    }
    // TODO others
}



#[derive(Debug, thiserror::Error, Clone, Copy, Serialize, Deserialize)]
pub enum GenomeEditStepBrainRegionError {
    #[error("A Brain region with the given ID is already being added!")]
    BrainRegionWithIDAlreadyBeingAdded,
    #[error("A Brain region with the given ancestryID is already being added!")]
    BrainRegionWithAncestryIDAlreadyBeingAdded,
    // TODO fill with errors
    #[error("TODO")]
    Unknown,
}

//endregion

//region cortical area

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

//endregion