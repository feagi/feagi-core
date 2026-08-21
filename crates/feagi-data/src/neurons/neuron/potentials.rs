use crate::{create_quantized_contiguous_linear_collections, create_wrapped_quantized_decimal};

create_wrapped_quantized_decimal!(
    /// The membrane potential of a single neuron (NOT VOXEL)
    pub NeuronPotential
);

create_quantized_contiguous_linear_collections!(pub, NeuronPotential, NeuronLocalIndex, NeuronCount, NeuronPotential);