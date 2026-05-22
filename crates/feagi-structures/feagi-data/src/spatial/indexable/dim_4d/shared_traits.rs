use crate::LinearIndexCountType;
use crate::spatial::indexable::shared_traits::{SpatialIndexingCoordinate, SpatialIndexingDimension};

pub trait SpatialIndexingCoordinate4D<LinearIndexCount>:
SpatialIndexingCoordinate<LinearIndexCount>
where
    LinearIndexCount: LinearIndexCountType
{
    fn get_x_coord(&self) -> LinearIndexCount;
    fn get_y_coord(&self) -> LinearIndexCount;
    fn get_z_coord(&self) -> LinearIndexCount;
    fn get_d_coord(&self) -> LinearIndexCount;

    fn set_x_coord(&mut self, x: LinearIndexCount);
    fn set_y_coord(&mut self, y: LinearIndexCount);
    fn set_z_coord(&mut self, z: LinearIndexCount);
    fn set_d_coord(&mut self, z: LinearIndexCount);
}


pub trait SpatialIndexingDimension4D<LinearIndexCount, CoordinateType>:
SpatialIndexingDimension<LinearIndexCount, CoordinateType>
where
    LinearIndexCount: LinearIndexCountType,
    CoordinateType: SpatialIndexingCoordinate4D<LinearIndexCount>,
{
    fn get_x_dim(&self) -> LinearIndexCount;
    fn get_y_dim(&self) -> LinearIndexCount;
    fn get_z_dim(&self) -> LinearIndexCount;
    fn get_d_dim(&self) -> LinearIndexCount;

    fn set_x_dim(&mut self, x: LinearIndexCount);
    fn set_y_dim(&mut self, y: LinearIndexCount);
    fn set_z_dim(&mut self, z: LinearIndexCount);
    fn set_d_dim(&mut self, z: LinearIndexCount);

    /// Is given coordinate within these dimensions
    fn contains_coordinate(&self, coordinate: &CoordinateType) -> bool {
        coordinate.get_x_coord() < self.get_x_dim()
            && coordinate.get_y_coord() < self.get_y_dim()
            && coordinate.get_z_coord() < self.get_z_dim()
            && coordinate.get_d_coord() < self.get_d_dim()
    }

    fn get_max_linear_index(&self) -> LinearIndexCount {
        self.get_x_dim() * self.get_y_dim() * self.get_z_dim() * self.get_d_dim()
    }

    fn convert_coordinate_to_linear_index(&self, coord: &CoordinateType) -> LinearIndexCount {
        todo!()
    }

    fn convert_linear_index_to_coordinate(&self, linear_index_index: LinearIndexCount) -> CoordinateType {
        todo!()
    }

}