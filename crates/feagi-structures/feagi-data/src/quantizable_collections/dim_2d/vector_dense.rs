use feagi_ecs::collection::{FeagiECSCollectionOnCPU, FeagiECSCollectionOnDevice};
use crate::quantizable_collections::shared_traits::{QuantizableLinearCollectionAsSlice, QuantizableLinearCollectionBase, QuantizableLinearCollectionCPUData, QuantizableLinearCollectionIterWithIndex};
use crate::quantizable_collections::dim_2d::spatial_shared_traits::{QuantizableSpatialCollection2DBase, QuantizableSpatialCollection2DCPUData, QuantizableSpatialCollection2DIterWithCoordinate};
use crate::quantizable_linear::base_types::QuantizedIndexCountTrait;
use crate::quantizable_spatial::index::{SpatialIndexCoordinate2D, SpatialIndexDimensions2D};

pub struct QuantizableSpatialCollection2DVectorDense<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    values: Vec<Value>,
    dimensions: SpatialIndexDimensions2D<LIQ>
}

impl<LIQ, Value> QuantizableSpatialCollection2DVectorDense<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    pub fn new_uniform(dimensions: SpatialIndexDimensions2D<LIQ>, filling_value: Value) -> Self {
        let values = vec![filling_value; dimensions.max_linear_index().to_usize()];

        Self {
            values,
            dimensions,
        }
    }

    pub fn new_with_iter<I>(dimensions: SpatialIndexDimensions2D<LIQ>, iterator: I) -> Self
    where
        I: IntoIterator<Item = Value>,
    {
        // TODO debug checks
        /*
        let expected_len = dimensions.max_linear_index().to_usize();
        let values: Vec<Value> = iterator.into_iter().collect();

        assert_eq!(
            values.len(),
            expected_len,
            "iterator must produce exactly one value per spatial element",
        );

         */

        let values: Vec<Value> = iterator.into_iter().collect();

        Self {
            values,
            dimensions,
        }
    }

    /// Mainly used to make more unique wrapper systems
    pub fn internal_get_values_mut(&mut self) -> &mut Vec<Value> {
        &mut self.values
    }

    /// Mainly used to make more unique wrapper systems
    pub fn internal_get_dimensions_mut(&mut self) -> &mut SpatialIndexDimensions2D<LIQ> {
        &mut self.dimensions
    }
}

impl<LIQ, Value> QuantizableLinearCollectionBase<LIQ, Value> for QuantizableSpatialCollection2DVectorDense<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn max_linear_index(&self) -> LIQ {
        self.dimensions.max_linear_index()
    }
}

impl<LIQ, Value> FeagiECSCollectionOnDevice for QuantizableSpatialCollection2DVectorDense<LIQ, Value> where LIQ: QuantizedIndexCountTrait, Value: Clone, {}

impl<LIQ, Value> FeagiECSCollectionOnCPU for QuantizableSpatialCollection2DVectorDense<LIQ, Value> where LIQ: QuantizedIndexCountTrait, Value: Clone {}

impl<LIQ, Value> QuantizableLinearCollectionCPUData<LIQ, Value> for QuantizableSpatialCollection2DVectorDense<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn try_get_value(&self, index: LIQ) -> Option<&Value> {
        self.values.get(index.to_usize())
    }

    fn try_get_value_mut(&mut self, index: LIQ) -> Option<&mut Value> {
        self.values.get_mut(index.to_usize())
    }

    fn get_unchecked_value(&self, index: LIQ) -> &Value {
        &self.values[index.to_usize()]
    }

    fn get_unchecked_value_mut(&mut self, index: LIQ) -> &mut Value {
        &mut self.values[index.to_usize()]
    }
}

impl<LIQ, Value> QuantizableLinearCollectionAsSlice<LIQ, Value> for QuantizableSpatialCollection2DVectorDense<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn get_values_slice(&self) -> &[Value] {
        self.values.as_slice()
    }

    fn get_values_slice_mut(&mut self) -> &mut [Value] {
        self.values.as_mut_slice()
    }
}

impl<LIQ, Value> QuantizableLinearCollectionIterWithIndex<LIQ, Value> for QuantizableSpatialCollection2DVectorDense<LIQ, Value>
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
            .enumerate()
            .map(|(index, value)| (LIQ::from_usize_unchecked(index), value))
    }

    fn iter_mut_with_index<'a>(&'a mut self) -> impl Iterator<Item=(LIQ, &'a mut Value)>
    where
        Value: 'a
    {
        self.values
            .iter_mut()
            .enumerate()
            .map(|(index, value)| (LIQ::from_usize_unchecked(index), value))
    }
}

impl<LIQ, Value> QuantizableSpatialCollection2DCPUData<LIQ, Value> for QuantizableSpatialCollection2DVectorDense<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
}

impl<LIQ, Value> QuantizableSpatialCollection2DBase<LIQ, Value> for QuantizableSpatialCollection2DVectorDense<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn get_dimensions(&self) -> &SpatialIndexDimensions2D<LIQ> {
        &self.dimensions
    }
}

impl<LIQ, Value> QuantizableSpatialCollection2DIterWithCoordinate<LIQ, Value> for QuantizableSpatialCollection2DVectorDense<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn iter_with_index_and_coordinate<'a>(&'a self) -> impl Iterator<Item=(LIQ, SpatialIndexCoordinate2D<LIQ>, &'a Value)>
    where
        Value: 'a
    {
        let dimensions = &self.dimensions;
        self.values
            .iter()
            .enumerate()
            .map(move |(index, value)| {
                let linear_index = LIQ::from_usize_unchecked(index);
                (linear_index, dimensions.linear_index_to_coordinate(linear_index), value)
            })
    }

    fn iter_mut_with_index_and_coordinate<'a>(&'a mut self) -> impl Iterator<Item=(LIQ, SpatialIndexCoordinate2D<LIQ>, &'a mut Value)>
    where
        Value: 'a
    {
        let dimensions = &self.dimensions;
        self.values
            .iter_mut()
            .enumerate()
            .map(move |(index, value)| {
                let linear_index = LIQ::from_usize_unchecked(index);
                (linear_index, dimensions.linear_index_to_coordinate(linear_index), value)
            })
    }
}
