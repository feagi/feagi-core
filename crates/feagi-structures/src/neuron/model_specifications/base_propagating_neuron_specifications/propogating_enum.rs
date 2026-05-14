use crate::base_feagi_types::quantizable_types::QuantizableValueType;
use crate::neuron::model_specifications::base_specifications::NeuronMembranePotential;

pub enum HasNeuronFired<NPUPotential: QuantizableValueType> {
    NoFire,
    Firing(NeuronMembranePotential<NPUPotential>)
}