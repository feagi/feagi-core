

//region Linear

use feagi_basis_neuron::collections::linear::LinearCorticalNeuronActivationVector;
use feagi_basis_quantization::prelude::FeagiIndexQuantization;
use crate::feagi_serializable::FeagiDataSerializableQuantized;


impl<'de, FIQ: FeagiIndexQuantization>  FeagiDataSerializableQuantized<'de, FIQ> for LinearCorticalNeuronActivationVector<FIQ::NeuronIndexQuant> {}






//endregion
















