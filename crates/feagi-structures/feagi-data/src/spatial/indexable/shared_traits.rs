use crate::feagi_ecs::component::{FECSComponentBase};
use crate::LinearIndexCountType;


/// A trait that defines an Xd coordinate within X space
pub trait SpatialIndexingCoordinate<LinearIndexCount>:
where
    LinearIndexCount: LinearIndexCountType
{

}

/// A trait that defines an Xd size within X space
pub trait SpatialIndexingDimension<LinearIndexCount, CoordinateType>:
where
    LinearIndexCount: LinearIndexCountType,
    CoordinateType: SpatialIndexingCoordinate<LinearIndexCount>,
{
    fn contains_coordinate(&self, coordinate: &CoordinateType) -> bool;
    
    fn get_max_linear_index(&self) -> LinearIndexCount;
    
    fn convert_coordinate_to_linear_index(&self, coord: &CoordinateType) -> LinearIndexCount;
    
    fn convert_linear_index_to_coordinate(&self, linear_index_index: LinearIndexCount) -> CoordinateType;
}