use crate::LinearIndexCountType;
use crate::spatial::FeagiDataSpatialError;

// TODO macros that automatically make wrapper, and return wrapped linear index too


//region Coordinate

pub struct SpatialCoordinate4D<CI: LinearIndexCountType> {
    x: CI,
    y: CI,
    z: CI,
    w: CI
}

impl<CI: LinearIndexCountType> SpatialCoordinate4D<CI> {
    pub fn new(x: CI, y: CI, z: CI, w: CI) -> SpatialCoordinate4D<CI> {
        SpatialCoordinate4D{x, y, z, w }
    }

    pub fn get_x(&self) -> &CI {
        &self.x
    }

    pub fn get_y(&self) -> &CI {
        &self.y
    }

    pub fn get_z(&self) -> &CI {
        &self.z
    }

    pub fn get_w(&self) -> &CI {
        &self.w
    }

    pub fn get_x_mut(&mut self) -> &mut CI {
        &mut self.x
    }

    pub fn get_y_mut(&mut self) -> &mut CI {
        &mut self.y
    }

    pub fn get_z_mut(&mut self) -> &mut CI {
        &mut self.z
    }

    pub fn get_w_mut(&mut self) -> &mut CI {
        &mut self.w
    }
}

//endregion

//region Dimensions

pub struct SpatialDimensions4D<CI: LinearIndexCountType> {
    x: CI,
    y: CI,
    z: CI,
    w: CI
}

impl<CI: LinearIndexCountType> SpatialDimensions4D<CI> {

    /// Creates new dimensions without checking if any of them are 0
    pub fn new_unchecked(x: CI, y: CI, z: CI, w: CI) -> SpatialDimensions4D<CI> {
        SpatialDimensions4D{x, y, z, w }
    }

    /// Creates new dimensions without checking if any of them are 0
    pub fn new_checked(x: CI, y: CI, z: CI, w: CI) -> Result<SpatialDimensions4D<CI>, FeagiDataSpatialError>
    {
        // TODO check quant overflow

        if x == CI::QUANT_ZERO || y == CI::QUANT_ZERO || z == CI::QUANT_ZERO || w == CI::QUANT_ZERO {
            return Err(FeagiDataSpatialError::InvalidDimensions{
                context: "No dimension axis may be 0!",
            } )
        }
        Ok(SpatialDimensions4D{x, y, z, w })
    }

    pub fn get_x(&self) -> &CI {
        &self.x
    }

    pub fn get_y(&self) -> &CI {
        &self.y
    }

    pub fn get_z(&self) -> &CI {
        &self.z
    }

    pub fn get_w(&self) -> &CI {
        &self.w
    }


    pub fn does_coordinate_fit(&self, coordinate: SpatialCoordinate4D<CI>) -> bool {
        if coordinate.x < self.x && coordinate.y < self.y && coordinate.z < self.z && coordinate.w < self.w {
            return true
        }
        false
    }

    pub fn max_linear_index(&self) -> CI {
        self.x * self.y * self.z * self.w
    }

    pub fn coordinate_to_linear_index(&self, coordinate: SpatialCoordinate4D<CI>) -> CI {
        todo!()
    }

    pub fn linear_index_to_coordinate(&self, linear_index: CI) -> SpatialCoordinate4D<CI> {
        todo!()
    }


    // TODO perhaps add debug checks here?

    pub fn get_x_mut(&mut self) -> &mut CI {
        &mut self.x
    }

    pub fn get_y_mut(&mut self) -> &mut CI {
        &mut self.y
    }

    pub fn get_z_mut(&mut self) -> &mut CI {
        &mut self.z
    }

    pub fn get_w_mut(&mut self) -> &mut CI {
        &mut self.w
    }
}


//endregion



