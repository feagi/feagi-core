
//region Coordinate

use crate::quantizable_linear::base_types::QuantizedIndexCountTrait;
use crate::quantizable_spatial::feagi_data_spatial_error::FeagiDimensionsErrKey;
use crate::quantizable_spatial::shared_traits::{SpatialCoordinate4DTrait, SpatialDimensions4DTrait};
use crate::quantizable_spatial::FeagiSpatialError;

/// Coordinate defined by index/count integers
pub struct SpatialIndexCoordinate4D<QIC: QuantizedIndexCountTrait> {
    x: QIC,
    y: QIC,
    z: QIC,
    w: QIC
}

impl<QIC: QuantizedIndexCountTrait> SpatialIndexCoordinate4D<QIC> {
    pub fn new(x: QIC, y: QIC, z: QIC, w: QIC) -> SpatialIndexCoordinate4D<QIC> {
        SpatialIndexCoordinate4D {x, y, z, w }
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

    pub fn get_w(&self) -> &QIC {
        &self.w
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

    pub fn get_w_mut(&mut self) -> &mut QIC {
        &mut self.w
    }
}

impl<QIC: QuantizedIndexCountTrait> SpatialCoordinate4DTrait<QIC> for SpatialIndexCoordinate4D<QIC> {
    fn new(x: QIC, y: QIC, z: QIC, w: QIC) -> Self {
        SpatialIndexCoordinate4D {x, y, z, w }
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

    fn get_w(&self) -> &QIC {
        &self.w
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

    fn get_w_mut(&mut self) -> &mut QIC {
        &mut self.w
    }
}

//endregion

//region Dimensions

/// Dimensions defined by index/count integers
pub struct SpatialIndexDimensions4D<QIC: QuantizedIndexCountTrait> {
    x: QIC,
    y: QIC,
    z: QIC,
    w: QIC
}

impl<QIC: QuantizedIndexCountTrait> SpatialIndexDimensions4D<QIC> {

    /// Creates new dimensions without checking if any of them are 0
    pub fn new_unchecked(x: QIC, y: QIC, z: QIC, w: QIC) -> SpatialIndexDimensions4D<QIC> {
        SpatialIndexDimensions4D{x, y, z, w }
    }

    /// Creates new dimensions without checking if any of them are 0
    pub fn new_checked(x: QIC, y: QIC, z: QIC, w: QIC) -> Result<SpatialIndexDimensions4D<QIC>, FeagiSpatialError>
    {
        // TODO check quant overflow

        if x == QIC::QUANT_ZERO || y == QIC::QUANT_ZERO || z == QIC::QUANT_ZERO || w == QIC::QUANT_ZERO {
            return Err(FeagiDimensionsErrKey::new("No dimension axis may be 0!").into())
        }
        Ok(SpatialIndexDimensions4D{x, y, z, w })
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

    pub fn get_w(&self) -> &QIC {
        &self.w
    }

    pub fn max_linear_index(&self) -> QIC {
        self.x * self.y * self.z * self.w
    }

    pub fn does_coordinate_fit(&self, coordinate: SpatialIndexCoordinate4D<QIC>) -> bool {
        if coordinate.x < self.x && coordinate.y < self.y && coordinate.z < self.z && coordinate.w < self.w {
            return true
        }
        false
    }

    /// Find the linear index of a given coordinate given incrementing along x -> y -> z -> w
    pub fn coordinate_to_linear_index(&self, coordinate: SpatialIndexCoordinate4D<QIC>) -> QIC {
        coordinate.x
            + (coordinate.y * self.x)
            + (coordinate.z * self.x * self.y)
            + (coordinate.w * self.x * self.y * self.z)
    }

    pub fn linear_index_to_coordinate(&self, linear_index: QIC) -> SpatialIndexCoordinate4D<QIC> {
        let xy_area = self.x * self.y;
        let xyz_area = xy_area * self.z;

        let x = linear_index % self.x;
        let y = (linear_index / self.x) % self.y;
        let z = (linear_index / xy_area) % self.z;
        let w = linear_index / xyz_area;

        SpatialIndexCoordinate4D::new(x, y, z, w)
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

    pub fn get_w_mut(&mut self) -> &mut QIC {
        &mut self.w
    }
}

//endregion

impl<QIC: QuantizedIndexCountTrait> SpatialDimensions4DTrait<QIC, SpatialIndexCoordinate4D<QIC>>
    for SpatialIndexDimensions4D<QIC>
{
    fn new_unchecked(x: QIC, y: QIC, z: QIC, w: QIC) -> Self {
        SpatialIndexDimensions4D{x, y, z, w }
    }

    fn new_checked(x: QIC, y: QIC, z: QIC, w: QIC) -> Result<Self, FeagiSpatialError> {
        SpatialIndexDimensions4D::new_checked(x, y, z, w)
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

    fn get_w(&self) -> &QIC {
        &self.w
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

    fn get_w_mut(&mut self) -> &mut QIC {
        &mut self.w
    }
}
