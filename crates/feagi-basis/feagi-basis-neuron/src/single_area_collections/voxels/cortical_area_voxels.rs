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

/*
pub struct CorticalAreaVoxels<
    QI: QuantizedUnsignedIntegerUnwrappedTrait,
    QP: QuantizedDecimalUnwrappedTrait
>
{
    dimensions: (),
    stride: (),
    axis_order: (),
    data: (),
    
}
 */















/*
pub enum CorticalAreaVoxels<
    QI: QuantizedUnsignedIntegerUnwrappedTrait,
    QP: QuantizedDecimalUnwrappedTrait> 
{
    Interconnect(),
    
}


 */

