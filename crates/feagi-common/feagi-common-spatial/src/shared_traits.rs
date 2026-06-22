use crate::FeagiSpatialError;

#[doc(hidden)]
pub trait SpatialCoordinate2DTrait<Axis>: Sized {
    fn new(x: Axis, y: Axis) -> Self;

    fn get_x(&self) -> &Axis;
    fn get_y(&self) -> &Axis;

    fn get_x_mut(&mut self) -> &mut Axis;
    fn get_y_mut(&mut self) -> &mut Axis;
}

#[doc(hidden)]
pub trait SpatialCoordinate3DTrait<Axis>: Sized {
    fn new(x: Axis, y: Axis, z: Axis) -> Self;

    fn get_x(&self) -> &Axis;
    fn get_y(&self) -> &Axis;
    fn get_z(&self) -> &Axis;

    fn get_x_mut(&mut self) -> &mut Axis;
    fn get_y_mut(&mut self) -> &mut Axis;
    fn get_z_mut(&mut self) -> &mut Axis;
}

#[doc(hidden)]
pub trait SpatialCoordinate4DTrait<Axis>: Sized {
    fn new(x: Axis, y: Axis, z: Axis, w: Axis) -> Self;

    fn get_x(&self) -> &Axis;
    fn get_y(&self) -> &Axis;
    fn get_z(&self) -> &Axis;
    fn get_w(&self) -> &Axis;

    fn get_x_mut(&mut self) -> &mut Axis;
    fn get_y_mut(&mut self) -> &mut Axis;
    fn get_z_mut(&mut self) -> &mut Axis;
    fn get_w_mut(&mut self) -> &mut Axis;
}

#[doc(hidden)]
pub trait SpatialDimensions2DTrait<Axis, Coordinate>: Sized
where
    Axis: PartialOrd,
    Coordinate: SpatialCoordinate2DTrait<Axis>,
{
    fn new_unchecked(x: Axis, y: Axis) -> Self;
    fn new_checked(x: Axis, y: Axis) -> Result<Self, FeagiSpatialError>;

    fn get_x(&self) -> &Axis;
    fn get_y(&self) -> &Axis;

    fn get_x_mut(&mut self) -> &mut Axis;
    fn get_y_mut(&mut self) -> &mut Axis;

    fn does_coordinate_fit(&self, coordinate: Coordinate) -> bool {
        coordinate.get_x() < self.get_x() && coordinate.get_y() < self.get_y()
    }
}

#[doc(hidden)]
pub trait SpatialDimensions3DTrait<Axis, Coordinate>: Sized
where
    Axis: PartialOrd,
    Coordinate: SpatialCoordinate3DTrait<Axis>,
{
    fn new_unchecked(x: Axis, y: Axis, z: Axis) -> Self;
    fn new_checked(x: Axis, y: Axis, z: Axis) -> Result<Self, FeagiSpatialError>;

    fn get_x(&self) -> &Axis;
    fn get_y(&self) -> &Axis;
    fn get_z(&self) -> &Axis;

    fn get_x_mut(&mut self) -> &mut Axis;
    fn get_y_mut(&mut self) -> &mut Axis;
    fn get_z_mut(&mut self) -> &mut Axis;

    fn does_coordinate_fit(&self, coordinate: Coordinate) -> bool {
        coordinate.get_x() < self.get_x()
            && coordinate.get_y() < self.get_y()
            && coordinate.get_z() < self.get_z()
    }
}

#[doc(hidden)]
pub trait SpatialDimensions4DTrait<Axis, Coordinate>: Sized
where
    Axis: PartialOrd,
    Coordinate: SpatialCoordinate4DTrait<Axis>,
{
    fn new_unchecked(x: Axis, y: Axis, z: Axis, w: Axis) -> Self;
    fn new_checked(x: Axis, y: Axis, z: Axis, w: Axis) -> Result<Self, FeagiSpatialError>;

    fn get_x(&self) -> &Axis;
    fn get_y(&self) -> &Axis;
    fn get_z(&self) -> &Axis;
    fn get_w(&self) -> &Axis;

    fn get_x_mut(&mut self) -> &mut Axis;
    fn get_y_mut(&mut self) -> &mut Axis;
    fn get_z_mut(&mut self) -> &mut Axis;
    fn get_w_mut(&mut self) -> &mut Axis;

    fn does_coordinate_fit(&self, coordinate: Coordinate) -> bool {
        coordinate.get_x() < self.get_x()
            && coordinate.get_y() < self.get_y()
            && coordinate.get_z() < self.get_z()
            && coordinate.get_w() < self.get_w()
    }
}
