use core::marker::PhantomData;
use crate::quantizable_base::{QuantizedIndexCountTrait, QuantizedIndexCountWrapperTrait};
use crate::quantizable_collections::QuantizableCollectionBaseTrait;
use crate::quantizable_spatial::unsigned_spatial_3d::{SpatialDimension3DTrait, SpatialUnsignedCoordinate3DTrait};

/// Base trait of all 3D Collections
pub trait QuantizableSpatial3DCollectionUncheckedTrait<
    QuantIndex: QuantizedIndexCountTrait,
    WrappedLinearIndexCountType: QuantizedIndexCountWrapperTrait<QuantIndex>,
    Coordinate3DType: SpatialUnsignedCoordinate3DTrait<QuantIndex, WrappedLinearIndexCountType>,
    Dimension3DType: SpatialDimension3DTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate3DType>,
    Value
>:
QuantizableCollectionBaseTrait<QuantIndex, WrappedLinearIndexCountType, Value>
{
    fn get_3d_dimensions(&self) -> &Dimension3DType;

    /// Returns what coordinate a given linear index would be at, but does not verify that it exists (will always return a value though)
    fn calculate_coordinate_at_index_unverified(&self, linear_index: WrappedLinearIndexCountType) -> Coordinate3DType {
        self.get_3d_dimensions().linear_index_to_coordinate(&linear_index)
    }

    fn does_coordinate_exist(&self, coordinate: &Coordinate3DType) -> bool {
        let linear_index = self.get_3d_dimensions().coordinate_to_linear_index(coordinate);
        self.does_value_exist_at_linear_index(linear_index)
    }
}




/// 3D Trait for collections that are dense (not sparse)
pub trait QuantizableSpatial3DCollectionDenseUncheckedTrait<
    QuantIndex: QuantizedIndexCountTrait,
    WrappedLinearIndexCountType: QuantizedIndexCountWrapperTrait<QuantIndex>,
    Coordinate3DType: SpatialUnsignedCoordinate3DTrait<QuantIndex, WrappedLinearIndexCountType>,
    Dimension3DType: SpatialDimension3DTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate3DType>,
    Value
>:
QuantizableSpatial3DCollectionUncheckedTrait<
    QuantIndex,
    WrappedLinearIndexCountType,
    Coordinate3DType,
    Dimension3DType,
    Value
>
{
    fn get_values_as_slice(&self) -> &[Value];
    fn get_values_as_mut_slice(&mut self) -> &mut [Value];
}


//region Single Item Iteration

// Adds iteration support
pub trait QuantizableSpatial3DCollectionIndexedIter<
    QuantIndex: QuantizedIndexCountTrait,
    WrappedLinearIndexCountType: QuantizedIndexCountWrapperTrait<QuantIndex>,
    Coordinate3DType: SpatialUnsignedCoordinate3DTrait<QuantIndex, WrappedLinearIndexCountType>,
    Dimension3DType: SpatialDimension3DTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate3DType>,
    Value
>:QuantizableSpatial3DCollectionUncheckedTrait<
    QuantIndex,
    WrappedLinearIndexCountType,
    Coordinate3DType,
    Dimension3DType,
    Value
>
{
    
}


trait IterItemCoordinate3DTrait<
    QuantIndex: QuantizedIndexCountTrait,
    WrappedLinearIndexCountType: QuantizedIndexCountWrapperTrait<QuantIndex>,
    Coordinate3DType: SpatialUnsignedCoordinate3DTrait<QuantIndex, WrappedLinearIndexCountType>,
    Dimension3DType: SpatialDimension3DTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate3DType>,
    Collection3DType: QuantizableSpatial3DCollectionUncheckedTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate3DType, Dimension3DType, Value>,
    Value,
>
{
    fn get_linear_index(&self) -> WrappedLinearIndexCountType;
    fn calculate_current_coordinate(&self) -> Coordinate3DType;
    fn get_value_ref(&self) -> &Value;
}

pub struct IterItemCoordinate3DRef<
    'a,
    QuantIndex: QuantizedIndexCountTrait,
    WrappedLinearIndexCountType: QuantizedIndexCountWrapperTrait<QuantIndex>,
    Coordinate3DType: SpatialUnsignedCoordinate3DTrait<QuantIndex, WrappedLinearIndexCountType>,
    Dimension3DType: SpatialDimension3DTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate3DType>,
    Collection3DType: QuantizableSpatial3DCollectionUncheckedTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate3DType, Dimension3DType, Value>,
    Value,
>
{
    linear_index: WrappedLinearIndexCountType,
    collection_ref: &'a Collection3DType,
    value_ref: &'a Value,
    _p: PhantomData<(QuantIndex, Coordinate3DType, Dimension3DType)>,
}

impl<
    'a,
    QuantIndex: QuantizedIndexCountTrait,
    WrappedLinearIndexCountType: QuantizedIndexCountWrapperTrait<QuantIndex>,
    Coordinate3DType: SpatialUnsignedCoordinate3DTrait<QuantIndex, WrappedLinearIndexCountType>,
    Dimension3DType: SpatialDimension3DTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate3DType>,
    Collection3DType: QuantizableSpatial3DCollectionUncheckedTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate3DType, Dimension3DType, Value>,
    Value,
> IterItemCoordinate3DTrait<
    QuantIndex,
    WrappedLinearIndexCountType,
    Coordinate3DType,
    Dimension3DType,
    Collection3DType,
    Value>
for IterItemCoordinate3DRef<
    'a,
    QuantIndex,
    WrappedLinearIndexCountType,
    Coordinate3DType,
    Dimension3DType,
    Collection3DType,
    Value
>
{
    fn get_linear_index(&self) -> WrappedLinearIndexCountType {
        self.linear_index
    }
    fn calculate_current_coordinate(&self) -> Coordinate3DType {
        self.collection_ref.calculate_coordinate_at_index_unverified(self.linear_index)
    }
    fn get_value_ref(&self) -> &Value {
        self.value_ref
    }
}

pub struct IterItemCoordinate3DRefMut<
    'a,
    QuantIndex: QuantizedIndexCountTrait,
    WrappedLinearIndexCountType: QuantizedIndexCountWrapperTrait<QuantIndex>,
    Coordinate3DType: SpatialUnsignedCoordinate3DTrait<QuantIndex, WrappedLinearIndexCountType>,
    Dimension3DType: SpatialDimension3DTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate3DType>,
    Collection3DType: QuantizableSpatial3DCollectionUncheckedTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate3DType, Dimension3DType, Value>,
    Value,
>
{
    linear_index: WrappedLinearIndexCountType,
    collection_ref: &'a Collection3DType,
    value_ref: &'a mut Value,
    _p: PhantomData<(QuantIndex, Coordinate3DType, Dimension3DType)>,
}

impl<
    'a,
    QuantIndex: QuantizedIndexCountTrait,
    WrappedLinearIndexCountType: QuantizedIndexCountWrapperTrait<QuantIndex>,
    Coordinate3DType: SpatialUnsignedCoordinate3DTrait<QuantIndex, WrappedLinearIndexCountType>,
    Dimension3DType: SpatialDimension3DTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate3DType>,
    Collection3DType: QuantizableSpatial3DCollectionUncheckedTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate3DType, Dimension3DType, Value>,
    Value,
> IterItemCoordinate3DTrait<
    QuantIndex,
    WrappedLinearIndexCountType,
    Coordinate3DType,
    Dimension3DType,
    Collection3DType,
    Value> for IterItemCoordinate3DRefMut<
    'a,
    QuantIndex,
    WrappedLinearIndexCountType,
    Coordinate3DType, Dimension3DType,
    Collection3DType,
    Value
>
{
    fn get_linear_index(&self) -> WrappedLinearIndexCountType {
        self.linear_index
    }
    fn calculate_current_coordinate(&self) -> Coordinate3DType {
        self.collection_ref.calculate_coordinate_at_index_unverified(self.linear_index)
    }
    fn get_value_ref(&self) -> &Value {
        self.value_ref
    }
}

impl<
    'a,
    QuantIndex: QuantizedIndexCountTrait,
    WrappedLinearIndexCountType: QuantizedIndexCountWrapperTrait<QuantIndex>,
    Coordinate3DType: SpatialUnsignedCoordinate3DTrait<QuantIndex, WrappedLinearIndexCountType>,
    Dimension3DType: SpatialDimension3DTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate3DType>,
    Collection3DType: QuantizableSpatial3DCollectionUncheckedTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate3DType, Dimension3DType, Value>,
    Value,
> IterItemCoordinate3DRefMut<
    'a,
    QuantIndex,
    WrappedLinearIndexCountType,
    Coordinate3DType,
    Dimension3DType,
    Collection3DType,
    Value
>
{
    pub fn get_value_ref_mut(&mut self) -> &mut Value {
        self.value_ref
    }
}

// TODO RAYON

//endregion

