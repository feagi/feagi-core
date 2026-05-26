use crate::quantizable::base_types::QuantizedSignedIntegerTrait;
use crate::spatial::feagi_data_spatial_error::FeagiDimensionsErrKey;
use crate::spatial::shared_traits::{SpatialCoordinate2DTrait, SpatialDimensions2DTrait};
use crate::spatial::FeagiSpatialError;
// TODO macros that automatically make wrapper, and return wrapped linear index too


//region Coordinate

/// Coordinate defined by Signed integers
pub struct SpatialSintCoordinate2D<QSI: QuantizedSignedIntegerTrait> {
    x: QSI,
    y: QSI,
}

impl<QSI: QuantizedSignedIntegerTrait> SpatialSintCoordinate2D<QSI> {
    pub fn new(x: QSI, y: QSI) -> SpatialSintCoordinate2D<QSI> {
        SpatialSintCoordinate2D {x, y }
    }

    pub fn get_x(&self) -> &QSI {
        &self.x
    }

    pub fn get_y(&self) -> &QSI {
        &self.y
    }


    pub fn get_x_mut(&mut self) -> &mut QSI {
        &mut self.x
    }

    pub fn get_y_mut(&mut self) -> &mut QSI {
        &mut self.y
    }

}

impl<QSI: QuantizedSignedIntegerTrait> SpatialCoordinate2DTrait<QSI> for SpatialSintCoordinate2D<QSI> {
    fn new(x: QSI, y: QSI) -> Self {
        SpatialSintCoordinate2D {x, y }
    }

    fn get_x(&self) -> &QSI {
        &self.x
    }

    fn get_y(&self) -> &QSI {
        &self.y
    }

    fn get_x_mut(&mut self) -> &mut QSI {
        &mut self.x
    }

    fn get_y_mut(&mut self) -> &mut QSI {
        &mut self.y
    }
}

//endregion

//region Dimensions

/// Dimensions defined by Signed integers
pub struct SpatialSintDimensions2D<QSI: QuantizedSignedIntegerTrait> {
    x: QSI,
    y: QSI,
}

impl<QSI: QuantizedSignedIntegerTrait> SpatialSintDimensions2D<QSI> {

    /// Creates new dimensions without checking if any of them are 0
    pub fn new_unchecked(x: QSI, y: QSI) -> SpatialSintDimensions2D<QSI> {
        SpatialSintDimensions2D {x, y }
    }

    /// Creates new dimensions without checking if any of them are 0
    pub fn new_checked(x: QSI, y: QSI) -> Result<SpatialSintDimensions2D<QSI>, FeagiSpatialError>
    {
        // TODO check quant overflow

        if x.is_zero_or_negative() || y.is_zero_or_negative() {
            return Err(FeagiDimensionsErrKey::new("No dimension axis may be 0 or negative!!").into())
        }
        Ok(SpatialSintDimensions2D {x, y})
    }

    pub fn get_x(&self) -> &QSI {
        &self.x
    }

    pub fn get_y(&self) -> &QSI {
        &self.y
    }


    pub fn does_coordinate_fit(&self, coordinate: SpatialSintCoordinate2D<QSI>) -> bool {
        if coordinate.x < self.x && coordinate.y < self.y {
            return true
        }
        false
    }

    // TODO perhaps add debug checks here?

    pub fn get_x_mut(&mut self) -> &mut QSI {
        &mut self.x
    }

    pub fn get_y_mut(&mut self) -> &mut QSI {
        &mut self.y
    }

}


//endregion

impl<QSI: QuantizedSignedIntegerTrait> SpatialDimensions2DTrait<QSI, SpatialSintCoordinate2D<QSI>>
    for SpatialSintDimensions2D<QSI>
{
    fn new_unchecked(x: QSI, y: QSI) -> Self {
        SpatialSintDimensions2D {x, y }
    }

    fn new_checked(x: QSI, y: QSI) -> Result<Self, FeagiSpatialError> {
        SpatialSintDimensions2D::new_checked(x, y)
    }

    fn get_x(&self) -> &QSI {
        &self.x
    }

    fn get_y(&self) -> &QSI {
        &self.y
    }

    fn get_x_mut(&mut self) -> &mut QSI {
        &mut self.x
    }

    fn get_y_mut(&mut self) -> &mut QSI {
        &mut self.y
    }
}



