use crate::cortical_area::parameters::body::dynamics::components::quantization::quantization::CorticalAreaQuantization;
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use crate::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;
use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;
use feagi_data::values::quantizable::DecimalQuantizationLevel;

/// The input surface of a cortical area, this is what incoming mappings will attempt to input into
pub trait DendriteModelTrait<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

/// The input surface of a cortical area, this is what incoming mappings will attempt to input into
pub trait DendriteConfigTrait<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

/// An input surface that allows synapses to map to a neuron here by its local neuron index
pub trait PerNeuronPotentialDendriteModelTrait<NPUIQ, BEIQ, CAMQ>: DendriteModelTrait<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
    const INCOMING_POTENTIAL_QUANTIZATION: DecimalQuantizationLevel = CAMQ::MEMBRANE_POTENTIAL_QUANT_LEVEL;
}

/// An input surface that allows synapses to map to a neuron here by its local neuron index
pub trait PerNeuronPotentialDendriteConfigTrait<NPUIQ, BEIQ, CAMQ>: DendriteConfigTrait<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
    const INCOMING_POTENTIAL_QUANTIZATION: DecimalQuantizationLevel = CAMQ::MEMBRANE_POTENTIAL_QUANT_LEVEL;
}

//region Null

/// A dendrite model that does nothing. No synapse inputs are possible
pub struct NullDendriteModel<NPUIQ, BEIQ, CAMQ>(core::marker::PhantomData<(NPUIQ, BEIQ, CAMQ)>)
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization;

impl<NPUIQ, BEIQ, CAMQ> DendriteModelTrait<NPUIQ, BEIQ, CAMQ> for NullDendriteModel<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

//endregion

//region Per Neuron - Linear

///
pub struct LinearPerNeuronPotentialDendriteModel<NPUIQ, BEIQ, CAMQ>(core::marker::PhantomData<(NPUIQ, BEIQ, CAMQ)>)
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization;

impl<NPUIQ, BEIQ, CAMQ> DendriteModelTrait<NPUIQ, BEIQ, CAMQ> for LinearPerNeuronPotentialDendriteModel<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

impl<NPUIQ, BEIQ, CAMQ> LinearPerNeuronPotentialDendriteModel<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

//endregion

//region Per Neuron - Dimensional

pub struct DimensionalPerNeuronPotentialDendriteModel<NPUIQ, BEIQ, CAMQ>(core::marker::PhantomData<(NPUIQ, BEIQ, CAMQ)>)
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization;

impl<NPUIQ, BEIQ, CAMQ> DendriteModelTrait<NPUIQ, BEIQ, CAMQ> for DimensionalPerNeuronPotentialDendriteModel<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

impl<NPUIQ, BEIQ, CAMQ> DimensionalPerNeuronPotentialDendriteModel<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
    
}

//endregion

//region Area Activity Hashing

// TODO

/// A dendrite model where all incoming area activities are hashed for identification
pub trait AreaActivityHashingDendriteModelTrait<NPUIQ, BEIQ, CAMQ>: DendriteModelTrait<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

//endregion
