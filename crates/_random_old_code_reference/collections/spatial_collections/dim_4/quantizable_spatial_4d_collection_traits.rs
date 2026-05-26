use core::marker::PhantomData;
use crate::quantizable::base_types::{QuantizedIndexCountTrait, QuantizedIndexCountWrapperTrait};
use crate::quantizable::collections::QuantizableCollectionBaseTrait;
use crate::quantizable::spatial::unsigned_spatial_4d::{SpatialDimension4DTrait, SpatialUnsignedCoordinate4DTrait};

pub trait QuantizableSpatial4DQuantTypes {
    type LinearIndex: QuantizedIndexCountTrait;
    type WrappedLinearIndexType: QuantizedIndexCountWrapperTrait<Self::LinearIndex>;
    type Coord4DType: SpatialUnsignedCoordinate4DTrait<Self::LinearIndex, Self::WrappedLinearIndexType>;
    type Dim4DType: SpatialDimension4DTrait<Self::LinearIndex, Self::WrappedLinearIndexType, Self::Coord4DType>;
    type Value;
}

/// Base trait of all 4D Collections
pub trait QuantizableSpatial4DCollectionUncheckedTrait<
    Q4D: QuantizableSpatial4DQuantTypes,
>:
QuantizableCollectionBaseTrait<Q4D::LinearIndex, Q4D::WrappedLinearIndexType, Q4D::Value>
{
    fn get_4d_dimensions(&self) -> &Q4D::Dim4DType;

    /// Returns what coordinate a given linear index would be at, but does not verify that it exists (will always return a value though)
    fn calculate_coordinate_at_index_unverified(&self, linear_index: Q4D::WrappedLinearIndexType) -> Q4D::Coord4DType {
        self.get_4d_dimensions().linear_index_to_coordinate(&linear_index)
    }

    fn does_coordinate_exist(&self, coordinate: &Q4D::Coord4DType) -> bool {
        let linear_index = self.get_4d_dimensions().coordinate_to_linear_index(coordinate);
        self.does_value_exist_at_linear_index(linear_index)
    }
}

/// 4D Trait for collections that are dense (not sparse)
pub trait QuantizableSpatial4DCollectionDenseUncheckedTrait<
    Q4D: QuantizableSpatial4DQuantTypes,
>:
QuantizableSpatial4DCollectionUncheckedTrait<
    Q4D,
>
{
    fn get_values_as_slice(&self) -> &[Q4D::Value];
    fn get_values_as_mut_slice(&mut self) -> &mut [Q4D::Value];
}


//region Single Item Iteration

// Adds iteration support
pub trait QuantizableSpatial4DCollectionIndexedIter<
    Q4D: QuantizableSpatial4DQuantTypes,
>:QuantizableSpatial4DCollectionUncheckedTrait<
    Q4D,
>
{

}

trait IterItemCoordinate4DTrait<
    Q4D: QuantizableSpatial4DQuantTypes,
    Collection4DType: QuantizableSpatial4DCollectionUncheckedTrait<Q4D>,
>
{
    fn get_linear_index(&self) -> Q4D::WrappedLinearIndexType;
    fn calculate_current_coordinate(&self) -> Q4D::Coord4DType;
    fn get_value_ref(&self) -> &Q4D::Value;
}

pub struct IterItemCoordinate4DRef<
    'a,
    Q4D: QuantizableSpatial4DQuantTypes,
    Collection4DType: QuantizableSpatial4DCollectionUncheckedTrait<Q4D>,
>
{
    linear_index: Q4D::WrappedLinearIndexType,
    collection_ref: &'a Collection4DType,
    value_ref: &'a Q4D::Value,
}

impl<
    'a,
    Q4D: QuantizableSpatial4DQuantTypes,
    Collection4DType: QuantizableSpatial4DCollectionUncheckedTrait<Q4D>>
IterItemCoordinate4DTrait <
    Q4D,
    Collection4DType>
for IterItemCoordinate4DRef <
    'a,
    Q4D,
    Collection4DType,
>
{
    fn get_linear_index(&self) -> Q4D::WrappedLinearIndexType {
        self.linear_index
    }
    fn calculate_current_coordinate(&self) -> Q4D::Coord4DType {
        self.collection_ref.calculate_coordinate_at_index_unverified(self.linear_index)
    }
    fn get_value_ref(&self) -> &Q4D::Value {
        self.value_ref
    }
}

pub struct IterItemCoordinate4DRefMut<
    'a,
    Q4D: QuantizableSpatial4DQuantTypes,
    Collection4DType: QuantizableSpatial4DCollectionUncheckedTrait<Q4D>,
>
{
    linear_index: Q4D::WrappedLinearIndexType,
    collection_ref: &'a Collection4DType,
    value_ref: &'a mut Q4D::Value,
}

impl<
    'a,
    Q4D: QuantizableSpatial4DQuantTypes,
    Collection4DType: QuantizableSpatial4DCollectionUncheckedTrait<Q4D>,
> IterItemCoordinate4DTrait<
    Q4D,
    Collection4DType,
    > for IterItemCoordinate4DRefMut<
    'a,
    Q4D,
    Collection4DType,
>
{
    fn get_linear_index(&self) -> Q4D::WrappedLinearIndexType {
        self.linear_index
    }
    fn calculate_current_coordinate(&self) -> Q4D::Coord4DType {
        self.collection_ref.calculate_coordinate_at_index_unverified(self.linear_index)
    }
    fn get_value_ref(&self) -> &Q4D::Value {
        self.value_ref
    }
}

impl<
    'a,
    Q4D: QuantizableSpatial4DQuantTypes,
    Collection4DType: QuantizableSpatial4DCollectionUncheckedTrait<Q4D>,
> IterItemCoordinate4DRefMut<
    'a,
    Q4D,
    Collection4DType,
>
{
    pub fn get_value_ref_mut(&mut self) -> &mut Q4D::Value {
        self.value_ref
    }
}

// TODO RAYON

//endregion
