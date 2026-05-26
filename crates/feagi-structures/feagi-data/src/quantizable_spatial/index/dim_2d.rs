
//region Coordinate

use crate::quantizable_linear::base_types::QuantizedIndexCountTrait;
use crate::quantizable_spatial::feagi_data_spatial_error::FeagiDimensionsErrKey;
use crate::quantizable_spatial::shared_traits::{SpatialCoordinate2DTrait, SpatialDimensions2DTrait};
use crate::quantizable_spatial::FeagiSpatialError;

/// Coordinate defined by index/count integers
pub struct SpatialIndexCoordinate2D<QIC: QuantizedIndexCountTrait> {
    x: QIC,
    y: QIC,
}

impl<QIC: QuantizedIndexCountTrait> SpatialIndexCoordinate2D<QIC> {
    pub fn new(x: QIC, y: QIC) -> SpatialIndexCoordinate2D<QIC> {
        SpatialIndexCoordinate2D {x, y }
    }

    pub fn get_x(&self) -> &QIC {
        &self.x
    }

    pub fn get_y(&self) -> &QIC {
        &self.y
    }

    pub fn get_x_mut(&mut self) -> &mut QIC {
        &mut self.x
    }

    pub fn get_y_mut(&mut self) -> &mut QIC {
        &mut self.y
    }
}

impl<QIC: QuantizedIndexCountTrait> SpatialCoordinate2DTrait<QIC> for SpatialIndexCoordinate2D<QIC> {
    fn new(x: QIC, y: QIC) -> Self {
        SpatialIndexCoordinate2D {x, y }
    }

    fn get_x(&self) -> &QIC {
        &self.x
    }

    fn get_y(&self) -> &QIC {
        &self.y
    }

    fn get_x_mut(&mut self) -> &mut QIC {
        &mut self.x
    }

    fn get_y_mut(&mut self) -> &mut QIC {
        &mut self.y
    }
}

//endregion

//region Dimensions

/// Dimensions defined by index/count integers
pub struct SpatialIndexDimensions2D<QIC: QuantizedIndexCountTrait> {
    x: QIC,
    y: QIC,
}

impl<QIC: QuantizedIndexCountTrait> SpatialIndexDimensions2D<QIC> {

    /// Creates new dimensions without checking if any of them are 0
    pub fn new_unchecked(x: QIC, y: QIC) -> SpatialIndexDimensions2D<QIC> {
        SpatialIndexDimensions2D {x, y }
    }

    /// Creates new dimensions without checking if any of them are 0
    pub fn new_checked(x: QIC, y: QIC) -> Result<SpatialIndexDimensions2D<QIC>, FeagiSpatialError>
    {
        // TODO check quant overflow

        if x == QIC::QUANT_ZERO || y == QIC::QUANT_ZERO {
            return Err(FeagiDimensionsErrKey::new("No dimension axis may be 0!").into())
        }
        Ok(SpatialIndexDimensions2D {x, y})
    }

    pub fn get_x(&self) -> &QIC {
        &self.x
    }

    pub fn get_y(&self) -> &QIC {
        &self.y
    }

    pub fn max_linear_index(&self) -> QIC {
        self.x * self.y
    }

    pub fn does_coordinate_fit(&self, coordinate: SpatialIndexCoordinate2D<QIC>) -> bool {
        if coordinate.x < self.x && coordinate.y < self.y {
            return true
        }
        false
    }

    /// Find the linear index of a given coordinate given incrementing along x -> y
    pub fn coordinate_to_linear_index(&self, coordinate: SpatialIndexCoordinate2D<QIC>) -> QIC {
        coordinate.x + (coordinate.y * self.x)
    }

    pub fn linear_index_to_coordinate(&self, linear_index: QIC) -> SpatialIndexCoordinate2D<QIC> {
        let x = linear_index % self.x;
        let y = linear_index / self.x;

        SpatialIndexCoordinate2D::new(x, y)
    }

    // TODO perhaps add debug checks here?

    pub fn get_x_mut(&mut self) -> &mut QIC {
        &mut self.x
    }

    pub fn get_y_mut(&mut self) -> &mut QIC {
        &mut self.y
    }
}

//endregion

impl<QIC: QuantizedIndexCountTrait> SpatialDimensions2DTrait<QIC, SpatialIndexCoordinate2D<QIC>>
    for SpatialIndexDimensions2D<QIC>
{
    fn new_unchecked(x: QIC, y: QIC) -> Self {
        SpatialIndexDimensions2D {x, y }
    }

    fn new_checked(x: QIC, y: QIC) -> Result<Self, FeagiSpatialError> {
        SpatialIndexDimensions2D::new_checked(x, y)
    }

    fn get_x(&self) -> &QIC {
        &self.x
    }

    fn get_y(&self) -> &QIC {
        &self.y
    }

    fn get_x_mut(&mut self) -> &mut QIC {
        &mut self.x
    }

    fn get_y_mut(&mut self) -> &mut QIC {
        &mut self.y
    }
}
