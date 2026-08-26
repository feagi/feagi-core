use std::marker::PhantomData;
use feagi_data::values::quantizable::PercentageUnsigned;
use crate::cortical_area::cortical_area_model_quantization::CorticalAreaModelQuantization;
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use crate::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;

/// Any cortical level data that should be configurable by genome developers
pub trait CorticalDataProperties<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaModelQuantization,
{
    /// Denotes that there is data that needs to be allocated. 
    /// Only the null implementation has this false
    const HAS_DATA_TO_ALLOCATE: bool = true;
    
    // extend per neuron firing implementation
}

/// Any cortical level data that is internal, not to be exposed to genome developers
pub trait CorticalDataInternal<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaModelQuantization,
{
    /// Denotes that there is data that needs to be allocated.
    /// Only the null implementation has this false
    const HAS_DATA_TO_ALLOCATE: bool = true;

    // extend per neuron firing implementation
}

/// Any cortical level data that is shared (visible by mappings)
pub trait CorticalDataShared<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaModelQuantization,
{
    /// Denotes that there is data that needs to be allocated.
    /// Only the null implementation has this false
    const HAS_DATA_TO_ALLOCATE: bool = true;

    // extend per neuron firing implementation
}

/// Any per neuron level data that should be configurable by genome developers
pub trait NeuronDataProperties<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaModelQuantization,
{
    /// Denotes that there is data that needs to be allocated. 
    /// Only the null implementation has this false
    const HAS_DATA_TO_ALLOCATE: bool = true;

    // extend per neuron firing implementation
}

/// Any per neuron level data that is internal, not to be exposed to genome developers
pub trait NeuronDataInternal<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaModelQuantization,
{
    /// Denotes that there is data that needs to be allocated. 
    /// Only the null implementation has this false
    const HAS_DATA_TO_ALLOCATE: bool = true;

    // extend per neuron firing implementation
}

//region Null Implementation

/// While implementations are generally model specific, the null is an exception for any
/// model that doesnt need a given context
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

impl<NPUIQ, BEIQ, CAMQ> CorticalDataInternal<NPUIQ, BEIQ, CAMQ> for NullData<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaModelQuantization
{
    const HAS_DATA_TO_ALLOCATE: bool = false;
}

impl<NPUIQ, BEIQ, CAMQ> CorticalDataShared<NPUIQ, BEIQ, CAMQ> for NullData<NPUIQ, BEIQ, CAMQ>
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

impl<NPUIQ, BEIQ, CAMQ> NeuronDataInternal<NPUIQ, BEIQ, CAMQ> for NullData<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaModelQuantization
{
    const HAS_DATA_TO_ALLOCATE: bool = false;
}

//endregion