//! Traits for synapses describing connections going from dimensional cortical areas to dimensional cortical areas


// TODO some things should be moved to a higher level trait as we understand other synapse types more

use feagi_structures::base_quantizable::{QuantizablePercentType, QuantizableUIntType, QuantizableValueType};
use crate::neuron::npu_neuron_type::NPUNeuronType;
use crate::quantizables::{NPUNeuronIndex, NPUSynapseIndex, PSPMultiplier, SynapticWeight};
use crate::synapse::base_traits::{BaseSynapseAllocStorageTrait, BaseSynapseStaticStorageTrait};
use crate::synapse::feagi_npu_synapse_error::FeagiNPUSynapseError;

pub trait Dim2DimSynapseStaticStorageTrait<SynapseIndexQuant, NeuronIndexQuant, PercentageQuant, PotentialQuant >:
BaseSynapseStaticStorageTrait<SynapseIndexQuant, NeuronIndexQuant, PercentageQuant, PotentialQuant >
where
    SynapseIndexQuant: QuantizableUIntType,
    NeuronIndexQuant: QuantizableUIntType,
    PercentageQuant: QuantizablePercentType,
    PotentialQuant: QuantizableValueType,
{



    // TODO how should we iterate this?

}

pub trait Dim2DimSynapseAllocStorageTrait<SynapseIndexQuant, NeuronIndexQuant, PercentageQuant, PotentialQuant >:
BaseSynapseAllocStorageTrait<SynapseIndexQuant, NeuronIndexQuant, PercentageQuant, PotentialQuant >
where
    SynapseIndexQuant: QuantizableUIntType,
    NeuronIndexQuant: QuantizableUIntType,
    PercentageQuant: QuantizablePercentType,
    PotentialQuant: QuantizableValueType,
{
    fn create_synapse_connections(&mut self, source_neurons_indexes: &[NPUNeuronIndex<NeuronIndexQuant>],
                                  source_neurons_type: NPUNeuronType,
                                  destination_neuron_indexes: &[NPUNeuronIndex<NeuronIndexQuant>],
                                  destination_neurons_type: NPUNeuronType) -> Result<&[NPUSynapseIndex<SynapseIndexQuant>], FeagiNPUSynapseError>;

    fn remove_synapse_connections_by_synapse_index(&mut self, synapse_indexes: &[NPUSynapseIndex<SynapseIndexQuant>]) -> Result<(), FeagiNPUSynapseError>;

    fn remove_synapse_connections_by_source_neuron_index(&mut self, source_neurons_type: NPUNeuronType, source_neurons_indexes: &[NPUNeuronIndex<NeuronIndexQuant>]) -> Result<(), FeagiNPUSynapseError>;

    fn remove_synapse_connections_by_destination_neuron_index(&mut self, destination_destination_type: NPUNeuronType, destination_neuron_indexes: &[NPUNeuronIndex<NeuronIndexQuant>]) -> Result<(), FeagiNPUSynapseError>;






    // we may need to support some sort of function to write synapse data

    // TODO how should we iterate this?

}