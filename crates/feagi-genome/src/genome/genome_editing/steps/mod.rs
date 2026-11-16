use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::genome::genome_editing::GenomeEditSequenceContext;

pub mod brain_region;
pub mod cortical_area;

pub(crate) enum GenomeEditStepSequenceStep<FIQ: FeagiIndexQuantization, GESC: GenomeEditSequenceContext> {
    BrainRegion(GESC::BrainRegionStepEnum),
    CorticalArea(GESC::CorticalAreaStepEnum<FIQ>),
    //ConnectivityRule
    //CorticalMapping
    
}