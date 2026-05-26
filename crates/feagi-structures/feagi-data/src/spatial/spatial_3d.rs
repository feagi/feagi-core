use crate::LinearIndexCountType;
use crate::spatial::FeagiDataSpatialError;

// TODO macros that automatically make wrapper, and return wrapped linear index too


//region Coordinate

pub struct SpatialCoordinate3D<CI: LinearIndexCountType> {
    x: CI,
    y: CI,
    z: CI
}

impl<CI: LinearIndexCountType> SpatialCoordinate3D<CI> {
    pub fn new(x: CI, y: CI, z: CI) -> SpatialCoordinate3D<CI> {
        SpatialCoordinate3D{x, y, z }
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

    pub fn get_x_mut(&mut self) -> &mut CI {
        &mut self.x
    }

    pub fn get_y_mut(&mut self) -> &mut CI {
        &mut self.y
    }

    pub fn get_z_mut(&mut self) -> &mut CI {
        &mut self.z
    }
}

//endregion

//region Dimensions

pub struct SpatialDimensions3D<CI: LinearIndexCountType> {
    x: CI,
    y: CI,
    z: CI
}

impl<CI: LinearIndexCountType> SpatialDimensions3D<CI> {

    /// Creates new dimensions without checking if any of them are 0
    pub fn new_unchecked(x: CI, y: CI, z: CI) -> SpatialDimensions3D<CI> {
        SpatialDimensions3D{x, y, z }
    }

    /// Creates new dimensions without checking if any of them are 0
    pub fn new_checked(x: CI, y: CI, z: CI) -> Result<SpatialDimensions3D<CI>, FeagiDataSpatialError>
    {
        // TODO check quant overflow

        if x == CI::QUANT_ZERO || y == CI::QUANT_ZERO || z == CI::QUANT_ZERO {
            return Err(FeagiDataSpatialError::InvalidDimensions{
                context: "No dimension axis may be 0!",
            } )
        }
        Ok(SpatialDimensions3D{x, y, z })
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

    pub fn does_coordinate_fit(&self, coordinate: SpatialCoordinate3D<CI>) -> bool {
        if coordinate.x < self.x && coordinate.y < self.y && coordinate.z < self.z {
            return true
        }
        false
    }

    pub fn max_linear_index(&self) -> CI {
        self.x * self.y * self.z
    }

    pub fn coordinate_to_linear_index(&self, coordinate: SpatialCoordinate3D<CI>) -> CI {
        todo!()
    }

    pub fn linear_index_to_coordinate(&self, linear_index: CI) -> SpatialCoordinate3D<CI> {
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
}


//endregion



