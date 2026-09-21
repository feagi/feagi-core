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

#[cfg(feature = "alloc")]
/// A vector of neuron potentials for a given cortical area (any cortical area,
/// dimensional or otherwise)
pub type CorticalAreaNeuronPotentialVector<
    IQuant: QuantizedUnsignedIntegerUnwrappedTrait,
    PQuant: QuantizedDecimalUnwrappedTrait
> = ParDataVector<
    CorticalAreaNeuronLocalIndex<IQuant>,
    CorticalAreaNeuronPotential<PQuant>
>;

/// A slice of neuron potentials for a given cortical area (any cortical area,
/// dimensional or otherwise)
pub type CorticalAreaNeuronPotentialSlice<
    'a,
    IQuant: QuantizedUnsignedIntegerUnwrappedTrait,
    PQuant: QuantizedDecimalUnwrappedTrait
> = feagi_basis_collections::generic_data::par_data::ParDataSlice<
    'a,
    CorticalAreaNeuronLocalIndex<IQuant>,
    CorticalAreaNeuronPotential<PQuant>
>;

/// A mut slice of neuron potentials for a given cortical area (any cortical area,
/// dimensional or otherwise)
pub type CorticalAreaNeuronPotentialSliceMut<
    'a,
    IQuant: QuantizedUnsignedIntegerUnwrappedTrait,
    PQuant: QuantizedDecimalUnwrappedTrait
> = feagi_basis_collections::generic_data::par_data::ParDataSliceMut<
    'a,
    CorticalAreaNeuronLocalIndex<IQuant>,
    CorticalAreaNeuronPotential<PQuant>
>;

/// An array of neuron potentials for a given cortical area (any cortical area,
/// dimensional or otherwise)
pub type CorticalAreaNeuronPotentialArray<
    IQuant: QuantizedUnsignedIntegerUnwrappedTrait,
    PQuant: QuantizedDecimalUnwrappedTrait,
    const N: usize
> = feagi_basis_collections::generic_data::par_data::ParDataArray<
    CorticalAreaNeuronLocalIndex<IQuant>,
    CorticalAreaNeuronPotential<PQuant>,
    N
>;
