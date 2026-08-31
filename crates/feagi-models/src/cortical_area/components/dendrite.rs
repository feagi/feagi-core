use crate::cortical_area::parameters::body::dynamics::components::quantization::quantization::CorticalAreaQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use feagi_data::values::quantizable::DecimalQuantizationLevel;

/// The input surface of a cortical area, this is what incoming mappings will attempt to input into
pub trait DendriteModelTrait<FIQ, CAMQ>
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

/// The input surface of a cortical area, this is what incoming mappings will attempt to input into
pub trait DendriteConfigTrait<FIQ, CAMQ>
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

/// An input surface that allows synapses to map to a neuron here by its local neuron index
pub trait PerNeuronPotentialDendriteModelTrait<FIQ, CAMQ>: DendriteModelTrait<FIQ, CAMQ>
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
    const INCOMING_POTENTIAL_QUANTIZATION: DecimalQuantizationLevel = CAMQ::MEMBRANE_POTENTIAL_QUANT_LEVEL;
}

/// An input surface that allows synapses to map to a neuron here by its local neuron index
pub trait PerNeuronPotentialDendriteConfigTrait<FIQ, CAMQ>: DendriteConfigTrait<FIQ, CAMQ>
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
    const INCOMING_POTENTIAL_QUANTIZATION: DecimalQuantizationLevel = CAMQ::MEMBRANE_POTENTIAL_QUANT_LEVEL;
}

//region Null

/// A dendrite model that does nothing. No synapse inputs are possible
pub struct NullDendriteModel<FIQ, CAMQ>(core::marker::PhantomData<(FIQ, CAMQ)>)
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization;

impl<FIQ, CAMQ> DendriteModelTrait<FIQ, CAMQ> for NullDendriteModel<FIQ, CAMQ>
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

//endregion

//region Per Neuron - Linear

///
pub struct LinearPerNeuronPotentialDendriteModel<FIQ, CAMQ>(core::marker::PhantomData<(FIQ, CAMQ)>)
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization;

impl<FIQ, CAMQ> DendriteModelTrait<FIQ, CAMQ> for LinearPerNeuronPotentialDendriteModel<FIQ, CAMQ>
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

impl<FIQ, CAMQ> LinearPerNeuronPotentialDendriteModel<FIQ, CAMQ>
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

//endregion

//region Per Neuron - Dimensional

pub struct DimensionalPerNeuronPotentialDendriteModel<FIQ, CAMQ>(core::marker::PhantomData<(FIQ, CAMQ)>)
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization;

impl<FIQ, CAMQ> DendriteModelTrait<FIQ, CAMQ> for DimensionalPerNeuronPotentialDendriteModel<FIQ, CAMQ>
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

impl<FIQ, CAMQ> DimensionalPerNeuronPotentialDendriteModel<FIQ, CAMQ>
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
    
}

//endregion

//region Area Activity Hashing

// TODO

/// A dendrite model where all incoming area activities are hashed for identification
pub trait AreaActivityHashingDendriteModelTrait<FIQ, CAMQ>: DendriteModelTrait<FIQ, CAMQ>
where
    FIQ: FeagiIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

//endregion
