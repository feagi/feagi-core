use feagi_data::values::quantizable::DecimalQuantizationLevel;
use crate::cortical_area::parameters::body::dynamics::components::quantization::quantization::{CorticalAreaQuantization};
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;
use crate::quantization_levels::neuron_processing_unit_index_quantization::NeuronProcessingUnitIndexQuantization;

/// The direct output of a cortical area, this is what mappings will extend out of
pub trait AxonModelTrait<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
    
}

/// The direct output of a cortical area, this is what mappings will extend out of
pub trait AxonConfigTrait<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

/// An output surface that allows synapses to map to a neuron from here by its local neuron index
pub trait PerNeuronPotentialAxonModelTrait<NPUIQ, BEIQ, CAMQ>: AxonModelTrait<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
    const INCOMING_POTENTIAL_QUANTIZATION: DecimalQuantizationLevel = CAMQ::MEMBRANE_POTENTIAL_QUANT_LEVEL;
}

/// An output surface that allows synapses to map to a neuron from here by its local neuron index
pub trait PerNeuronPotentialAxonConfigTrait<NPUIQ, BEIQ, CAMQ>: AxonConfigTrait<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
    const INCOMING_POTENTIAL_QUANTIZATION: DecimalQuantizationLevel = CAMQ::MEMBRANE_POTENTIAL_QUANT_LEVEL;
}

//region Null

/// An axon model that does nothing. No synapse outputs are possible
pub struct NullAxonModel<NPUIQ, BEIQ, CAMQ>(core::marker::PhantomData<(NPUIQ, BEIQ, CAMQ)>)
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization;

impl<NPUIQ, BEIQ, CAMQ> AxonModelTrait<NPUIQ, BEIQ, CAMQ> for NullAxonModel<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

//endregion

//region Per Neuron - Linear

///
pub struct LinearPerNeuronPotentialAxonModel<NPUIQ, BEIQ, CAMQ>(core::marker::PhantomData<(NPUIQ, BEIQ, CAMQ)>)
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization;

impl<NPUIQ, BEIQ, CAMQ> AxonModelTrait<NPUIQ, BEIQ, CAMQ> for LinearPerNeuronPotentialAxonModel<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

impl<NPUIQ, BEIQ, CAMQ> LinearPerNeuronPotentialAxonModel<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

//endregion

//region Per Neuron - Dimensional

pub struct DimensionalPerNeuronPotentialAxonModel<NPUIQ, BEIQ, CAMQ>(core::marker::PhantomData<(NPUIQ, BEIQ, CAMQ)>)
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization;

impl<NPUIQ, BEIQ, CAMQ> AxonModelTrait<NPUIQ, BEIQ, CAMQ> for DimensionalPerNeuronPotentialAxonModel<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
}

impl<NPUIQ, BEIQ, CAMQ> DimensionalPerNeuronPotentialAxonModel<NPUIQ, BEIQ, CAMQ>
where
    NPUIQ: NeuronProcessingUnitIndexQuantization,
    BEIQ: BurstEngineIndexQuantization,
    CAMQ: CorticalAreaQuantization,
{
    
}

//endregion



