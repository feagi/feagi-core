use ahash::AHashMap;
use feagi_ecs::tag_device::{FeagiECSTagCPU, FeagiECSTagGenericDevice};
use crate::quantizable_collections::dim_3d::spatial_shared_traits::{QuantizableSpatialCollection3DBase, QuantizableSpatialCollection3DCPUData, QuantizableSpatialCollection3DIterWithCoordinate};
use crate::quantizable_collections::shared_traits::{QuantizableLinearCollectionBase, QuantizableLinearCollectionCPUData, QuantizableLinearCollectionCPUIterWithIndex, QuantizableLinearCollectionCPUSparse};
use crate::quantizable_linear::base_types::QuantizedIndexCountTrait;
use crate::quantizable_spatial::index::{SpatialIndexCoordinate3D, SpatialIndexDimensions3D};


// TODO some way to free memory, resize?

pub struct QuantizableSpatialCollection3DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    values: AHashMap<LIQ, Value>,
    dimensions: SpatialIndexDimensions3D<LIQ>
}

impl<LIQ, Value> QuantizableSpatialCollection3DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    pub fn new(dimensions: SpatialIndexDimensions3D<LIQ>) -> Self {
        QuantizableSpatialCollection3DHashmapSparse{
            values: AHashMap::new(),
            dimensions
        }
    }

    /// Mainly used to make more unique wrapper systems
    pub fn internal_get_values_mut(&mut self) -> &mut AHashMap<LIQ, Value>{
        &mut self.values
    }

    /// Mainly used to make more unique wrapper systems
    pub fn internal_get_dimensions_mut(&mut self) -> &mut SpatialIndexDimensions3D<LIQ> {
        &mut self.dimensions
    }
}

impl<LIQ, Value> QuantizableLinearCollectionBase<LIQ, Value> for QuantizableSpatialCollection3DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn max_linear_index(&self) -> LIQ {
        self.dimensions.max_linear_index()
    }
}

impl<LIQ, Value> QuantizableSpatialCollection3DBase<LIQ, Value> for QuantizableSpatialCollection3DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn get_dimensions(&self) -> &SpatialIndexDimensions3D<LIQ> {
        &self.dimensions
    }
}


//region ECS CPU Access

impl<LIQ, Value> QuantizableLinearCollectionCPUData<LIQ, Value> for QuantizableSpatialCollection3DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn try_get_value(&self, index: LIQ) -> Option<&Value> {
        self.values.get(&index)
    }

    fn try_get_value_mut(&mut self, index: LIQ) -> Option<&mut Value> {
        self.values.get_mut(&index)
    }

    // The unchecked are the same

    fn get_unchecked_value(&self, index: LIQ) -> &Value {
        self.values.get(&index).unwrap()
    }

    fn get_unchecked_value_mut(&mut self, index: LIQ) -> &mut Value {
        self.values.get_mut(&index).unwrap()
    }
}

impl<LIQ, Value> QuantizableLinearCollectionCPUIterWithIndex<LIQ, Value> for QuantizableSpatialCollection3DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn iter_with_index<'a>(&'a self) -> impl Iterator<Item=(LIQ, &'a Value)>
    where
        Value: 'a
    {
        self.values
            .iter()
            .map(|(index, value)| (*index, value))
    }

    fn iter_mut_with_index<'a>(&'a mut self) -> impl Iterator<Item=(LIQ, &'a mut Value)>
    where
        Value: 'a
    {
        self.values
            .iter_mut()
            .map(|(index, value)| (*index, value))
    }
}

impl<LIQ, Value> QuantizableLinearCollectionCPUSparse<LIQ, Value> for QuantizableSpatialCollection3DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn insert_value_at_index(&mut self, index: LIQ, value: Value) -> Option<Value> {
        self.values.insert(index, value)
    }

    fn remove_value_at_index(&mut self, index: LIQ) -> Option<Value> {
        self.values.remove(&index)
    }
}

impl<LIQ, Value> QuantizableSpatialCollection3DCPUData<LIQ, Value> for QuantizableSpatialCollection3DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
}

impl<LIQ, Value> QuantizableSpatialCollection3DIterWithCoordinate<LIQ, Value> for QuantizableSpatialCollection3DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn iter_with_index_and_coordinate<'a>(&'a self) -> impl Iterator<Item=(LIQ, SpatialIndexCoordinate3D<LIQ>, &'a Value)>
    where
        Value: 'a
    {
        let dimensions = &self.dimensions;
        self.values
            .iter()
            .map(move |(index, value)| (*index, dimensions.linear_index_to_coordinate(*index), value))
    }

    fn iter_mut_with_index_and_coordinate<'a>(&'a mut self) -> impl Iterator<Item=(LIQ, SpatialIndexCoordinate3D<LIQ>, &'a mut Value)>
    where
        Value: 'a
    {
        let dimensions = &self.dimensions;
        self.values
            .iter_mut()
            .map(move |(index, value)| (*index, dimensions.linear_index_to_coordinate(*index), value))
    }
}

//endregion


//region ECS Tagging

impl<LIQ, Value> FeagiECSTagGenericDevice for QuantizableSpatialCollection3DHashmapSparse<LIQ, Value> where LIQ: QuantizedIndexCountTrait, Value: Clone, {}

impl<LIQ, Value> FeagiECSTagCPU for QuantizableSpatialCollection3DHashmapSparse<LIQ, Value> where LIQ: QuantizedIndexCountTrait, Value: Clone, {}

//endregion