use crate::create_wrapped_quantized_decimal;
use crate::create_wrapped_quantized_contiguous_linear_vector;
use crate::neurons::neuron_potentials::indexing::{NeuronCount, NeuronLocalIndex};
use crate::values::quantizable::{QuantizedDecimalUnwrappedTrait, QuantizedUnsignedIntegerUnwrappedTrait};

create_wrapped_quantized_decimal!(
    /// The membrane potential of a single neuron (NOT VOXEL)
    pub NeuronPotential
);

create_wrapped_quantized_contiguous_linear_vector!(
    /// Contiguous vector of per-neuron membrane potentials indexed by [`NeuronLocalIndex`].
    pub NeuronPotentialLinearVector,
    NeuronLocalIndex,
    NeuronCount,
    NeuronPotential,
    I: QuantizedUnsignedIntegerUnwrappedTrait,
    Q: QuantizedDecimalUnwrappedTrait,
);
