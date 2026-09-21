//! Neuron Properties of cortical areas in general (not assuming dimensionality or otherwise)

use feagi_basis_collections::generic_data::par_data::{GenericParData};
use feagi_basis_quantization::prelude::*;

create_wrapped_quantized_decimal!(
    /// Represents the Membrane Potential of a single neuron
    pub CorticalAreaNeuronPotential
);

create_wrapped_quantized_unsigned_integer!(
    /// Index of a neuron relative to its parent cortical area
    pub CorticalAreaNeuronLocalIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// The number of neurons (not voxels) within a cortical area
    pub CorticalAreaNeuronCount
);

