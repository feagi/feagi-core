use crate::basis::specifications::brain_regions::BrainRegionID;
use crate::cortical_area::{CorticalAreaType, CorticalIdentifier, NeuronLayoutType};
use crate::genome::genome_object::GenomeObjectMetadata;
use crate::genome::genome_object::GenomeObjectPositioning;
use feagi_connectome::cortical_area::{CorticalAreaEngineIndex, NeuronModel, NeuronModelUniversalCorticalProperties};
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use std::collections::HashSet;
use feagi_connectome::cortical_area::neuron_layout::NeuronLayout;

pub struct CorticalAreaContainerDesigner<FIQ, CT>
where
    FIQ: FeagiIndexQuantization,
    CT: CorticalAreaType
{
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
    pub parent_id: BrainRegionID,
    pub incoming_mappings: HashSet<CorticalIdentifier>,
    pub outgoing_mappings: HashSet<CorticalIdentifier>,
    pub layout_context: <<CT::NeuronLayout as NeuronLayoutType>::Layout<FIQ> as NeuronLayout<FIQ>>::CorticalLevelContext,
    pub neuron_model_data: // TODO Cortical Type Specific enum wrapping an enum (not cortical type specific) of NeuronModelDataDesigner at different quants
}

impl<FIQ, CT> CorticalAreaContainerDesigner<FIQ, CT>
where
    FIQ: FeagiIndexQuantization,
    CT: CorticalAreaType
{


    
}


pub struct NeuronModelDataDesigner<FIQ: FeagiIndexQuantization, NM: NeuronModel> {
    pub cortical_universal: NeuronModelUniversalCorticalProperties<NM>,
    pub cortical_properties: NM::CorticalDataProperties,
    pub cortical_data_variance: NM::CorticalDataVariance,
    pub cortical_npu_index: CorticalAreaEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
    pub burst_engine_index: u8,
    // TODO NM specific visualization context
    // TODO generator used as an enum
}





/*

pub struct CorticalAreaContainerTypedStorage<FIQ: FeagiIndexQuantization, NM: NeuronModel>
{
    pub metadata: GenomeObjectMetadata,
    pub positioning: GenomeObjectPositioning,
    pub parent_id: BrainRegionID,
    pub cortical_area_model: (),
}

 */



