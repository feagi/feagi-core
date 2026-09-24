use serde::de::DeserializeOwned;
use feagi_basis_collections::generic_data::par_data::{ParData, ParDataStore};
use feagi_basis_collections::generic_data::spatial::SpatialParDataOwning;
use feagi_basis_collections::prelude::*;
use feagi_basis_collections::spatial_indexing_structs::axis_order::AxisOrderIncrementing;
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

//region Linear Collections

/// Membrane potentials of the neurons in one cortical area, in the backing store `Store`.
pub type CorticalAreaNeuronData<QI, Store> =
ParData<CorticalAreaNeuronLocalIndex<QI>, Store>;

#[cfg(feature = "alloc")]
pub type CorticalAreaNeuronDataVector<QI, QP> =
ParData<CorticalAreaNeuronLocalIndex<QI>, Vec<CorticalAreaNeuronPotential<QP>>>;

pub type CorticalAreaNeuronDataArray<QI, QP, const N: usize> =
ParData<CorticalAreaNeuronLocalIndex<QI>, [CorticalAreaNeuronPotential<QP>; N]>;

pub type CorticalAreaNeuronDataSlice<'a, QI, QP> =
ParData<CorticalAreaNeuronLocalIndex<QI>, &'a [CorticalAreaNeuronPotential<QP>]>;

pub type CorticalAreaNeuronDataSliceMut<'a, QI, QP> =
ParData<CorticalAreaNeuronLocalIndex<QI>, &'a mut [CorticalAreaNeuronPotential<QP>]>;

//endregion

//region Dimensional Collections

// TODO further restrict order type to neuron specific implementations

create_wrapped_quantized_unsigned_integer!(
    /// Index of a neuron's position along the x y z or d
    pub DimensionalCorticalAreaNeuronAxisPosition
);

/// Defines the 4D coordinates of a dimensional neuron
pub type DimensionalCorticalAreaNeuronCoordinates<QI: QuantizedUnsignedIntegerUnwrappedTrait> =
SpatialCoordinate<DimensionalCorticalAreaNeuronAxisPosition<QI>, 4>;

/// Defines the 4D dimensions of a dimensional cortical area
pub type DimensionalCorticalAreaDimensions<QI: QuantizedUnsignedIntegerUnwrappedTrait> =
SpatialDimensions<DimensionalCorticalAreaNeuronAxisPosition<QI>, 4>;

/// A collection of dimensional neuron potentials that owns all its inner fields, using default
/// incrementing indexing
pub type DimensionalCorticalAreaPotentials<
    QI: QuantizedUnsignedIntegerUnwrappedTrait,
    QP: QuantizedDecimalUnwrappedTrait,
    S: ParDataStore + DeserializeOwned + 'static
> = SpatialParDataOwning<
    DimensionalCorticalAreaNeuronAxisPosition<QI>,
    DimensionalCorticalAreaNeuronCoordinates<QI>,
    DimensionalCorticalAreaDimensions<QI>,
    AxisOrderIncrementing<3>,
    S,
    CorticalAreaNeuronPotential<QP>,
    3
>;

//endregion

