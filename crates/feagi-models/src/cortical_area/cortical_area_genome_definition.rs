use core::marker::PhantomData;
use feagi_genomic_context::cortical_area::CorticalID;
use crate::cortical_area::cortical_area::CorticalAreaModel;
use crate::cortical_area::parameters::body::dynamics::components::quantization::quantization::CorticalAreaQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub struct CorticalAreaGenomeDefinition<FIQ, CAMQ, CAM>
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization,
    CAM: CorticalAreaModel<FIQ, CAMQ>
{
    pub cortical_areas: Vec<(CorticalID, CAM)>,
    _p: PhantomData<(FIQ, CAMQ)>
}

