use core::marker::PhantomData;
use crate::quantizable_base::{QuantizedIndexCountTrait, QuantizedIndexCountWrapperTrait};
use crate::quantizable_collections::QuantizableCollectionBaseTrait;
use crate::quantizable_spatial::unsigned_spatial_4d::{SpatialDimension4DTrait, SpatialUnsignedCoordinate4DTrait};

pub trait QuantizableSpatial4DCollectionUncheckedTrait<
    QuantIndex: QuantizedIndexCountTrait,
    WrappedLinearIndexCountType: QuantizedIndexCountWrapperTrait<QuantIndex>,
    Coordinate4DType: SpatialUnsignedCoordinate4DTrait<QuantIndex, WrappedLinearIndexCountType>,
    Dimension4DType: SpatialDimension4DTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate4DType>,
    Value
>:
QuantizableCollectionBaseTrait<QuantIndex, WrappedLinearIndexCountType, Value>
{
    fn get_4d_dimensions(&self) -> &Dimension4DType;

    /// Returns what coordinate a given linear index would be at, but does not verify that it exists (will always return a value though)
    fn calculate_coordinate_at_index_unverified(&self, linear_index: WrappedLinearIndexCountType) -> Coordinate4DType {
        self.get_4d_dimensions().linear_index_to_coordinate(&linear_index)
    }

    fn does_coordinate_exist(&self, coordinate: &Coordinate4DType) -> bool {
        let linear_index = self.get_4d_dimensions().coordinate_to_linear_index(coordinate);
        self.does_value_exist_at_linear_index(linear_index)
    }
}

trait IterItemCoordinate4DTrait<
    QuantIndex: QuantizedIndexCountTrait,
    WrappedLinearIndexCountType: QuantizedIndexCountWrapperTrait<QuantIndex>,
    Coordinate4DType: SpatialUnsignedCoordinate4DTrait<QuantIndex, WrappedLinearIndexCountType>,
    Dimension4DType: SpatialDimension4DTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate4DType>,
    Collection4DType: QuantizableSpatial4DCollectionUncheckedTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate4DType, Dimension4DType, Value>,
    Value,
>
{
    fn get_linear_index(&self) -> WrappedLinearIndexCountType;
    fn calculate_current_coordinate(&self) -> Coordinate4DType;
    fn get_value_ref(&self) -> &Value;
}

pub struct IterItemCoordinate4DRef<
    'a,
    QuantIndex: QuantizedIndexCountTrait,
    WrappedLinearIndexCountType: QuantizedIndexCountWrapperTrait<QuantIndex>,
    Coordinate4DType: SpatialUnsignedCoordinate4DTrait<QuantIndex, WrappedLinearIndexCountType>,
    Dimension4DType: SpatialDimension4DTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate4DType>,
    Collection4DType: QuantizableSpatial4DCollectionUncheckedTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate4DType, Dimension4DType, Value>,
    Value,
>
{
    linear_index: WrappedLinearIndexCountType,
    collection_ref: &'a Collection4DType,
    value_ref: &'a Value,
    _p: PhantomData<(QuantIndex, Coordinate4DType, Dimension4DType)>,
}

impl<
    'a,
    QuantIndex: QuantizedIndexCountTrait,
    WrappedLinearIndexCountType: QuantizedIndexCountWrapperTrait<QuantIndex>,
    Coordinate4DType: SpatialUnsignedCoordinate4DTrait<QuantIndex, WrappedLinearIndexCountType>,
    Dimension4DType: SpatialDimension4DTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate4DType>,
    Collection4DType: QuantizableSpatial4DCollectionUncheckedTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate4DType, Dimension4DType, Value>,
    Value,
> IterItemCoordinate4DTrait<
    QuantIndex,
    WrappedLinearIndexCountType,
    Coordinate4DType,
    Dimension4DType,
    Collection4DType,
    Value>
for IterItemCoordinate4DRef<
    'a,
    QuantIndex,
    WrappedLinearIndexCountType,
    Coordinate4DType,
    Dimension4DType,
    Collection4DType,
    Value
>
{
    fn get_linear_index(&self) -> WrappedLinearIndexCountType {
        self.linear_index
    }
    fn calculate_current_coordinate(&self) -> Coordinate4DType {
        self.collection_ref.calculate_coordinate_at_index_unverified(self.linear_index)
    }
    fn get_value_ref(&self) -> &Value {
        self.value_ref
    }
}

pub struct IterItemCoordinate4DRefMut<
    'a,
    QuantIndex: QuantizedIndexCountTrait,
    WrappedLinearIndexCountType: QuantizedIndexCountWrapperTrait<QuantIndex>,
    Coordinate4DType: SpatialUnsignedCoordinate4DTrait<QuantIndex, WrappedLinearIndexCountType>,
    Dimension4DType: SpatialDimension4DTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate4DType>,
    Collection4DType: QuantizableSpatial4DCollectionUncheckedTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate4DType, Dimension4DType, Value>,
    Value,
>
{
    linear_index: WrappedLinearIndexCountType,
    collection_ref: &'a Collection4DType,
    value_ref: &'a mut Value,
    _p: PhantomData<(QuantIndex, Coordinate4DType, Dimension4DType)>,
}

impl<
    'a,
    QuantIndex: QuantizedIndexCountTrait,
    WrappedLinearIndexCountType: QuantizedIndexCountWrapperTrait<QuantIndex>,
    Coordinate4DType: SpatialUnsignedCoordinate4DTrait<QuantIndex, WrappedLinearIndexCountType>,
    Dimension4DType: SpatialDimension4DTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate4DType>,
    Collection4DType: QuantizableSpatial4DCollectionUncheckedTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate4DType, Dimension4DType, Value>,
    Value,
> IterItemCoordinate4DTrait<
    QuantIndex,
    WrappedLinearIndexCountType,
    Coordinate4DType,
    Dimension4DType,
    Collection4DType,
    Value> for IterItemCoordinate4DRefMut<
    'a,
    QuantIndex,
    WrappedLinearIndexCountType,
    Coordinate4DType, Dimension4DType,
    Collection4DType,
    Value
>
{
    fn get_linear_index(&self) -> WrappedLinearIndexCountType {
        self.linear_index
    }
    fn calculate_current_coordinate(&self) -> Coordinate4DType {
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
    Coordinate4DType: SpatialUnsignedCoordinate4DTrait<QuantIndex, WrappedLinearIndexCountType>,
    Dimension4DType: SpatialDimension4DTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate4DType>,
    Collection4DType: QuantizableSpatial4DCollectionUncheckedTrait<QuantIndex, WrappedLinearIndexCountType, Coordinate4DType, Dimension4DType, Value>,
    Value,
> IterItemCoordinate4DRefMut<
    'a,
    QuantIndex,
    WrappedLinearIndexCountType,
    Coordinate4DType,
    Dimension4DType,
    Collection4DType,
    Value
>
{
    pub fn get_value_ref_mut(&mut self) -> &mut Value {
        self.value_ref
    }
}
