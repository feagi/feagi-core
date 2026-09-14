//region Linear

use feagi_basis_neuron::collections::linear::LinearCorticalNeuronActivationVector;
use crate::feagi_serializable::FeagiDataSerializableQuantized;
use feagi_basis_quantization::prelude::FeagiIndexQuantization;

#[cfg(feature = "alloc")]
impl<'de, FIQ: FeagiIndexQuantization> FeagiDataSerializableQuantized<'de, FIQ> for
LinearCorticalNeuronActivationVector<FIQ::NeuronIndexQuant> {}

//endregion
