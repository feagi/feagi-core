
//region Coordinate

use crate::quantizable_linear::base_types::QuantizedIndexCountTrait;
use crate::quantizable_spatial::feagi_data_spatial_error::FeagiDimensionsErrKey;
use crate::quantizable_spatial::shared_traits::{SpatialCoordinate3DTrait, SpatialDimensions3DTrait};
use crate::quantizable_spatial::FeagiSpatialError;

/// Coordinate defined by unsigned integers
pub struct SpatialIndexCoordinate3D<QIC: QuantizedIndexCountTrait> {
    x: QIC,
    y: QIC,
    z: QIC
}

impl<QIC: QuantizedIndexCountTrait> SpatialIndexCoordinate3D<QIC> {
    pub fn new(x: QIC, y: QIC, z: QIC) -> SpatialIndexCoordinate3D<QIC> {
        SpatialIndexCoordinate3D {x, y, z }
    }

    pub fn get_x(&self) -> &QIC {
        &self.x
    }

    pub fn get_y(&self) -> &QIC {
        &self.y
    }

    pub fn get_z(&self) -> &QIC {
        &self.z
    }

    pub fn get_x_mut(&mut self) -> &mut QIC {
        &mut self.x
    }

    pub fn get_y_mut(&mut self) -> &mut QIC {
        &mut self.y
    }

    pub fn get_z_mut(&mut self) -> &mut QIC {
        &mut self.z
    }
}

impl<QIC: QuantizedIndexCountTrait> SpatialCoordinate3DTrait<QIC> for SpatialIndexCoordinate3D<QIC> {
    fn new(x: QIC, y: QIC, z: QIC) -> Self {
        SpatialIndexCoordinate3D {x, y, z }
    }

    fn get_x(&self) -> &QIC {
        &self.x
    }

    fn get_y(&self) -> &QIC {
        &self.y
    }

    fn get_z(&self) -> &QIC {
        &self.z
    }

    fn get_x_mut(&mut self) -> &mut QIC {
        &mut self.x
    }

    fn get_y_mut(&mut self) -> &mut QIC {
        &mut self.y
    }

    fn get_z_mut(&mut self) -> &mut QIC {
        &mut self.z
    }
}

//endregion

//region Dimensions

/// Dimensions defined by unsigned integers
pub struct SpatialIndexDimensions3D<QIC: QuantizedIndexCountTrait> {
    x: QIC,
    y: QIC,
    z: QIC
}

impl<QIC: QuantizedIndexCountTrait> SpatialIndexDimensions3D<QIC> {

    /// Creates new dimensions without checking if any of them are 0
    pub fn new_unchecked(x: QIC, y: QIC, z: QIC) -> SpatialIndexDimensions3D<QIC> {
        SpatialIndexDimensions3D {x, y, z }
    }

    /// Creates new dimensions without checking if any of them are 0
    pub fn new_checked(x: QIC, y: QIC, z: QIC) -> Result<SpatialIndexDimensions3D<QIC>, FeagiSpatialError>
    {
        // TODO check quant overflow

        if x == QIC::QUANT_ZERO || y == QIC::QUANT_ZERO || z == QIC::QUANT_ZERO {
            return Err(FeagiDimensionsErrKey::new("No dimension axis may be 0!").into())
        }
        Ok(SpatialIndexDimensions3D {x, y, z })
    }

    pub fn get_x(&self) -> &QIC {
        &self.x
    }

    pub fn get_y(&self) -> &QIC {
        &self.y
    }

    pub fn get_z(&self) -> &QIC {
        &self.z
    }

    pub fn max_linear_index(&self) -> QIC {
        self.x * self.y * self.z
    }

    pub fn does_coordinate_fit(&self, coordinate: SpatialIndexCoordinate3D<QIC>) -> bool {
        if coordinate.x < self.x && coordinate.y < self.y && coordinate.z < self.z {
            return true
        }
        false
    }

    /// Find the linear index of a given coordinate given incrementing along x -> y -> z
    pub fn coordinate_to_linear_index(&self, coordinate: SpatialIndexCoordinate3D<QIC>) -> QIC {
        coordinate.x + (coordinate.y * self.x) + (coordinate.z * self.x * self.y)
    }

    pub fn linear_index_to_coordinate(&self, linear_index: QIC) -> SpatialIndexCoordinate3D<QIC> {
        let x = linear_index % self.x;
        let y = (linear_index / self.x) % self.y;
        let z = linear_index / (self.x * self.y);

        SpatialIndexCoordinate3D::new(x, y, z)
    }

    // TODO perhaps add debug checks here?

    pub fn get_x_mut(&mut self) -> &mut QIC {
        &mut self.x
    }

    pub fn get_y_mut(&mut self) -> &mut QIC {
        &mut self.y
    }

    pub fn get_z_mut(&mut self) -> &mut QIC {
        &mut self.z
    }
}


//endregion

impl<QIC: QuantizedIndexCountTrait> SpatialDimensions3DTrait<QIC, SpatialIndexCoordinate3D<QIC>>
    for SpatialIndexDimensions3D<QIC>
{
    fn new_unchecked(x: QIC, y: QIC, z: QIC) -> Self {
        SpatialIndexDimensions3D {x, y, z }
    }

    fn new_checked(x: QIC, y: QIC, z: QIC) -> Result<Self, FeagiSpatialError> {
        SpatialIndexDimensions3D::new_checked(x, y, z)
    }

    fn get_x(&self) -> &QIC {
        &self.x
    }

    fn get_y(&self) -> &QIC {
        &self.y
    }

    fn get_z(&self) -> &QIC {
        &self.z
    }

    fn get_x_mut(&mut self) -> &mut QIC {
        &mut self.x
    }

    fn get_y_mut(&mut self) -> &mut QIC {
        &mut self.y
    }

    fn get_z_mut(&mut self) -> &mut QIC {
        &mut self.z
    }
}
