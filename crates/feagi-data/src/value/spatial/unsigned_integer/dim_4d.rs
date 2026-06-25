
//region Coordinate

use feagi_common_quantizable::QuantizedUnsignedIntegerTrait;
use crate::feagi_data_spatial_error::FeagiDimensionsErrKey;
use crate::FeagiSpatialError;
use crate::shared_traits::{SpatialCoordinate4DTrait, SpatialDimensions4DTrait};

/// Coordinate defined by unsigned integers
pub struct SpatialUintCoordinate4D<QUI: QuantizedUnsignedIntegerTrait> {
    x: QUI,
    y: QUI,
    z: QUI,
    w: QUI
}

impl<QUI: QuantizedUnsignedIntegerTrait> SpatialUintCoordinate4D<QUI> {
    pub fn new(x: QUI, y: QUI, z: QUI, w: QUI) -> SpatialUintCoordinate4D<QUI> {
        SpatialUintCoordinate4D {x, y, z, w }
    }

    pub fn get_x(&self) -> &QUI {
        &self.x
    }

    pub fn get_y(&self) -> &QUI {
        &self.y
    }

    pub fn get_z(&self) -> &QUI {
        &self.z
    }

    pub fn get_w(&self) -> &QUI {
        &self.w
    }

    pub fn get_x_mut(&mut self) -> &mut QUI {
        &mut self.x
    }

    pub fn get_y_mut(&mut self) -> &mut QUI {
        &mut self.y
    }

    pub fn get_z_mut(&mut self) -> &mut QUI {
        &mut self.z
    }

    pub fn get_w_mut(&mut self) -> &mut QUI {
        &mut self.w
    }
}

impl<QUI: QuantizedUnsignedIntegerTrait> SpatialCoordinate4DTrait<QUI> for SpatialUintCoordinate4D<QUI> {
    fn new(x: QUI, y: QUI, z: QUI, w: QUI) -> Self {
        SpatialUintCoordinate4D {x, y, z, w }
    }

    fn get_x(&self) -> &QUI {
        &self.x
    }

    fn get_y(&self) -> &QUI {
        &self.y
    }

    fn get_z(&self) -> &QUI {
        &self.z
    }

    fn get_w(&self) -> &QUI {
        &self.w
    }

    fn get_x_mut(&mut self) -> &mut QUI {
        &mut self.x
    }

    fn get_y_mut(&mut self) -> &mut QUI {
        &mut self.y
    }

    fn get_z_mut(&mut self) -> &mut QUI {
        &mut self.z
    }

    fn get_w_mut(&mut self) -> &mut QUI {
        &mut self.w
    }
}

//endregion

//region Dimensions

/// Dimensions defined by unsigned integers
pub struct SpatialDimensions4D<QUI: QuantizedUnsignedIntegerTrait> {
    x: QUI,
    y: QUI,
    z: QUI,
    w: QUI
}

impl<QUI: QuantizedUnsignedIntegerTrait> SpatialDimensions4D<QUI> {

    /// Creates new dimensions without checking if any of them are 0
    pub fn new_unchecked(x: QUI, y: QUI, z: QUI, w: QUI) -> SpatialDimensions4D<QUI> {
        SpatialDimensions4D{x, y, z, w }
    }

    /// Creates new dimensions without checking if any of them are 0
    pub fn new_checked(x: QUI, y: QUI, z: QUI, w: QUI) -> Result<SpatialDimensions4D<QUI>, FeagiSpatialError>
    {
        // TODO check quant overflow

        if x == QUI::QUANT_ZERO || y == QUI::QUANT_ZERO || z == QUI::QUANT_ZERO || w == QUI::QUANT_ZERO {
            return Err(FeagiDimensionsErrKey::new("No dimension axis may be 0!").into())
        }
        Ok(SpatialDimensions4D{x, y, z, w })
    }

    pub fn get_x(&self) -> &QUI {
        &self.x
    }

    pub fn get_y(&self) -> &QUI {
        &self.y
    }

    pub fn get_z(&self) -> &QUI {
        &self.z
    }

    pub fn get_w(&self) -> &QUI {
        &self.w
    }


    pub fn does_coordinate_fit(&self, coordinate: SpatialUintCoordinate4D<QUI>) -> bool {
        if coordinate.x < self.x && coordinate.y < self.y && coordinate.z < self.z && coordinate.w < self.w {
            return true
        }
        false
    }

    // TODO perhaps add debug checks here?

    pub fn get_x_mut(&mut self) -> &mut QUI {
        &mut self.x
    }

    pub fn get_y_mut(&mut self) -> &mut QUI {
        &mut self.y
    }

    pub fn get_z_mut(&mut self) -> &mut QUI {
        &mut self.z
    }

    pub fn get_w_mut(&mut self) -> &mut QUI {
        &mut self.w
    }
}


//endregion

impl<QUI: QuantizedUnsignedIntegerTrait> SpatialDimensions4DTrait<QUI, SpatialUintCoordinate4D<QUI>>
    for SpatialDimensions4D<QUI>
{
    fn new_unchecked(x: QUI, y: QUI, z: QUI, w: QUI) -> Self {
        SpatialDimensions4D{x, y, z, w }
    }

    fn new_checked(x: QUI, y: QUI, z: QUI, w: QUI) -> Result<Self, FeagiSpatialError> {
        SpatialDimensions4D::new_checked(x, y, z, w)
    }

    fn get_x(&self) -> &QUI {
        &self.x
    }

    fn get_y(&self) -> &QUI {
        &self.y
    }

    fn get_z(&self) -> &QUI {
        &self.z
    }

    fn get_w(&self) -> &QUI {
        &self.w
    }

    fn get_x_mut(&mut self) -> &mut QUI {
        &mut self.x
    }

    fn get_y_mut(&mut self) -> &mut QUI {
        &mut self.y
    }

    fn get_z_mut(&mut self) -> &mut QUI {
        &mut self.z
    }

    fn get_w_mut(&mut self) -> &mut QUI {
        &mut self.w
    }
}



