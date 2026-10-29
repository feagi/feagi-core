use crate::collections::quant_indexed::quant_indexed::QuantContiguousStore;
use crate::collections::quant_indexed::spatial::SpatialQuantIndexedOwning;
use crate::collections::spatial_helpers::axis_order::AxisOrderIncrementing;
use crate::prelude::{QuantIndexed, QuantizedDecimalUnwrappedTrait, QuantizedUnsignedIntegerUnwrappedTrait, SpatialCoordinate, SpatialDimensions};
use crate::{create_wrapped_quantized_decimal, create_wrapped_quantized_unsigned_integer};
use serde::de::DeserializeOwned;

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

create_wrapped_quantized_unsigned_integer!(
    /// A generic count of neurons
    pub NeuronCount
);

//region Linear Collections

/// Membrane potentials of the neurons in one cortical area, in the backing store `Store`.
pub type CorticalAreaNeuronData<QI, Store> = QuantIndexed<CorticalAreaNeuronLocalIndex<QI>, Store>;

#[cfg(feature = "alloc")]
pub type CorticalAreaNeuronDataVector<QI, QP> = QuantIndexed<CorticalAreaNeuronLocalIndex<QI>, Vec<CorticalAreaNeuronPotential<QP>>>;

pub type CorticalAreaNeuronDataArray<QI, QP, const N: usize> = QuantIndexed<CorticalAreaNeuronLocalIndex<QI>, [CorticalAreaNeuronPotential<QP>; N]>;

pub type CorticalAreaNeuronDataSlice<'a, QI, QP> = QuantIndexed<CorticalAreaNeuronLocalIndex<QI>, &'a [CorticalAreaNeuronPotential<QP>]>;

pub type CorticalAreaNeuronDataSliceMut<'a, QI, QP> = QuantIndexed<CorticalAreaNeuronLocalIndex<QI>, &'a mut [CorticalAreaNeuronPotential<QP>]>;

//endregion

//region Dimensional Collections

/// Defines the 4D coordinates of a dimensional neuron
pub type DimensionalCorticalAreaNeuronCoordinates<QI: QuantizedUnsignedIntegerUnwrappedTrait> = SpatialCoordinate<QI, 4>;

/// Defines the 4D dimensions of a dimensional cortical area
pub type DimensionalCorticalAreaDimensions<QI: QuantizedUnsignedIntegerUnwrappedTrait> = SpatialDimensions<QI, 4>;

/// A collection of dimensional neuron potentials that owns all its inner fields, using default
/// incrementing indexing
pub type DimensionalCorticalAreaPotentials<
    QI: QuantizedUnsignedIntegerUnwrappedTrait,
    QP: QuantizedDecimalUnwrappedTrait,
    S: QuantContiguousStore + DeserializeOwned + 'static,
> = SpatialQuantIndexedOwning<
    QI,
    DimensionalCorticalAreaNeuronCoordinates<QI>,
    DimensionalCorticalAreaDimensions<QI>,
    AxisOrderIncrementing<3>,
    S,
    CorticalAreaNeuronPotential<QP>,
    3,
>;

//endregion
