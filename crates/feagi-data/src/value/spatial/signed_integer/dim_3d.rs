
//region Coordinate

use feagi_common_quantizable::QuantizedSignedIntegerTrait;
use crate::feagi_data_spatial_error::FeagiDimensionsErrKey;
use crate::FeagiSpatialError;
use crate::shared_traits::{SpatialCoordinate3DTrait, SpatialDimensions3DTrait};

/// Coordinate defined by Signed integers
pub struct SpatialSintCoordinate3D<QSI: QuantizedSignedIntegerTrait> {
    x: QSI,
    y: QSI,
    z: QSI
}

impl<QSI: QuantizedSignedIntegerTrait> SpatialSintCoordinate3D<QSI> {
    pub fn new(x: QSI, y: QSI, z: QSI) -> SpatialSintCoordinate3D<QSI> {
        SpatialSintCoordinate3D {x, y, z }
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

    pub fn get_x_mut(&mut self) -> &mut QSI {
        &mut self.x
    }

    pub fn get_y_mut(&mut self) -> &mut QSI {
        &mut self.y
    }

    pub fn get_z_mut(&mut self) -> &mut QSI {
        &mut self.z
    }
}

impl<QSI: QuantizedSignedIntegerTrait> SpatialCoordinate3DTrait<QSI> for SpatialSintCoordinate3D<QSI> {
    fn new(x: QSI, y: QSI, z: QSI) -> Self {
        SpatialSintCoordinate3D {x, y, z }
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

    fn get_x_mut(&mut self) -> &mut QSI {
        &mut self.x
    }

    fn get_y_mut(&mut self) -> &mut QSI {
        &mut self.y
    }

    fn get_z_mut(&mut self) -> &mut QSI {
        &mut self.z
    }
}

//endregion

//region Dimensions

/// Dimensions defined by Signed integers
pub struct SpatialSintDimensions3D<QSI: QuantizedSignedIntegerTrait> {
    x: QSI,
    y: QSI,
    z: QSI
}

impl<QSI: QuantizedSignedIntegerTrait> SpatialSintDimensions3D<QSI> {

    /// Creates new dimensions without checking if any of them are 0
    pub fn new_unchecked(x: QSI, y: QSI, z: QSI) -> SpatialSintDimensions3D<QSI> {
        SpatialSintDimensions3D {x, y, z }
    }

    /// Creates new dimensions without checking if any of them are 0
    pub fn new_checked(x: QSI, y: QSI, z: QSI) -> Result<SpatialSintDimensions3D<QSI>, FeagiSpatialError>
    {
        // TODO check quant overflow

        if x.is_zero_or_negative() || y.is_zero_or_negative() || z.is_zero_or_negative() {
            return Err(FeagiDimensionsErrKey::new("No dimension axis may be 0 or negative!!").into())
        }
        Ok(SpatialSintDimensions3D {x, y, z })
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

    pub fn does_coordinate_fit(&self, coordinate: SpatialSintCoordinate3D<QSI>) -> bool {
        if coordinate.x < self.x && coordinate.y < self.y && coordinate.z < self.z {
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
}


//endregion

impl<QSI: QuantizedSignedIntegerTrait> SpatialDimensions3DTrait<QSI, SpatialSintCoordinate3D<QSI>>
    for SpatialSintDimensions3D<QSI>
{
    fn new_unchecked(x: QSI, y: QSI, z: QSI) -> Self {
        SpatialSintDimensions3D {x, y, z }
    }

    fn new_checked(x: QSI, y: QSI, z: QSI) -> Result<Self, FeagiSpatialError> {
        SpatialSintDimensions3D::new_checked(x, y, z)
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

    fn get_x_mut(&mut self) -> &mut QSI {
        &mut self.x
    }

    fn get_y_mut(&mut self) -> &mut QSI {
        &mut self.y
    }

    fn get_z_mut(&mut self) -> &mut QSI {
        &mut self.z
    }
}



