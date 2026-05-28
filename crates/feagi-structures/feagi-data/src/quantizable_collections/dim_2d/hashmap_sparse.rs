use ahash::AHashMap;
use feagi_ecs::collection::{FeagiECSCollectionDataLivesOnCPU, FeagiECSCollectionDataLivesOnDeviceBase};
use crate::quantizable_collections::dim_2d::spatial_shared_traits::{QuantizableSpatialCollection2DBase, QuantizableSpatialCollection2DCPUData, QuantizableSpatialCollection2DIterWithIndex};
use crate::quantizable_collections::shared_traits::{QuantizableLinearCollectionBase, QuantizableLinearCollectionCPUData};
use crate::quantizable_linear::base_types::QuantizedIndexCountTrait;
use crate::quantizable_spatial::index::SpatialIndexDimensions2D;


// TODO some way to free memory, resize?

pub struct QuantizableSpatialCollection2DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    values: AHashMap<LIQ, Value>,
    dimensions: SpatialIndexDimensions2D<LIQ>
}

impl<LIQ, Value> QuantizableSpatialCollection2DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    pub fn new(dimensions: SpatialIndexDimensions2D<LIQ>) -> Self {
        QuantizableSpatialCollection2DHashmapSparse{
            values: AHashMap::new(),
            dimensions
        }
    }

    /// Mainly used to make more unique wrapper systems
    pub fn internal_get_values_mut(&mut self) -> &mut AHashMap<LIQ, Value>{
        &mut self.values
    }

    /// Mainly used to make more unique wrapper systems
    pub fn internal_get_dimensions_mut(&mut self) -> &mut SpatialIndexDimensions2D<LIQ> {
        &mut self.dimensions
    }
}

impl<LIQ, Value> QuantizableLinearCollectionBase<LIQ, Value> for QuantizableSpatialCollection2DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn max_linear_index(&self) -> LIQ {
        self.dimensions.max_linear_index()
    }
}

impl<LIQ, Value> FeagiECSCollectionDataLivesOnDeviceBase for QuantizableSpatialCollection2DHashmapSparse<LIQ, Value> where LIQ: QuantizedIndexCountTrait, Value: Clone, {}

impl<LIQ, Value> FeagiECSCollectionDataLivesOnCPU for QuantizableSpatialCollection2DHashmapSparse<LIQ, Value> where LIQ: QuantizedIndexCountTrait, Value: Clone {}

impl<LIQ, Value> QuantizableLinearCollectionCPUData<LIQ, Value> for QuantizableSpatialCollection2DHashmapSparse<LIQ, Value>
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

impl<LIQ, Value> QuantizableSpatialCollection2DCPUData<LIQ, Value> for QuantizableSpatialCollection2DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
}

impl<LIQ, Value> QuantizableSpatialCollection2DBase<LIQ, Value> for QuantizableSpatialCollection2DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn get_dimensions(&self) -> &SpatialIndexDimensions2D<LIQ> {
        &self.dimensions
    }
}

impl<LIQ, Value> QuantizableSpatialCollection2DIterWithIndex<LIQ, Value> for QuantizableSpatialCollection2DHashmapSparse<LIQ, Value>
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
