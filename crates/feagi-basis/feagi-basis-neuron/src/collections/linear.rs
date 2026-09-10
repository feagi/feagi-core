use feagi_basis_quantization::generic_collections::bitpacked::linear::{BitPackedArraySizeAware, BitPackedSliceMutSizeAware, BitPackedSliceSizeAware, BitPackedVectorSizeAware};
use feagi_basis_quantization::generic_collections::generic_par_data::linear::{ParDataArray, ParDataSlice, ParDataSliceMut, ParDataVector};
use feagi_basis_quantization::prelude::QuantizedDecimalTrait;
use feagi_basis_quantization::values::quantizable::QuantizedUnsignedIntegerTrait;
use crate::wrapped_types::{CorticalNeuronLocalIndex, CorticalNeuronPotential};
// TODO size aware vs unaware?

// Neuron Linear Activations (Size Aware)

pub type LinearCorticalNeuronActivationVector<QI: QuantizedUnsignedIntegerTrait> =
BitPackedVectorSizeAware<CorticalNeuronLocalIndex<QI>>;

pub type LinearCorticalNeuronActivationSlice<'a, QI: QuantizedUnsignedIntegerTrait> =
BitPackedSliceSizeAware<'a, CorticalNeuronLocalIndex<QI>>;

pub type LinearCorticalNeuronActivationSliceMut<'a, QI: QuantizedUnsignedIntegerTrait> =
BitPackedSliceMutSizeAware<'a, CorticalNeuronLocalIndex<QI>>;

pub type LinearCorticalNeuronActivationArray<QI: QuantizedUnsignedIntegerTrait, const N: usize> =
BitPackedArraySizeAware<CorticalNeuronLocalIndex<QI>, N>;


// Neuron Linear Potentials

pub type LinearCorticalNeuronPotentialVector<QI: QuantizedUnsignedIntegerTrait, Q: QuantizedDecimalTrait> =
    ParDataVector<CorticalNeuronLocalIndex<QI>, CorticalNeuronPotential<Q>>;

pub type LinearCorticalNeuronPotentialSlice<'a, QI: QuantizedUnsignedIntegerTrait, Q: QuantizedDecimalTrait> =
    ParDataSlice<'a, CorticalNeuronLocalIndex<QI>, CorticalNeuronPotential<Q>>;

pub type LinearCorticalNeuronPotentialSliceMut<'a, QI: QuantizedUnsignedIntegerTrait, Q: QuantizedDecimalTrait> =
    ParDataSliceMut<'a, CorticalNeuronLocalIndex<QI>, CorticalNeuronPotential<Q>>;

pub type LinearCorticalNeuronPotentialArray<QI: QuantizedUnsignedIntegerTrait, Q: QuantizedDecimalTrait, const N: usize> =
    ParDataArray<CorticalNeuronLocalIndex<QI>, CorticalNeuronPotential<Q>, N>;

