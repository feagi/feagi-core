
//region Coordinate

use feagi_common_quantizable::QuantizedUnsignedIntegerTrait;
use crate::feagi_data_spatial_error::FeagiDimensionsErrKey;
use crate::FeagiSpatialError;
use crate::shared_traits::{SpatialCoordinate3DTrait, SpatialDimensions3DTrait};

/// Coordinate defined by unsigned integers
pub struct SpatialUintCoordinate3D<QUI: QuantizedUnsignedIntegerTrait> {
    x: QUI,
    y: QUI,
    z: QUI
}

impl<QUI: QuantizedUnsignedIntegerTrait> SpatialUintCoordinate3D<QUI> {
    pub fn new(x: QUI, y: QUI, z: QUI) -> SpatialUintCoordinate3D<QUI> {
        SpatialUintCoordinate3D {x, y, z }
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

    pub fn get_x_mut(&mut self) -> &mut QUI {
        &mut self.x
    }

    pub fn get_y_mut(&mut self) -> &mut QUI {
        &mut self.y
    }

    pub fn get_z_mut(&mut self) -> &mut QUI {
        &mut self.z
    }
}

impl<QUI: QuantizedUnsignedIntegerTrait> SpatialCoordinate3DTrait<QUI> for SpatialUintCoordinate3D<QUI> {
    fn new(x: QUI, y: QUI, z: QUI) -> Self {
        SpatialUintCoordinate3D {x, y, z }
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

    fn get_x_mut(&mut self) -> &mut QUI {
        &mut self.x
    }

    fn get_y_mut(&mut self) -> &mut QUI {
        &mut self.y
    }

    fn get_z_mut(&mut self) -> &mut QUI {
        &mut self.z
    }
}

//endregion

//region Dimensions

/// Dimensions defined by unsigned integers
pub struct SpatialUintDimensions3D<QUI: QuantizedUnsignedIntegerTrait> {
    x: QUI,
    y: QUI,
    z: QUI
}

impl<QUI: QuantizedUnsignedIntegerTrait> SpatialUintDimensions3D<QUI> {

    /// Creates new dimensions without checking if any of them are 0
    pub fn new_unchecked(x: QUI, y: QUI, z: QUI) -> SpatialUintDimensions3D<QUI> {
        SpatialUintDimensions3D {x, y, z }
    }

    /// Creates new dimensions without checking if any of them are 0
    pub fn new_checked(x: QUI, y: QUI, z: QUI) -> Result<SpatialUintDimensions3D<QUI>, FeagiSpatialError>
    {
        // TODO check quant overflow

        if x == QUI::QUANT_ZERO || y == QUI::QUANT_ZERO || z == QUI::QUANT_ZERO {
            return Err(FeagiDimensionsErrKey::new("No dimension axis may be 0!").into())
        }
        Ok(SpatialUintDimensions3D {x, y, z })
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

    pub fn does_coordinate_fit(&self, coordinate: SpatialUintCoordinate3D<QUI>) -> bool {
        if coordinate.x < self.x && coordinate.y < self.y && coordinate.z < self.z {
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
}


//endregion

impl<QUI: QuantizedUnsignedIntegerTrait> SpatialDimensions3DTrait<QUI, SpatialUintCoordinate3D<QUI>>
    for SpatialUintDimensions3D<QUI>
{
    fn new_unchecked(x: QUI, y: QUI, z: QUI) -> Self {
        SpatialUintDimensions3D {x, y, z }
    }

    fn new_checked(x: QUI, y: QUI, z: QUI) -> Result<Self, FeagiSpatialError> {
        SpatialUintDimensions3D::new_checked(x, y, z)
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

    fn get_x_mut(&mut self) -> &mut QUI {
        &mut self.x
    }

    fn get_y_mut(&mut self) -> &mut QUI {
        &mut self.y
    }

    fn get_z_mut(&mut self) -> &mut QUI {
        &mut self.z
    }
}



