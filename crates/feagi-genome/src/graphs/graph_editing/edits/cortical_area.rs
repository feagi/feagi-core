use feagi_basis::neurons::DimensionalCorticalAreaDimensions;
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::basis::cortical_area::cortical_id::{CorticalID, FauxCorticalID};
use crate::basis::genome_object_metadata::GenomeObjectMetadata;

//region Adding

pub type AddCorticalAreaRequest<FIQ: FeagiIndexQuantization> = AddCorticalArea<FIQ, FauxCorticalID>;


pub type AddCorticalAreaAction<FIQ: FeagiIndexQuantization> = AddCorticalArea<FIQ, CorticalID>;

pub enum AddCorticalArea<FIQ: FeagiIndexQuantization, IDType> {
    Interconnect {
        id: IDType,
        metadata: GenomeObjectMetadata,
        dimensions: DimensionalCorticalAreaDimensions<FIQ::NeuronIndexQuant>, // TODO let writer handle
        generator: ()
    }
}







//endregion

//region Editing

// TODO port

//endregion 


//region Deleting

// TODO port

//endregion 
