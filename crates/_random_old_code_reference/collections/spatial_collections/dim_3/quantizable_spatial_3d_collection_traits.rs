use core::marker::PhantomData;
use crate::quantizable::base_types::{QuantizedIndexCountTrait, QuantizedIndexCountWrapperTrait};
use crate::quantizable::collections::QuantizableCollectionBaseTrait;
use crate::quantizable::spatial::unsigned_spatial_3d::{SpatialDimension3DTrait, SpatialUnsignedCoordinate3DTrait};

pub trait QuantizableSpatial3DQuantTypes {
    type LinearIndex: QuantizedIndexCountTrait;
    type WrappedLinearIndexType: QuantizedIndexCountWrapperTrait<Self::LinearIndex>;
    type Coord3DType: SpatialUnsignedCoordinate3DTrait<Self::LinearIndex, Self::WrappedLinearIndexType>;
    type Dim3DType: SpatialDimension3DTrait<Self::LinearIndex, Self::WrappedLinearIndexType, Self::Coord3DType>;
    type Value;
}

/// Base trait of all 3D Collections
pub trait QuantizableSpatial3DCollectionUncheckedTrait<
    Q3D: QuantizableSpatial3DQuantTypes,
>:
QuantizableCollectionBaseTrait<Q3D::LinearIndex, Q3D::WrappedLinearIndexType, Q3D::Value>
{
    fn get_3d_dimensions(&self) -> &Q3D::Dim3DType;

    /// Returns what coordinate a given linear index would be at, but does not verify that it exists (will always return a value though)
    fn calculate_coordinate_at_index_unverified(&self, linear_index: Q3D::WrappedLinearIndexType) -> Q3D::Coord3DType {
        self.get_3d_dimensions().linear_index_to_coordinate(&linear_index)
    }

    fn does_coordinate_exist(&self, coordinate: &Q3D::Coord3DType) -> bool {
        let linear_index = self.get_3d_dimensions().coordinate_to_linear_index(coordinate);
        self.does_value_exist_at_linear_index(linear_index)
    }
}




/// 3D Trait for collections that are dense (not sparse)
pub trait QuantizableSpatial3DCollectionDenseUncheckedTrait<
    Q3D: QuantizableSpatial3DQuantTypes,
>:
QuantizableSpatial3DCollectionUncheckedTrait<
    Q3D,
>
{
    fn get_values_as_slice(&self) -> &[Q3D::Value];
    fn get_values_as_mut_slice(&mut self) -> &mut [Q3D::Value];
}


//region Single Item Iteration

// Adds iteration support
pub trait QuantizableSpatial3DCollectionIndexedIter<
    Q3D: QuantizableSpatial3DQuantTypes,
>:QuantizableSpatial3DCollectionUncheckedTrait<
    Q3D,
>
{

}

trait IterItemCoordinate3DTrait<
    Q3D: QuantizableSpatial3DQuantTypes,
    Collection3DType: QuantizableSpatial3DCollectionUncheckedTrait<Q3D>,
>
{
    fn get_linear_index(&self) -> Q3D::WrappedLinearIndexType;
    fn calculate_current_coordinate(&self) -> Q3D::Coord3DType;
    fn get_value_ref(&self) -> &Q3D::Value;
}

pub struct IterItemCoordinate3DRef<
    'a,
    Q3D: QuantizableSpatial3DQuantTypes,
    Collection3DType: QuantizableSpatial3DCollectionUncheckedTrait<Q3D>,
>
{
    linear_index: Q3D::WrappedLinearIndexType,
    collection_ref: &'a Collection3DType,
    value_ref: &'a Q3D::Value,
}

impl<
    'a,
    Q3D: QuantizableSpatial3DQuantTypes,
    Collection3DType: QuantizableSpatial3DCollectionUncheckedTrait<Q3D>>
IterItemCoordinate3DTrait <
    Q3D,
    Collection3DType>
for IterItemCoordinate3DRef <
    'a,
    Q3D,
    Collection3DType,
>
{
    fn get_linear_index(&self) -> Q3D::WrappedLinearIndexType {
        self.linear_index
    }
    fn calculate_current_coordinate(&self) -> Q3D::Coord3DType {
        self.collection_ref.calculate_coordinate_at_index_unverified(self.linear_index)
    }
    fn get_value_ref(&self) -> &Q3D::Value {
        self.value_ref
    }
}

pub struct IterItemCoordinate3DRefMut<
    'a,
    Q3D: QuantizableSpatial3DQuantTypes,
    Collection3DType: QuantizableSpatial3DCollectionUncheckedTrait<Q3D>,
>
{
    linear_index: Q3D::WrappedLinearIndexType,
    collection_ref: &'a Collection3DType,
    value_ref: &'a mut Q3D::Value,
}

impl<
    'a,
    Q3D: QuantizableSpatial3DQuantTypes,
    Collection3DType: QuantizableSpatial3DCollectionUncheckedTrait<Q3D>,
> IterItemCoordinate3DTrait<
    Q3D,
    Collection3DType,
    > for IterItemCoordinate3DRefMut<
    'a,
    Q3D,
    Collection3DType,
>
{
    fn get_linear_index(&self) -> Q3D::WrappedLinearIndexType {
        self.linear_index
    }
    fn calculate_current_coordinate(&self) -> Q3D::Coord3DType {
        self.collection_ref.calculate_coordinate_at_index_unverified(self.linear_index)
    }
    fn get_value_ref(&self) -> &Q3D::Value {
        self.value_ref
    }
}

impl<
    'a,
    Q3D: QuantizableSpatial3DQuantTypes,
    Collection3DType: QuantizableSpatial3DCollectionUncheckedTrait<Q3D>,
> IterItemCoordinate3DRefMut<
    'a,
    Q3D,
    Collection3DType,
>
{
    pub fn get_value_ref_mut(&mut self) -> &mut Q3D::Value {
        self.value_ref
    }
}

// TODO RAYON

//endregion

