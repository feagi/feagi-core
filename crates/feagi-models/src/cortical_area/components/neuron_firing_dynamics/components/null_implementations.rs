//! While implementations are generally model specific, this null one is an exception for any
//! model that doesnt need this context

use core::marker::PhantomData;
use crate::cortical_area::components::neuron_firing_dynamics::components::cortical_data_properties::CorticalDataProperties;
use crate::cortical_area::components::neuron_firing_dynamics::components::cortical_data_work::CorticalDataWork;
use crate::cortical_area::components::neuron_firing_dynamics::components::neuron_data_properties::NeuronDataProperties;
use crate::cortical_area::components::neuron_firing_dynamics::components::neuron_data_work::NeuronDataWork;
use crate::cortical_area::cortical_area_model_quantization::CorticalAreaModelQuantization;
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use crate::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;

/// Used to indicate there is no data for this level
pub struct NullData<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaModelQuantization
{
    _p: PhantomData<(NPUIQ, BEIQ, CAMQ)>
}

impl<NPUIQ, BEIQ, CAMQ> CorticalDataProperties<NPUIQ, BEIQ, CAMQ> for NullData<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaModelQuantization
{
    const HAS_DATA_TO_ALLOCATE: bool = false;
}

impl<NPUIQ, BEIQ, CAMQ> CorticalDataWork<NPUIQ, BEIQ, CAMQ> for NullData<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaModelQuantization
{
    const HAS_DATA_TO_ALLOCATE: bool = false;
}


impl<NPUIQ, BEIQ, CAMQ> NeuronDataProperties<NPUIQ, BEIQ, CAMQ> for NullData<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaModelQuantization
{
    const HAS_DATA_TO_ALLOCATE: bool = false;
}


impl<NPUIQ, BEIQ, CAMQ> NeuronDataWork<NPUIQ, BEIQ, CAMQ> for NullData<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaModelQuantization
{
    const HAS_DATA_TO_ALLOCATE: bool = false;
}


