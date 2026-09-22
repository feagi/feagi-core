use serde::de::DeserializeOwned;
use feagi_basis_collections::generic_data::par_data::ParDataStore;
use feagi_basis_collections::generic_data::spatial::SpatialParDataOwning;
use feagi_basis_collections::prelude::*;
use feagi_basis_quantization::prelude::*;

create_wrapped_quantized_decimal!(
    /// Represents the Membrane Potential of the neuron(s) in a voxel. Most of the time, each
    /// voxel contains a single neuron, but in cases where there are more, they are averaged to
    /// make this
    pub CorticalAreaVoxelPotential
);

create_wrapped_quantized_unsigned_integer!(
    /// Represents the index of a voxel in a collection using a single uint value that represents
    /// the overall index incrementing from X, Y and Z
    pub CorticalAreaVoxelLinearIndex
);

create_wrapped_quantized_unsigned_integer!(
    /// The number of voxels within a dimensional cortical area
    pub CorticalAreaVoxelCount
);

create_wrapped_quantized_unsigned_integer!(
    /// Index of a voxel along one of the XYZ directions within a cortical area
    pub CorticalAreaVoxelCoordinateAxisIndex
);


/// Defines the voxel dimensions of a voxel cortical area
pub type CorticalAreaVoxelCoordinates<QI: QuantizedUnsignedIntegerUnwrappedTrait> =
SpatialCoordinate<CorticalAreaVoxelCoordinateAxisIndex<QI>, 3>;

/// Defines the voxel dimensions of a voxel cortical area
pub type CorticalAreaVoxelDimensions<QI: QuantizedUnsignedIntegerUnwrappedTrait> =
SpatialDimensions<CorticalAreaVoxelCoordinateAxisIndex<QI>, 3>;


/// Represents a single voxel and its coordinate within a dimensional cortical area
pub struct CorticalAreaVoxel<QI: QuantizedUnsignedIntegerUnwrappedTrait, QP: QuantizedDecimalUnwrappedTrait> {
    pub coordinate: CorticalAreaVoxelCoordinates<QI>,
    pub potential: CorticalAreaVoxelPotential<QP>
}


/// A collection of Cortical Area Voxels that owns all its inner fields
pub type CorticalAreaVoxels<
    QI: QuantizedUnsignedIntegerUnwrappedTrait,
    QP: QuantizedDecimalUnwrappedTrait,
    S: ParDataStore + DeserializeOwned + 'static
> = SpatialParDataOwning<
    CorticalAreaVoxelLinearIndex<QI>,
    CorticalAreaVoxelCoordinates<QI>,
    CorticalAreaVoxelDimensions<QI>,
    S,
    CorticalAreaVoxelPotential<QP>,
    3
>;