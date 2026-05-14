use crate::base_feagi_types::quantizable_types::{QuantizableUIntType, QuantizableValueType};
use crate::neuron::model_specifications::base_dimensional_specifications::base_dimensional_neuron_specifications::BaseDimensionalNeuronCollectionSharedTrait;
use crate::neuron::model_specifications::base_propagating_neuron_specifications::base_propagating_neuron_specifications::BasePropagatingNeuronsCollectionSharedTrait;
use crate::neuron::model_specifications::base_propagating_neuron_specifications::HasNeuronFired;
use crate::neuron::model_specifications::base_specifications::{LinearNeuronIndexCount, NeuronMembranePotential};
use crate::quantization_level::CorticalAreaNeuronQuantization;



pub trait DimensionalPropagatingNeuronsCollectionSharedTrait<CANQ: CorticalAreaNeuronQuantization>:
BaseDimensionalNeuronCollectionSharedTrait<CANQ> +
BasePropagatingNeuronsCollectionSharedTrait<CANQ>{

    fn fire_dimensional_neuron<GlobalBurstIndex: QuantizableUIntType, SynapsePotentialQuant: QuantizableValueType>(&mut self,
                                                                                                                   firing_neuron_index: LinearNeuronIndexCount<CANQ::NeuronIndexVoxelCountQuant>,
                                                                                                                   input_sum_potential: NeuronMembranePotential<SynapsePotentialQuant>,
                                                                                                                   current_burst_index: GlobalBurstIndex, // TODO? Do we care if its not wrapped?
                                                                                                                   cortical_configuration: &Self::CorticalConfigurationType)
                                                                                                                   -> HasNeuronFired<CANQ::NeuronValueQuant>;
}



