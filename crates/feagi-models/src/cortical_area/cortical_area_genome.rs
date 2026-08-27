use core::marker::PhantomData;
use feagi_genomic_context::cortical_area::CorticalID;
use crate::cortical_area::cortical_area::CorticalAreaModel;
use crate::cortical_area::parameters::body::dynamics::components::quantization::quantization::CorticalAreaQuantization;
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use crate::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;

pub struct CorticalAreaGenomeData<NPUIQ, BEIQ, CAMQ, CAM>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
    CAM: CorticalAreaModel<NPUIQ, BEIQ, CAMQ>
{
    pub cortical_areas: Vec<(CorticalID, CAM)>,
    _p: PhantomData<(NPUIQ, BEIQ, CAMQ)>
}

