use crate::quantizable_linear::base_types::QuantizedSignedIntegerTrait;
use crate::quantizable_spatial::feagi_data_spatial_error::FeagiDimensionsErrKey;
use crate::quantizable_spatial::shared_traits::{SpatialCoordinate4DTrait, SpatialDimensions4DTrait};
use crate::quantizable_spatial::FeagiSpatialError;
// TODO macros that automatically make wrapper, and return wrapped linear index too


//region Coordinate

/// Coordinate defined by Signed integers
pub struct SpatialSintCoordinate4D<QSI: QuantizedSignedIntegerTrait> {
    x: QSI,
    y: QSI,
    z: QSI,
    w: QSI
}

impl<QSI: QuantizedSignedIntegerTrait> SpatialSintCoordinate4D<QSI> {
    pub fn new(x: QSI, y: QSI, z: QSI, w: QSI) -> SpatialSintCoordinate4D<QSI> {
        SpatialSintCoordinate4D {x, y, z, w }
    }

    pub fn get_x(&self) -> &QSI {
        &self.x
    }

    pub fn get_y(&self) -> &QSI {
        &self.y
    }

    pub fn get_z(&self) -> &QSI {
        &self.z
    }

    pub fn get_w(&self) -> &QSI {
        &self.w
    }

    pub fn get_x_mut(&mut self) -> &mut QSI {
        &mut self.x
    }

    pub fn get_y_mut(&mut self) -> &mut QSI {
        &mut self.y
    }

    pub fn get_z_mut(&mut self) -> &mut QSI {
        &mut self.z
    }

    pub fn get_w_mut(&mut self) -> &mut QSI {
        &mut self.w
    }
}

impl<QSI: QuantizedSignedIntegerTrait> SpatialCoordinate4DTrait<QSI> for SpatialSintCoordinate4D<QSI> {
    fn new(x: QSI, y: QSI, z: QSI, w: QSI) -> Self {
        SpatialSintCoordinate4D {x, y, z, w }
    }

    fn get_x(&self) -> &QSI {
        &self.x
    }

    fn get_y(&self) -> &QSI {
        &self.y
    }

    fn get_z(&self) -> &QSI {
        &self.z
    }

    fn get_w(&self) -> &QSI {
        &self.w
    }

    fn get_x_mut(&mut self) -> &mut QSI {
        &mut self.x
    }

    fn get_y_mut(&mut self) -> &mut QSI {
        &mut self.y
    }

    fn get_z_mut(&mut self) -> &mut QSI {
        &mut self.z
    }

    fn get_w_mut(&mut self) -> &mut QSI {
        &mut self.w
    }
}

//endregion

//region Dimensions

/// Dimensions defined by Signed integers
pub struct SpatialDimensions4D<QSI: QuantizedSignedIntegerTrait> {
    x: QSI,
    y: QSI,
    z: QSI,
    w: QSI
}

impl<QSI: QuantizedSignedIntegerTrait> SpatialDimensions4D<QSI> {

    /// Creates new dimensions without checking if any of them are 0
    pub fn new_unchecked(x: QSI, y: QSI, z: QSI, w: QSI) -> SpatialDimensions4D<QSI> {
        SpatialDimensions4D{x, y, z, w }
    }

    /// Creates new dimensions without checking if any of them are 0
    pub fn new_checked(x: QSI, y: QSI, z: QSI, w: QSI) -> Result<SpatialDimensions4D<QSI>, FeagiSpatialError>
    {
        // TODO check quant overflow

        if x.is_zero_or_negative() || y.is_zero_or_negative() || z.is_zero_or_negative() || w.is_zero_or_negative() {
            return Err(FeagiDimensionsErrKey::new("No dimension axis may be 0 or negative!!").into())
        }
        Ok(SpatialDimensions4D{x, y, z, w })
    }

    pub fn get_x(&self) -> &QSI {
        &self.x
    }

    pub fn get_y(&self) -> &QSI {
        &self.y
    }

    pub fn get_z(&self) -> &QSI {
        &self.z
    }

    pub fn get_w(&self) -> &QSI {
        &self.w
    }


    pub fn does_coordinate_fit(&self, coordinate: SpatialSintCoordinate4D<QSI>) -> bool {
        if coordinate.x < self.x && coordinate.y < self.y && coordinate.z < self.z && coordinate.w < self.w {
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

    pub fn get_z_mut(&mut self) -> &mut QSI {
        &mut self.z
    }

    pub fn get_w_mut(&mut self) -> &mut QSI {
        &mut self.w
    }
}


//endregion

impl<QSI: QuantizedSignedIntegerTrait> SpatialDimensions4DTrait<QSI, SpatialSintCoordinate4D<QSI>>
    for SpatialDimensions4D<QSI>
{
    fn new_unchecked(x: QSI, y: QSI, z: QSI, w: QSI) -> Self {
        SpatialDimensions4D{x, y, z, w }
    }

    fn new_checked(x: QSI, y: QSI, z: QSI, w: QSI) -> Result<Self, FeagiSpatialError> {
        SpatialDimensions4D::new_checked(x, y, z, w)
    }

    fn get_x(&self) -> &QSI {
        &self.x
    }

    fn get_y(&self) -> &QSI {
        &self.y
    }

    fn get_z(&self) -> &QSI {
        &self.z
    }

    fn get_w(&self) -> &QSI {
        &self.w
    }

    fn get_x_mut(&mut self) -> &mut QSI {
        &mut self.x
    }

    fn get_y_mut(&mut self) -> &mut QSI {
        &mut self.y
    }

    fn get_z_mut(&mut self) -> &mut QSI {
        &mut self.z
    }

    fn get_w_mut(&mut self) -> &mut QSI {
        &mut self.w
    }
}



