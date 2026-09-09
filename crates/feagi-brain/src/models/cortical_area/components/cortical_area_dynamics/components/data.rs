use std::marker::PhantomData;
use crate::cortical_area::parameters::body::dynamics::components::quantization::quantization::CorticalAreaQuantization;

/// Any cortical level data that should be configurable by genome developers
pub trait CorticalDataProperties<CAMQ>: Clone + core::fmt::Debug + Default
where
    CAMQ: CorticalAreaQuantization,
{
    /// Denotes that there is data that needs to be allocated. 
    /// Only the null implementation has this false
    const HAS_DATA_TO_ALLOCATE: bool = true;
    
    // extend per neuron firing implementation
}

/// Any cortical level data that is internal, not to be exposed to genome developers
pub trait CorticalDataInternal<CAMQ>: Clone + core::fmt::Debug + Default
where
    CAMQ: CorticalAreaQuantization,
{
    /// Denotes that there is data that needs to be allocated.
    /// Only the null implementation has this false
    const HAS_DATA_TO_ALLOCATE: bool = true;

    // extend per neuron firing implementation
}

/// Any cortical level data that is visible by mappings
pub trait CorticalDataShared<CAMQ>: Clone + core::fmt::Debug + Default
where
    CAMQ: CorticalAreaQuantization,
{
    /// Denotes that there is data that needs to be allocated.
    /// Only the null implementation has this false
    const HAS_DATA_TO_ALLOCATE: bool = true;

    // extend per neuron firing implementation
}

/// Any per neuron level data that should be configurable by genome developers
pub trait NeuronDataProperties<CAMQ>: Clone + core::fmt::Debug + Default
where
    CAMQ: CorticalAreaQuantization,
{
    /// Denotes that there is data that needs to be allocated. 
    /// Only the null implementation has this false
    const HAS_DATA_TO_ALLOCATE: bool = true;

    // extend per neuron firing implementation
}

/// Any per neuron level data that is internal, not to be exposed to genome developers
pub trait NeuronDataInternal<CAMQ>: Clone + core::fmt::Debug + Default
where
    CAMQ: CorticalAreaQuantization,
{
    /// Denotes that there is data that needs to be allocated. 
    /// Only the null implementation has this false
    const HAS_DATA_TO_ALLOCATE: bool = true;

    // extend per neuron firing implementation
}

//region Null Implementation

/// While implementations are generally model specific, the null is an exception for any
/// model that doesnt need a given context
#[derive(Clone, Debug)]
pub struct NullData<CAMQ>
where
    CAMQ: CorticalAreaQuantization
{
    _p: PhantomData<(CAMQ)>
}

impl<CAMQ> Default for NullData<CAMQ>
where
    CAMQ: CorticalAreaQuantization
{
    fn default() -> Self {
        NullData {_p: PhantomData}
    }
}

impl<CAMQ> CorticalDataProperties<CAMQ> for NullData<CAMQ>
where
    CAMQ: CorticalAreaQuantization
{
    const HAS_DATA_TO_ALLOCATE: bool = false;
}

impl<CAMQ> CorticalDataInternal<CAMQ> for NullData<CAMQ>
where
    CAMQ: CorticalAreaQuantization
{
    const HAS_DATA_TO_ALLOCATE: bool = false;
}

impl<CAMQ> CorticalDataShared<CAMQ> for NullData<CAMQ>
where
    CAMQ: CorticalAreaQuantization
{
    const HAS_DATA_TO_ALLOCATE: bool = false;
}

impl<CAMQ> NeuronDataProperties<CAMQ> for NullData<CAMQ>
where
    CAMQ: CorticalAreaQuantization
{
    const HAS_DATA_TO_ALLOCATE: bool = false;
}

impl<CAMQ> NeuronDataInternal<CAMQ> for NullData<CAMQ>
where
    CAMQ: CorticalAreaQuantization
{
    const HAS_DATA_TO_ALLOCATE: bool = false;
}

//endregion