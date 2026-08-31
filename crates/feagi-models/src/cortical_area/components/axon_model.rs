use feagi_data::values::quantizable::DecimalQuantizationLevel;
use crate::cortical_area::parameters::body::dynamics::components::quantization::quantization::{CorticalAreaQuantization};
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// The direct output of a cortical area, this is what mappings will extend out of
pub trait AxonModelTrait<FIQ, CAMQ>
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
    
}

/// The direct output of a cortical area, this is what mappings will extend out of
pub trait AxonConfigTrait<FIQ, CAMQ>
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

/// An output surface that allows synapses to map to a neuron from here by its local neuron index
pub trait PerNeuronPotentialAxonModelTrait<FIQ, CAMQ>: AxonModelTrait<FIQ, CAMQ>
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
    const INCOMING_POTENTIAL_QUANTIZATION: DecimalQuantizationLevel = CAMQ::MEMBRANE_POTENTIAL_QUANT_LEVEL;
}

/// An output surface that allows synapses to map to a neuron from here by its local neuron index
pub trait PerNeuronPotentialAxonConfigTrait<FIQ, CAMQ>: AxonConfigTrait<FIQ, CAMQ>
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
    const INCOMING_POTENTIAL_QUANTIZATION: DecimalQuantizationLevel = CAMQ::MEMBRANE_POTENTIAL_QUANT_LEVEL;
}

//region Null

/// An axon model that does nothing. No synapse outputs are possible
pub struct NullAxonModel<FIQ, CAMQ>(core::marker::PhantomData<(FIQ, CAMQ)>)
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization;

impl<FIQ, CAMQ> AxonModelTrait<FIQ, CAMQ> for NullAxonModel<FIQ, CAMQ>
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

//endregion

//region Per Neuron - Linear

///
pub struct LinearPerNeuronPotentialAxonModel<FIQ, CAMQ>(core::marker::PhantomData<(FIQ, CAMQ)>)
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization;

impl<FIQ, CAMQ> AxonModelTrait<FIQ, CAMQ> for LinearPerNeuronPotentialAxonModel<FIQ, CAMQ>
where
    FIQ: FeagiIndexQuantization,

    CAMQ: CorticalAreaQuantization,
{
}

impl<FIQ, CAMQ> LinearPerNeuronPotentialAxonModel<FIQ, CAMQ>
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

//endregion

//region Per Neuron - Dimensional

pub struct DimensionalPerNeuronPotentialAxonModel<FIQ, CAMQ>(core::marker::PhantomData<(FIQ, CAMQ)>)
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization;

impl<FIQ, CAMQ> AxonModelTrait<FIQ, CAMQ> for DimensionalPerNeuronPotentialAxonModel<FIQ, CAMQ>
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

impl<FIQ, CAMQ> DimensionalPerNeuronPotentialAxonModel<FIQ, CAMQ>
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
    
}

//endregion



