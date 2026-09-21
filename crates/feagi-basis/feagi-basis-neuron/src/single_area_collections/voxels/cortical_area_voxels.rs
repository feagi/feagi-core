
use feagi_basis_collections::prelude::{SpatialCoordinate, SpatialDimensions};
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

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct CorticalAreaVoxelDimensions<
    QI: QuantizedUnsignedIntegerUnwrappedTrait,
> {
    dimensions: SpatialDimensions<CorticalAreaVoxelLinearIndex<QI>, 3>
}

impl<QI: QuantizedUnsignedIntegerUnwrappedTrait> CorticalAreaVoxelDimensions<QI> {

    pub fn new(
        x: CorticalAreaVoxelLinearIndex<QI>,
        y: CorticalAreaVoxelLinearIndex<QI>,
        z: CorticalAreaVoxelLinearIndex<QI>,
    ) -> Result<Self, ()> {
        let dimensions = SpatialDimensions::new_dimensions([x, y, z])
            .map_err(|e| ())?; // TODO error handling
        Ok(CorticalAreaVoxelDimensions { dimensions })
    }

    pub fn new_from_usize(x: usize, y: usize, z: usize) -> Result<Self, ()> {
        // TODO error handling
        let x = CorticalAreaVoxelLinearIndex::<QI>::quant_try_from_usize(x).unwrap();
        let y = CorticalAreaVoxelLinearIndex::<QI>::quant_try_from_usize(y).unwrap();
        let z = CorticalAreaVoxelLinearIndex::<QI>::quant_try_from_usize(z).unwrap();
        Self::new(x, y, z)
    }

    pub fn get_x(&self) -> &CorticalAreaVoxelLinearIndex<QI> {
        &self.dimensions.as_slice()[0]
    }

    pub fn get_y(&self) -> &CorticalAreaVoxelLinearIndex<QI> {
        &self.dimensions.as_slice()[1]
    }

    pub fn get_z(&self) -> &CorticalAreaVoxelLinearIndex<QI> {
        &self.dimensions.as_slice()[2]
    }
}


#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct CorticalAreaVoxelCoordinates<
    QI: QuantizedUnsignedIntegerUnwrappedTrait,
> {
    coordinates: SpatialCoordinate<CorticalAreaVoxelLinearIndex<QI>, 3>
}

impl<QI: QuantizedUnsignedIntegerUnwrappedTrait> CorticalAreaVoxelCoordinates<QI> {

    pub fn new(
        x: CorticalAreaVoxelLinearIndex<QI>,
        y: CorticalAreaVoxelLinearIndex<QI>,
        z: CorticalAreaVoxelLinearIndex<QI>,
    ) -> Self {
        let coordinates = SpatialCoordinate::new_coordinate([x, y, z])
        CorticalAreaVoxelCoordinates { coordinates }
    }

    pub fn new_from_usize(x: usize, y: usize, z: usize) -> Result<Self, ()> {
        // TODO error handling
        let x = CorticalAreaVoxelLinearIndex::<QI>::quant_try_from_usize(x).unwrap();
        let y = CorticalAreaVoxelLinearIndex::<QI>::quant_try_from_usize(y).unwrap();
        let z = CorticalAreaVoxelLinearIndex::<QI>::quant_try_from_usize(z).unwrap();
        Ok(Self::new(x, y, z))
    }

    pub fn get_x(&self) -> &CorticalAreaVoxelLinearIndex<QI> {
        &self.coordinates.as_slice()[0]
    }

    pub fn get_y(&self) -> &CorticalAreaVoxelLinearIndex<QI> {
        &self.coordinates.as_slice()[1]
    }

    pub fn get_z(&self) -> &CorticalAreaVoxelLinearIndex<QI> {
        &self.coordinates.as_slice()[2]
    }
}



pub trait CorticalAreaVoxels<
    QI: QuantizedUnsignedIntegerUnwrappedTrait,
    QP: QuantizedDecimalUnwrappedTrait
>
{
    // TODO axis order stuff

    fn get_dimensions(&self) -> CorticalAreaVoxelDimensions<QI>;

    fn get_potential(&self) ->  CorticalAreaVoxelPotential<QP>;
}



pub struct CorticalAreaVoxelsVectors<
    QI: QuantizedUnsignedIntegerUnwrappedTrait,
    QP: QuantizedDecimalUnwrappedTrait
>
{
    dimensions: (),
    stride: (),
    axis_order: (),
    data: (),

}
















/*
pub enum CorticalAreaVoxels<
    QI: QuantizedUnsignedIntegerUnwrappedTrait,
    QP: QuantizedDecimalUnwrappedTrait>
{
    Interconnect(),

}


 */

