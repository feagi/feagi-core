use crate::quantizable_collections::shared_traits::{QuantizableLinearCollectionCPUData, QuantizableLinearCollectionSyncSparse};
use crate::quantizable_linear::base_types::QuantizedIndexCountTrait;
use crate::quantizable_spatial::index::{SpatialIndexCoordinate3D, SpatialIndexDimensions3D};


pub trait QuantizableSpatialCollection3DBase<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn get_dimensions(&self) -> &SpatialIndexDimensions3D<LIQ>;
    
    fn max_linear_index(&self) -> LIQ {
        self.get_dimensions().max_linear_index()
    }
    
    fn does_coordinate_fit(&self, coord: SpatialIndexCoordinate3D<LIQ>) -> bool {
        self.get_dimensions().does_coordinate_fit(coord)
    }
    
    fn coordinate_to_linear_index(&self, coord: SpatialIndexCoordinate3D<LIQ>) -> LIQ {
        self.get_dimensions().coordinate_to_linear_index(coord)
    }
    
    fn linear_index_to_coordinate(&self, linear_index: LIQ) -> SpatialIndexCoordinate3D<LIQ> {
        self.get_dimensions().linear_index_to_coordinate(linear_index)
    }
}

/// Data is accessible by the CPU (CPU ECS)
pub trait QuantizableSpatialCollection3DCPUData<LIQ, Value>:
QuantizableSpatialCollection3DBase<LIQ, Value>
+ QuantizableLinearCollectionCPUData<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn try_get_value_by_coordinate(&self, coordinate: SpatialIndexCoordinate3D<LIQ>) -> Option<&Value> {
        self.try_get_value(self.coordinate_to_linear_index(coordinate))
    }

    fn try_get_value_by_coordinate_mut(&mut self, coordinate: SpatialIndexCoordinate3D<LIQ>) -> Option<&mut Value> {
        self.try_get_value_mut(self.coordinate_to_linear_index(coordinate))
    }
}


/// Data is iterable with the index and coordinate
pub trait QuantizableSpatialCollection3DIterWithCoordinate<LIQ, Value>:
QuantizableSpatialCollection3DCPUData<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn iter_with_index_and_coordinate<'a>(&'a self) -> impl Iterator<Item = (LIQ, SpatialIndexCoordinate3D<LIQ>, &'a Value)> where Value: 'a;

    fn iter_mut_with_index_and_coordinate<'a>(&'a mut self) -> impl Iterator<Item = (LIQ, SpatialIndexCoordinate3D<LIQ>, &'a mut Value)> where Value: 'a;
}


/// Structure is sparse, and thus can add / remove items
pub trait QuantizableSpatialCollection3DSyncSparse<LIQ, Value>:
QuantizableLinearCollectionSyncSparse<LIQ, Value>
+ QuantizableSpatialCollection3DCPUData<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn insert_value_at_coordinate(&mut self, coordinate: SpatialIndexCoordinate3D<LIQ>, value: Value) -> Option<Value> {
        self.insert_value_at_index(
            self.coordinate_to_linear_index(coordinate),
            value
        )
    }

    fn remove_value_at_coordinate(&mut self, coordinate: SpatialIndexCoordinate3D<LIQ>) -> Option<Value> {
        self.remove_value_at_index(
            self.coordinate_to_linear_index(coordinate),
        )
    }
}


/// Structure can be resized without needing to wait (no async)
pub trait QuantizableSpatialCollection3DSyncResizable<LIQ, Value>:
QuantizableSpatialCollection3DCPUData<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn resize_to_new_dimensions(&mut self, new_dimensions: SpatialIndexDimensions3D<LIQ>);
}