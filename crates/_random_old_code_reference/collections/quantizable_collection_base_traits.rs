use crate::quantizable::base_types::{QuantizedIndexCountTrait, QuantizedIndexCountWrapperTrait};
use crate::quantizable::FeagiDataQuantizedError;

/// Defines base Quantizable collections without methods that return enums (GPU friendly)
pub trait QuantizableCollectionBaseTrait<
    QuantIndex: QuantizedIndexCountTrait,
    WrappedIndexCountType: QuantizedIndexCountWrapperTrait<QuantIndex>,
    Value
>
{
    fn get_element_max_capacity(&self) -> WrappedIndexCountType;
    fn get_number_of_elements(&self) -> WrappedIndexCountType;

    fn does_value_exist_at_linear_index(&self, linear_index: WrappedIndexCountType) -> bool;
    fn get_value_at_linear_index_unchecked(&self, linear_index: WrappedIndexCountType) -> &Value;
    fn get_value_at_linear_index_unchecked_mut(&mut self, linear_index: WrappedIndexCountType) -> &mut Value;
}

/// Adds checking index retrieval to a QuantizableCollectionBaseTrait
pub trait QuantizableCollectionBaseCheckedTrait<
    QuantIndex: QuantizedIndexCountTrait,
    WrappedIndexCountType: QuantizedIndexCountWrapperTrait<QuantIndex>,
    Value
>:
QuantizableCollectionBaseTrait<QuantIndex, WrappedIndexCountType, Value>
{
    const INVALID_LINEAR_INDEX_STR: &'static str = "Invalid linear index!";

    fn get_value_at_linear_index_checked(&self, linear_index: WrappedIndexCountType) -> Result<&Value, FeagiDataQuantizedError> {
        if ! self.does_value_exist_at_linear_index(linear_index) {
            return Err(FeagiDataQuantizedError::CollectionInvalidIndexError {
                context: Self::INVALID_LINEAR_INDEX_STR,
                invalid_index: linear_index.quant_ref().to_u32(),
            })
        }
        Ok(self.get_value_at_linear_index_unchecked(linear_index))
    }
    fn get_value_at_linear_index_checked_mut(&mut self, linear_index: WrappedIndexCountType) -> Result<&mut Value, FeagiDataQuantizedError> {
        if ! self.does_value_exist_at_linear_index(linear_index) {
            return Err(FeagiDataQuantizedError::CollectionInvalidIndexError {
                context: Self::INVALID_LINEAR_INDEX_STR,
                invalid_index: linear_index.quant_ref().to_u32(),
            })
        }
        Ok(self.get_value_at_linear_index_unchecked_mut(linear_index))
    }
}


/// Adds Base Iter support to all Quantizable Collections
pub trait QuantizableCollectionBaseIterTrait<
    QuantIndex: QuantizedIndexCountTrait,
    WrappedIndexCountType: QuantizedIndexCountWrapperTrait<QuantIndex>,
    Value
>:
QuantizableCollectionBaseTrait<QuantIndex, WrappedIndexCountType, Value>
{
    fn iter_through_indexed_values<'a>(&self) -> impl Iterator<Item=(WrappedIndexCountType, &'a Value)> where Value: 'a;
    fn iter_through_indexed_values_mut<'a>(&self) -> impl Iterator<Item=(WrappedIndexCountType, &'a mut Value)> where Value: 'a;
}

// TODO RAYON