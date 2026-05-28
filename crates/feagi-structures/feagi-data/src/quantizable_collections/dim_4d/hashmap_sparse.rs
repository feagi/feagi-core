use ahash::AHashMap;
use feagi_ecs::collection::{FeagiECSCollectionDataLivesOnCPU, FeagiECSCollectionDataLivesOnDeviceBase};
use crate::quantizable_collections::dim_4d::spatial_shared_traits::{QuantizableSpatialCollection4DBase, QuantizableSpatialCollection4DCPUData, QuantizableSpatialCollection4DIterWithIndex};
use crate::quantizable_collections::shared_traits::{QuantizableLinearCollectionBase, QuantizableLinearCollectionCPUData};
use crate::quantizable_linear::base_types::QuantizedIndexCountTrait;
use crate::quantizable_spatial::index::SpatialIndexDimensions4D;


// TODO some way to free memory, resize?

pub struct QuantizableSpatialCollection4DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    values: AHashMap<LIQ, Value>,
    dimensions: SpatialIndexDimensions4D<LIQ>
}

impl<LIQ, Value> QuantizableSpatialCollection4DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    pub fn new(dimensions: SpatialIndexDimensions4D<LIQ>) -> Self {
        QuantizableSpatialCollection4DHashmapSparse{
            values: AHashMap::new(),
            dimensions
        }
    }

    /// Mainly used to make more unique wrapper systems
    pub fn internal_get_values_mut(&mut self) -> &mut AHashMap<LIQ, Value>{
        &mut self.values
    }

    /// Mainly used to make more unique wrapper systems
    pub fn internal_get_dimensions_mut(&mut self) -> &mut SpatialIndexDimensions4D<LIQ> {
        &mut self.dimensions
    }
}

impl<LIQ, Value> QuantizableLinearCollectionBase<LIQ, Value> for QuantizableSpatialCollection4DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn max_linear_index(&self) -> LIQ {
        self.dimensions.max_linear_index()
    }
}

impl<LIQ, Value> FeagiECSCollectionDataLivesOnDeviceBase for QuantizableSpatialCollection4DHashmapSparse<LIQ, Value> where LIQ: QuantizedIndexCountTrait, Value: Clone, {}

impl<LIQ, Value> FeagiECSCollectionDataLivesOnCPU for QuantizableSpatialCollection4DHashmapSparse<LIQ, Value> where LIQ: QuantizedIndexCountTrait, Value: Clone {}

impl<LIQ, Value> QuantizableLinearCollectionCPUData<LIQ, Value> for QuantizableSpatialCollection4DHashmapSparse<LIQ, Value>
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

impl<LIQ, Value> QuantizableSpatialCollection4DCPUData<LIQ, Value> for QuantizableSpatialCollection4DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
}

impl<LIQ, Value> QuantizableSpatialCollection4DBase<LIQ, Value> for QuantizableSpatialCollection4DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn get_dimensions(&self) -> &SpatialIndexDimensions4D<LIQ> {
        &self.dimensions
    }
}

impl<LIQ, Value> QuantizableSpatialCollection4DIterWithIndex<LIQ, Value> for QuantizableSpatialCollection4DHashmapSparse<LIQ, Value>
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
