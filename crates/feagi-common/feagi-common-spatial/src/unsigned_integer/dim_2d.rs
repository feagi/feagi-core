
//region Coordinate

use feagi_common_quantizable::QuantizedUnsignedIntegerTrait;
use crate::feagi_data_spatial_error::FeagiDimensionsErrKey;
use crate::FeagiSpatialError;
use crate::shared_traits::{SpatialCoordinate2DTrait, SpatialDimensions2DTrait};

/// Coordinate defined by unsigned integers
pub struct SpatialUintCoordinate2D<QUI: QuantizedUnsignedIntegerTrait> {
    x: QUI,
    y: QUI,
}

impl<QUI: QuantizedUnsignedIntegerTrait> SpatialUintCoordinate2D<QUI> {
    pub fn new(x: QUI, y: QUI) -> SpatialUintCoordinate2D<QUI> {
        SpatialUintCoordinate2D {x, y }
    }

    pub fn get_x(&self) -> &QUI {
        &self.x
    }

    pub fn get_y(&self) -> &QUI {
        &self.y
    }


    pub fn get_x_mut(&mut self) -> &mut QUI {
        &mut self.x
    }

    pub fn get_y_mut(&mut self) -> &mut QUI {
        &mut self.y
    }

}

impl<QUI: QuantizedUnsignedIntegerTrait> SpatialCoordinate2DTrait<QUI> for SpatialUintCoordinate2D<QUI> {
    fn new(x: QUI, y: QUI) -> Self {
        SpatialUintCoordinate2D {x, y }
    }

    fn get_x(&self) -> &QUI {
        &self.x
    }

    fn get_y(&self) -> &QUI {
        &self.y
    }

    fn get_x_mut(&mut self) -> &mut QUI {
        &mut self.x
    }

    fn get_y_mut(&mut self) -> &mut QUI {
        &mut self.y
    }
}

//endregion

//region Dimensions

/// Dimensions defined by unsigned integers
pub struct SpatialUintDimensions2D<QUI: QuantizedUnsignedIntegerTrait> {
    x: QUI,
    y: QUI,
}

impl<QUI: QuantizedUnsignedIntegerTrait> SpatialUintDimensions2D<QUI> {

    /// Creates new dimensions without checking if any of them are 0
    pub fn new_unchecked(x: QUI, y: QUI) -> SpatialUintDimensions2D<QUI> {
        SpatialUintDimensions2D {x, y }
    }

    /// Creates new dimensions without checking if any of them are 0
    pub fn new_checked(x: QUI, y: QUI) -> Result<SpatialUintDimensions2D<QUI>, FeagiSpatialError>
    {
        // TODO check quant overflow

        if x == QUI::QUANT_ZERO || y == QUI::QUANT_ZERO {
            return Err(FeagiDimensionsErrKey::new("No dimension axis may be 0!").into())
        }
        Ok(SpatialUintDimensions2D {x, y})
    }

    pub fn get_x(&self) -> &QUI {
        &self.x
    }

    pub fn get_y(&self) -> &QUI {
        &self.y
    }


    pub fn does_coordinate_fit(&self, coordinate: SpatialUintCoordinate2D<QUI>) -> bool {
        if coordinate.x < self.x && coordinate.y < self.y {
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

}


//endregion

impl<QUI: QuantizedUnsignedIntegerTrait> SpatialDimensions2DTrait<QUI, SpatialUintCoordinate2D<QUI>>
    for SpatialUintDimensions2D<QUI>
{
    fn new_unchecked(x: QUI, y: QUI) -> Self {
        SpatialUintDimensions2D {x, y }
    }

    fn new_checked(x: QUI, y: QUI) -> Result<Self, FeagiSpatialError> {
        SpatialUintDimensions2D::new_checked(x, y)
    }

    fn get_x(&self) -> &QUI {
        &self.x
    }

    fn get_y(&self) -> &QUI {
        &self.y
    }

    fn get_x_mut(&mut self) -> &mut QUI {
        &mut self.x
    }

    fn get_y_mut(&mut self) -> &mut QUI {
        &mut self.y
    }
}



